# mudra

post-quantum confidentiality for [[neurons]]. mudra (मुद्रा — seal/gesture in Sanskrit) is to a neuron's secrets what [[hemera]] is to a particle's identity: hemera proves what a file is; mudra hides what a neuron holds and shares it with whom the neuron chooses.

[[hemera]] answers: what exists, and how to verify it. [[neuron]] answers: who acts, and how that is proven. mudra answers: who may read, and how a secret reaches them.

## what lives here, and what left

crate `cyber-mudra` (lib `mudra`) — four modules, four assumptions, no shared crypto code:

| module | contract | status |
|---|---|---|
| `seal` | [[mudra/specs/seal|seal]] (KEM) | **implemented** — standard profile ML-KEM-768 (FIPS 203) over the `ml-kem` crate; profile-tagged envelopes, implicit rejection preserved, hemera-derived keys. native lattice profile: spec |
| `quorum` | [[mudra/specs/quorum|quorum]] (threshold) | **implemented** — Shamir over Goldilocks, `split` / `recover`, any k of n. VSS / DKG / threshold decryption: spec (the VSS as written is not executable — audit 2026-09-16) |
| `stealth` | [[mudra/specs/stealth|stealth]] (NIKE) | spec — waits on the genies (isogeny) algebra |
| `veil` | [[mudra/specs/veil|veil]] (FHE) | spec — waits on the jali (ring) algebra |

[private recovery](specs/private-recovery.md) composes them with BBG / inf / zheng so a wallet recovers without disclosing its records. `cargo test` runs the seal and quorum suites.

on 2026-10-07 the repo was cut to that one responsibility ([[soft3/roadmap/component-boundaries|component boundaries]]):

| moved | to | why |
|---|---|---|
| the implemented identity code — `NeuronId = Hemera(pubkey)`, NSIG1 statement authentication, domain-scoped keys, the legacy-key bridge (`cyber-mudra` crate) | [[neuron]] — crate `neuron-auth` | identity is a field of the neuron; its authority code lives with the subject |
| `identity.md` (proof-based authority), `neuron-auth.md` (NSIG1 envelope), `bridge.md`, `bridge-proof.md`, `neuron-measures.md` | `neuron/specs/` — `proof-authority.md`, `local-authority.md` § NSIG1, `bridge.md`, `bridge-proof.md`, `measures.md` | same |
| `delay.md` (VDF), `place.md` (position), the `order` module | [[foculus]] — `foculus/specs/delay.md`, `place.md`; ordering is `foculus/src/chain.rs` | time, order and position are reconciliation questions, not confidentiality |

## why no signatures or VRF

in the native design, [[zheng]] proves authority for an exact action: a neuron proves its ownership/policy relation in zero knowledge with the action, network, program and current policy bound into the statement ([[neuron/specs/proof-authority|proof-based authority]]). verifiable randomness likewise needs a specified relation, uniqueness and a pseudorandomness argument; proving a hash computation alone does not establish every VRF property.

## the separation

proofs ([[zheng]]) handle: authentication, integrity, randomness, metering.
mudra handles: confidentiality, key agreement, private computation, key distribution.

orthogonal concerns. proofs verify and charge; mudra hides and shares.

[[hemera]] · [[neuron]] · [[foculus]] · [[zheng]] · [[cybergraph]]
