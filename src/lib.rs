// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! mudra — post-quantum confidentiality for neurons.
//!
//! [`hemera`] proves what a particle is; [`neuron-auth`](https://github.com/cyberia-to/neuron)
//! proves who acts; mudra hides what a neuron holds and shares it with whom
//! the neuron chooses. Four modules, four assumptions, no shared crypto code.
//!
//! | module | spec | status |
//! |---|---|---|
//! | [`seal`] | `specs/seal.md` | **implemented** — standard profile ML-KEM-768 (FIPS 203) via the `ml-kem` crate, envelopes tagged by profile, hemera-derived keys. The native Goldilocks lattice profile is not built. |
//! | [`quorum`] | `specs/quorum.md` | **implemented** — Shamir secret sharing over Goldilocks (`split` / `recover`). Verifiable sharing, DKG and threshold decryption are not built: the 2026-09-16 audit found the VSS as specified is not executable as written. |
//! | [`stealth`] | `specs/stealth.md` | spec only — needs the genies (isogeny) algebra, which does not exist yet |
//! | [`veil`] | `specs/veil.md` | spec only — needs the jali (ring) algebra, which does not exist yet |
//!
//! The identity code this crate carried until 2026-10-07 (native neuron ids,
//! NSIG1, the legacy-key bridge) is `neuron-auth` in the neuron repository.

pub mod quorum;
pub mod seal;
pub mod stealth;
pub mod veil;
