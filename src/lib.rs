// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! mudra — cryptographic primitives: confidentiality, distribution, delay, ordering, position.
//!
//! Most of mudra is post-quantum and signature-free by design: a neuron's
//! identity is `Hemera(secret)` and authentication is a zheng proof of hash
//! preimage. Those modules (seal/stealth/veil/quorum/delay/order/place) are
//! specified in `specs/` and not yet implemented.
//!
//! **Phase 1 — the legacy-key bridge.** To migrate an existing Cosmos-SDK
//! network (spacepussy first), we must let a holder prove control of the
//! secp256k1 key they already have and bind it to a native neuron. That is the
//! one place classical signatures are structurally required. This crate
//! currently implements that bridge:
//!
//! - [`seed`]  — BIP-39 mnemonic → seed → BIP-32/44 secp256k1 key (coin type 118)
//! - [`cosmos`] — compressed pubkey → `ripemd160(sha256(pk))` → bech32 address
//! - [`claim`] — ADR-036 sign/verify of a `legacy address → native neuron` binding
//!
//! Verification here is native (in-process). Phase 2 re-expresses the same
//! secp256k1 + sha256 + ripemd160 verification as a nox program producing a
//! zheng proof, making the binding trustless. The Phase-1 code is the reference
//! implementation Phase 2 must match.

pub mod claim;
pub mod cosmos;
pub mod proof;
pub mod seed;

pub use claim::Claim;

/// The secp256k1 signing key type used across the bridge. Sourced from bip32's
/// re-export so its `k256` version is locked to the HD-derivation crate's.
pub use bip32::secp256k1::ecdsa::SigningKey;

/// Errors from the legacy-key bridge.
#[derive(Debug)]
pub enum Error {
    /// The BIP-39 mnemonic could not be parsed.
    Mnemonic(String),
    /// HD derivation (BIP-32/44) failed for the given path.
    Derive(String),
    /// Address encoding failed (bad HRP or bech32 error).
    Bech32(String),
    /// A key or signature was malformed.
    Key(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Mnemonic(m) => write!(f, "invalid mnemonic: {m}"),
            Error::Derive(m) => write!(f, "HD derivation failed: {m}"),
            Error::Bech32(m) => write!(f, "address encoding failed: {m}"),
            Error::Key(m) => write!(f, "malformed key or signature: {m}"),
        }
    }
}

impl std::error::Error for Error {}
