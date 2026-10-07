//! seal — encrypt for a recipient (`specs/seal.md`).
//!
//! Standard profile: ML-KEM-768 (FIPS 203) through the `ml-kem` crate.
//! Envelopes carry a one-byte profile tag so keys and ciphertexts identify
//! what they are; decapsulation keeps ML-KEM's implicit rejection (an invalid
//! ciphertext yields a pseudorandom secret, never an error that would act as
//! a decryption oracle). The shared secret enters hemera for domain-separated
//! key derivation, as the spec requires for every profile.
//!
//! The native Goldilocks/jali lattice profile in the spec is not built.

use hemera::Hash;
use ml_kem::{Decapsulate, Encapsulate, Kem, Key, KeyExport, MlKem768, Seed};
use rand_core::CryptoRng;

/// Profile tag written as the first envelope byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Profile {
    /// FIPS 203 ML-KEM-768: ek 1184 B, dk seed 64 B, ct 1088 B, ss 32 B.
    MlKem768 = 0x03,
}

impl Profile {
    pub fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x03 => Some(Profile::MlKem768),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SealError {
    /// Empty envelope or unknown profile byte.
    UnknownProfile,
    /// Envelope length does not match the profile.
    BadLength { expected: usize, got: usize },
    /// Key bytes failed the profile's validation (e.g. non-canonical ek).
    InvalidKey,
}

type Ek = ml_kem::EncapsulationKey768;
type Dk = ml_kem::DecapsulationKey768;

/// A recipient's public encapsulation key, with its profile.
#[derive(Clone)]
pub struct PublicKey {
    profile: Profile,
    key: Ek,
}

/// The recipient's secret decapsulation key.
pub struct SecretKey {
    profile: Profile,
    key: Dk,
}

/// A sealed secret for one recipient.
#[derive(Clone, PartialEq, Eq)]
pub struct Ciphertext {
    profile: Profile,
    bytes: Vec<u8>,
}

/// The 32-byte shared secret, never used raw: derive keys from it.
#[derive(Clone, PartialEq, Eq)]
pub struct SharedSecret([u8; 32]);

impl core::fmt::Debug for SharedSecret {
    /// Never prints the secret.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SharedSecret([redacted; 32])")
    }
}

const EK_LEN: usize = 1184;
/// the decapsulation key travels in its 64-byte seed form (FIPS 203 §7.1 (d ‖ z))
const DK_LEN: usize = 64;
const CT_LEN: usize = 1088;

/// Generate a fresh keypair under the standard profile.
pub fn keygen(rng: &mut impl CryptoRng) -> (SecretKey, PublicKey) {
    let (dk, ek) = MlKem768::generate_keypair_from_rng(rng);
    (
        SecretKey {
            profile: Profile::MlKem768,
            key: dk,
        },
        PublicKey {
            profile: Profile::MlKem768,
            key: ek,
        },
    )
}

/// Encapsulate a fresh shared secret for `pk`.
pub fn encap(pk: &PublicKey, rng: &mut impl CryptoRng) -> (Ciphertext, SharedSecret) {
    let (ct, ss) = pk.key.encapsulate_with_rng(rng);
    (
        Ciphertext {
            profile: pk.profile,
            bytes: ct.as_slice().to_vec(),
        },
        SharedSecret(ss.into()),
    )
}

/// Decapsulate. A ciphertext of the wrong profile or length is a hard error;
/// a well-formed but tampered ciphertext decapsulates to a pseudorandom secret
/// (FIPS 203 implicit rejection) — the caller learns nothing about validity.
pub fn decap(sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, SealError> {
    if ct.profile != sk.profile {
        return Err(SealError::UnknownProfile);
    }
    if ct.bytes.len() != CT_LEN {
        return Err(SealError::BadLength {
            expected: CT_LEN,
            got: ct.bytes.len(),
        });
    }
    let ss = sk
        .key
        .decapsulate_slice(&ct.bytes)
        .map_err(|_| SealError::BadLength {
            expected: CT_LEN,
            got: ct.bytes.len(),
        })?;
    Ok(SharedSecret(ss.into()))
}

impl SharedSecret {
    /// Domain-separated key derivation: hemera over `mudra:seal:v1 ‖ ss ‖ context`.
    /// Different contexts (payload cipher, envelope MAC, next epoch) get
    /// independent keys from one encapsulation.
    pub fn derive(&self, context: &[u8]) -> [u8; 32] {
        let mut input = Vec::with_capacity(13 + 32 + context.len());
        input.extend_from_slice(b"mudra:seal:v1");
        input.extend_from_slice(&self.0);
        input.extend_from_slice(context);
        let h: Hash = hemera::hash(&input);
        *h.as_bytes()
    }

    /// The raw secret, for tests and for profiles that specify their own KDF.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl PublicKey {
    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// Envelope: profile tag ‖ encoded key (FIPS 203 ek encoding).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + EK_LEN);
        out.push(self.profile as u8);
        out.extend_from_slice(self.key.to_bytes().as_slice());
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SealError> {
        let (profile, body) = split_envelope(bytes, EK_LEN)?;
        let enc = Key::<Ek>::try_from(body).map_err(|_| SealError::InvalidKey)?;
        let key = Ek::new(&enc).map_err(|_| SealError::InvalidKey)?;
        Ok(Self { profile, key })
    }
}

impl SecretKey {
    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + DK_LEN);
        out.push(self.profile as u8);
        out.extend_from_slice(self.key.to_bytes().as_slice());
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SealError> {
        let (profile, body) = split_envelope(bytes, DK_LEN)?;
        let seed = Seed::try_from(body).map_err(|_| SealError::InvalidKey)?;
        Ok(Self {
            profile,
            key: Dk::from_seed(seed),
        })
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            profile: self.profile,
            key: self.key.encapsulation_key().clone(),
        }
    }
}

impl Ciphertext {
    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.bytes.len());
        out.push(self.profile as u8);
        out.extend_from_slice(&self.bytes);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SealError> {
        let (profile, body) = split_envelope(bytes, CT_LEN)?;
        Ok(Self {
            profile,
            bytes: body.to_vec(),
        })
    }
}

fn split_envelope(bytes: &[u8], body_len: usize) -> Result<(Profile, &[u8]), SealError> {
    let (&tag, body) = bytes.split_first().ok_or(SealError::UnknownProfile)?;
    let profile = Profile::from_tag(tag).ok_or(SealError::UnknownProfile)?;
    if body.len() != body_len {
        return Err(SealError::BadLength {
            expected: body_len,
            got: body.len(),
        });
    }
    Ok((profile, body))
}
