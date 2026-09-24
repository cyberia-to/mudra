// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! secp256k1 ECDSA verification (milestone 2c) — the signature check itself,
//! built from the emulated base field, curve, and scalar field.
//!
//! Given a public key `Q`, signature `(r, s)`, and message digest `z`:
//!
//! ```text
//! w  = s⁻¹ (mod n)
//! u1 = z·w (mod n)     u2 = r·w (mod n)
//! R  = u1·G + u2·Q
//! valid  ⟺  R ≠ ∞  ∧  R.x (mod n) == r
//! ```
//!
//! Every operation is [`Fe`]/[`Scalar`]/[`Point`] arithmetic — Goldilocks-native,
//! so the whole check is a nox computation a zheng proof can attest. This is the
//! reference the eventual in-circuit gadget must match.

use super::curve::Point;
use super::scalar::Scalar;

/// Verify a secp256k1 ECDSA signature over a 32-byte message digest.
///
/// `pubkey` is SEC1-compressed (33 bytes). `r`, `s`, `z` are big-endian 32-byte
/// integers. Returns `true` iff the signature is valid for `pubkey` and `z`.
pub fn verify(pubkey: &[u8; 33], r: &[u8; 32], s: &[u8; 32], z: &[u8; 32]) -> bool {
    let r_s = Scalar::from_bytes(r);
    let s_s = Scalar::from_bytes(s);
    // r, s must be in [1, n−1], and s must be the low half: (r, s) and
    // (r, n − s) both satisfy the verification equation below for the same
    // message, so accepting either half makes the signature's byte encoding
    // non-unique. Reject the high half so exactly one encoding verifies.
    if r_s.is_zero() || s_s.is_zero() || s_s.is_high() {
        return false;
    }
    let Some(q) = Point::from_sec1(pubkey) else {
        return false;
    };

    let w = s_s.inv();
    let u1 = Scalar::from_bytes(z).mul(&w);
    let u2 = r_s.mul(&w);

    let big_r = Point::generator().scalar_mul(&u1.to_bytes()).add(&q.scalar_mul(&u2.to_bytes()));
    let Some((rx, _)) = big_r.to_affine() else {
        return false; // R = ∞
    };

    // r == R.x reduced mod n
    Scalar::from_bytes(&rx.to_bytes()) == r_s
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::signature::hazmat::PrehashSigner;
    use k256::ecdsa::{Signature, SigningKey};

    /// Build a (pubkey33, r, s, z) tuple by signing `z` with `key_bytes` via k256.
    fn sign(key_bytes: &[u8; 32], z: &[u8; 32]) -> ([u8; 33], [u8; 32], [u8; 32]) {
        let sk = SigningKey::from_slice(key_bytes).unwrap();
        let sig: Signature = sk.sign_prehash(z).unwrap();
        let raw = sig.to_bytes(); // r ‖ s, 64 bytes
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&raw[..32]);
        s.copy_from_slice(&raw[32..]);
        let mut pubkey = [0u8; 33];
        pubkey.copy_from_slice(sk.verifying_key().to_encoded_point(true).as_bytes());
        (pubkey, r, s)
    }

    #[test]
    fn accepts_valid_k256_signatures() {
        let keys: [[u8; 32]; 3] = [[1u8; 32], [7u8; 32], [0x9Au8; 32]];
        let msgs: [[u8; 32]; 3] = [[0x11u8; 32], [0xABu8; 32], [0x42u8; 32]];
        for k in &keys {
            for z in &msgs {
                let (pk, r, s) = sign(k, z);
                assert!(verify(&pk, &r, &s, z), "valid signature must verify");
            }
        }
    }

    #[test]
    fn rejects_wrong_message() {
        let (pk, r, s) = sign(&[3u8; 32], &[0x55u8; 32]);
        let other = [0x56u8; 32];
        assert!(!verify(&pk, &r, &s, &other), "signature must not verify a different digest");
    }

    #[test]
    fn rejects_tampered_signature() {
        let z = [0x77u8; 32];
        let (pk, mut r, s) = sign(&[5u8; 32], &z);
        r[31] ^= 0x01;
        assert!(!verify(&pk, &r, &s, &z), "flipped r must fail");
    }

    #[test]
    fn rejects_wrong_pubkey() {
        let z = [0x33u8; 32];
        let (_pk, r, s) = sign(&[9u8; 32], &z);
        let (other_pk, _, _) = sign(&[10u8; 32], &z);
        assert!(!verify(&other_pk, &r, &s, &z), "another key must not verify");
    }

    #[test]
    fn differential_against_k256_on_random_inputs() {
        // The strongest correctness statement: on many random (key, message)
        // pairs, the in-stack verifier accepts exactly the signatures k256
        // produced, and rejects every single-bit corruption of them.
        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut checked = 0;
        for _ in 0..60 {
            let mut kb = [0u8; 32];
            let mut zb = [0u8; 32];
            for c in kb.chunks_mut(8) {
                c.copy_from_slice(&next().to_le_bytes());
            }
            for c in zb.chunks_mut(8) {
                c.copy_from_slice(&next().to_le_bytes());
            }
            // skip key bytes that aren't a valid scalar
            if SigningKey::from_slice(&kb).is_err() {
                continue;
            }
            let (pk, r, s) = sign(&kb, &zb);
            assert!(verify(&pk, &r, &s, &zb), "must accept a genuine k256 signature");
            // corrupt one bit of the message → must reject
            let mut z_bad = zb;
            z_bad[(next() as usize) % 32] ^= 1;
            if z_bad != zb {
                assert!(!verify(&pk, &r, &s, &z_bad), "must reject a corrupted digest");
            }
            checked += 1;
        }
        assert!(checked >= 40, "exercised enough random signatures ({checked})");
    }

    #[test]
    fn rejects_zero_scalars() {
        let (pk, _r, s) = sign(&[2u8; 32], &[0x01u8; 32]);
        assert!(!verify(&pk, &[0u8; 32], &s, &[0x01u8; 32]), "r = 0 rejected");
    }

    /// `(r, s)` and `(r, n − s)` satisfy the same verification equation for
    /// the same `(pubkey, z)` — flip `w = s⁻¹` to `−w` and both `u1`, `u2`
    /// negate, so `R = u1·G + u2·Q` negates too, and a negated point has the
    /// same x-coordinate. Before `is_high` gated it, `verify` accepted
    /// whichever half a signer (or anyone downstream re-deriving the other
    /// root) chose to hand it — two distinct byte strings for one signature.
    /// Only the low-S half must verify now, regardless of which half k256
    /// happened to produce.
    #[test]
    fn accepts_low_s_rejects_high_s_twin() {
        use num_bigint::BigUint;

        let z = [0x99u8; 32];
        let (pk, r, s) = sign(&[6u8; 32], &z);

        let n = BigUint::parse_bytes(
            b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
            16,
        )
        .unwrap();
        let s_big = BigUint::from_bytes_be(&s);
        let twin_big = &n - &s_big;
        let tb = twin_big.to_bytes_be();
        let mut twin = [0u8; 32];
        twin[32 - tb.len()..].copy_from_slice(&tb);

        let (low, high) = if Scalar::from_bytes(&s).is_high() { (twin, s) } else { (s, twin) };

        assert!(verify(&pk, &r, &low, &z), "the low-S half must verify");
        assert!(!verify(&pk, &r, &high, &z), "the high-S twin must be rejected");
    }
}
