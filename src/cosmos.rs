// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Cosmos-SDK account addresses.
//!
//! A Cosmos account address is `bech32(hrp, ripemd160(sha256(compressed_pubkey)))`.
//! The 20-byte hash is the account id; the HRP (human-readable prefix) names the
//! chain. spacepussy uses `pussy`, bostrom uses `bostrom`.

use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use k256::ecdsa::VerifyingKey;

use crate::Error;

/// spacepussy address prefix.
pub const PUSSY: &str = "pussy";
/// bostrom address prefix.
pub const BOSTROM: &str = "bostrom";

/// The 33-byte SEC1-compressed public key for a verifying key.
pub fn compressed(vk: &VerifyingKey) -> [u8; 33] {
    let point = vk.to_encoded_point(true);
    let mut out = [0u8; 33];
    out.copy_from_slice(point.as_bytes());
    out
}

/// The 20-byte Cosmos account id: `ripemd160(sha256(compressed_pubkey))`.
pub fn account_id(compressed_pubkey: &[u8]) -> [u8; 20] {
    let sha = Sha256::digest(compressed_pubkey);
    let rip = Ripemd160::digest(sha);
    let mut out = [0u8; 20];
    out.copy_from_slice(&rip);
    out
}

/// The bech32 address for a compressed pubkey under a chain prefix (HRP).
///
/// Rejects anything that is not a 33-byte SEC1-compressed key (wrong length,
/// or a leading byte other than `0x02`/`0x03`) instead of hashing it anyway:
/// a truncated or uncompressed key would otherwise silently derive a
/// well-formed but wrong address, not an error.
pub fn address(compressed_pubkey: &[u8], hrp: &str) -> Result<String, Error> {
    if compressed_pubkey.len() != 33 || !matches!(compressed_pubkey[0], 0x02 | 0x03) {
        return Err(Error::Key(format!(
            "expected a 33-byte SEC1-compressed pubkey (0x02/0x03 prefix), got {} bytes",
            compressed_pubkey.len()
        )));
    }
    let id = account_id(compressed_pubkey);
    let hrp = bech32::Hrp::parse(hrp).map_err(|e| Error::Bech32(e.to_string()))?;
    bech32::encode::<bech32::Bech32>(hrp, &id).map_err(|e| Error::Bech32(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-answer vector: this compressed secp256k1 pubkey derives this cosmos
    // address. Anchors the whole sha256 → ripemd160 → bech32 pipeline. Cross-checked
    // against an independent from-scratch BIP-173 + ripemd160(sha256(pk))
    // computation (account id 7c2bb42a8be69791ec763e51f5a49bcd41e82237).
    const DOC_PUBKEY: &str = "02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c";
    const DOC_COSMOS_ADDR: &str = "cosmos10s4mg25tu6termrk8egltfyme4q7sg3her239u";

    #[test]
    fn address_matches_cosmos_sdk_known_vector() {
        let pk = hex::decode(DOC_PUBKEY).unwrap();
        assert_eq!(address(&pk, "cosmos").unwrap(), DOC_COSMOS_ADDR);
    }

    #[test]
    fn pussy_prefix_is_applied() {
        let pk = hex::decode(DOC_PUBKEY).unwrap();
        let addr = address(&pk, PUSSY).unwrap();
        assert!(addr.starts_with("pussy1"), "got {addr}");
    }

    #[test]
    fn same_key_different_chains_share_account_id() {
        // Only the HRP differs; the 20-byte id is chain-independent.
        let pk = hex::decode(DOC_PUBKEY).unwrap();
        let (_, pussy_data) = bech32::decode(&address(&pk, PUSSY).unwrap()).unwrap();
        let (_, bostrom_data) = bech32::decode(&address(&pk, BOSTROM).unwrap()).unwrap();
        assert_eq!(pussy_data, bostrom_data, "same account id under both prefixes");
    }

    #[test]
    fn rejects_uncompressed_key_instead_of_hashing_it() {
        // A 65-byte uncompressed SEC1 key (0x04 prefix) is not what this
        // function's contract promises; it must not silently derive an
        // address from the wrong bytes.
        let uncompressed = vec![0x04u8; 65];
        assert!(matches!(address(&uncompressed, PUSSY), Err(Error::Key(_))));
    }

    #[test]
    fn rejects_truncated_key_instead_of_hashing_it() {
        let pk = hex::decode(DOC_PUBKEY).unwrap();
        let truncated = &pk[..32];
        assert!(matches!(address(truncated, PUSSY), Err(Error::Key(_))));
    }

    #[test]
    fn rejects_wrong_prefix_byte_instead_of_hashing_it() {
        let mut pk = hex::decode(DOC_PUBKEY).unwrap();
        pk[0] = 0x00; // neither 0x02 nor 0x03
        assert!(matches!(address(&pk, PUSSY), Err(Error::Key(_))));
    }

    #[test]
    fn empty_key_is_rejected() {
        assert!(matches!(address(&[], PUSSY), Err(Error::Key(_))));
    }
}
