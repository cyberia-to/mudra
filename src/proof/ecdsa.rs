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
    // r, s must be in [1, n−1] as the literal 32-byte big-endian integers they
    // are, not merely reduce to a nonzero residue: `Scalar::from_bytes` folds
    // any input mod n, so a byte string with value in [n, 2^256) would verify
    // identically to its reduced form — a second, non-canonical encoding of
    // the same signature (signature malleability at the wire level). Round-
    // tripping through `to_bytes` catches exactly that: it differs from the
    // input iff the input was not already < n.
    if r_s.is_zero() || s_s.is_zero() || r_s.to_bytes() != *r || s_s.to_bytes() != *s {
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
    use super::super::field::Fe;
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

    /// secp256k1's order `n` is only ~2^128 below `2^256`, so for a small `r`
    /// there is a second, distinct 32-byte encoding `r' = r + n` that
    /// `Scalar::from_bytes` reduces back to the same residue. Before the
    /// canonical-encoding check, `verify` accepted `r'` as readily as `r` —
    /// a non-canonical malleable signature encoding that never occurs from
    /// real `R.x` arithmetic (`Scalar::to_bytes` always emits the reduced,
    /// canonical form) but is trivial for an attacker to hand a verifier.
    ///
    /// A genuine signature's `r` is the x-coordinate of a random curve point
    /// and so is essentially never small (< 2^128) by chance — real ECDSA
    /// signing can't exercise this deterministically. Build a small-`r`
    /// signature directly from the verification equation instead: with
    /// `s = 1`, `z = 0` (so `w = 1`, `u1 = 0`, `u2 = r`), `verify` accepts
    /// any `(pubkey, r, s, z)` for which `R = r·Q` has x-coordinate `r`,
    /// i.e. `Q = r⁻¹·P` for any point `P` with x-coordinate exactly `r`.
    /// Solving the curve equation `y² = x³ + 7` at a small `x` gives such a
    /// `P` directly, with no discrete log needed.
    #[test]
    fn rejects_non_canonical_r_encoding() {
        // Smallest x >= 1 for which x³ + 7 is a quadratic residue mod p.
        let (r_bytes, p_point) = (1u64..64)
            .find_map(|x_val| {
                let mut xb = [0u8; 32];
                xb[24..].copy_from_slice(&x_val.to_be_bytes());
                let x = Fe::from_bytes(&xb);
                let mut seven = [0u8; 32];
                seven[31] = 7;
                let rhs = x.sqr().mul(&x).add(&Fe::from_bytes(&seven));
                rhs.sqrt().map(|y| (xb, Point::from_affine(x, y)))
            })
            .expect("at least one small x is a valid curve point");

        let r_s = Scalar::from_bytes(&r_bytes);
        assert!(!r_s.is_zero());
        let q = p_point.scalar_mul(&r_s.inv().to_bytes()); // Q = r⁻¹·P
        let (qx, qy) = q.to_affine().unwrap();
        let mut pubkey = [0u8; 33];
        pubkey[0] = 0x02 | qy.parity();
        pubkey[1..].copy_from_slice(&qx.to_bytes());

        let s_bytes = Scalar::ONE.to_bytes();
        let z_bytes = [0u8; 32]; // z = 0 ⇒ u1 = 0, u2 = r·s⁻¹ = r

        assert!(
            verify(&pubkey, &r_bytes, &s_bytes, &z_bytes),
            "canonical small-r signature must verify"
        );

        // n = 0xFFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFE_BAAEDCE6_AF48A03B_BFD25E8C_D0364141
        const ORDER: [u8; 32] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFE, 0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C,
            0xD0, 0x36, 0x41, 0x41,
        ];
        let mut r_plus_n = [0u8; 32];
        let mut carry = 0u16;
        for i in (0..32).rev() {
            let sum = r_bytes[i] as u16 + ORDER[i] as u16 + carry;
            r_plus_n[i] = sum as u8;
            carry = sum >> 8;
        }
        assert_eq!(carry, 0, "r (tiny) + n must fit in 32 bytes");
        assert_ne!(r_plus_n, r_bytes, "r + n must be a distinct byte string from r");
        assert_eq!(
            Scalar::from_bytes(&r_plus_n),
            r_s,
            "r + n must reduce to the same residue as r"
        );
        assert!(
            !verify(&pubkey, &r_plus_n, &s_bytes, &z_bytes),
            "non-canonical r encoding (r + n) must be rejected, not silently reduced"
        );
    }
}
