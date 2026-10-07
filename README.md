# mudra

post-quantum confidentiality for [[neurons]]. mudra (मुद्रा — seal/gesture in Sanskrit) is to a neuron's secrets what [[hemera]] is to a particle's identity: hemera proves what a file is; mudra hides what a neuron holds and shares it with whom the neuron chooses.

[[hemera]] answers: what exists, and how to verify it. [[neuron]] answers: who acts, and how that is proven. mudra answers: who may read, and how a secret reaches them.

## what lives here, and what left

mudra is specification only. the four module contracts in [specs](specs/README.md) — [[mudra/specs/seal|seal]] (KEM), [[mudra/specs/stealth|stealth]] (NIKE), [[mudra/specs/veil|veil]] (FHE), [[mudra/specs/quorum|quorum]] (threshold) — are written and not yet implemented; [private recovery](specs/private-recovery.md) composes them with BBG / inf / zheng so a wallet recovers without disclosing its records.

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
