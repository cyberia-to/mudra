# mudra

post-quantum cryptographic primitives for [[neurons]]. mudra (मुद्रा — seal/gesture in Sanskrit) is to [[neurons]] what [[hemera]] is to [[particles]]: hemera gives content its identity and integrity (hashing, commitment, tree proofs); mudra gives agents their confidentiality and privacy (encrypting, exchanging keys, computing privately, distributing keys).

[[hemera]] answers: what exists, and how to verify it. mudra answers: who acts, and how to protect them.

## why no signatures or VRF

in the native design, [[zheng]] proves authority for an exact action. a neuron
proves its ownership/policy relation in zero knowledge, with the action,
network, program and current policy bound into the statement. hashes and the
proof construction have distinct security requirements. the full contract is
[identity](specs/identity.md).

verifiable randomness likewise requires a specified relation, uniqueness and
pseudorandomness argument. proving a hash computation alone does not establish
every VRF property.

proofs let an application combine programmable authority and state-transition
rules in one verified statement. the execution protocol separately defines
metering, fees and any verifiable-randomness relation.

the [[call]] mechanism in [[nox]] binds a neuron's private authority to the
message and requested action, both on-chain and off-chain. delivery receipts
require an explicit acknowledgement relation; authorization or correct
execution alone cannot establish delivery.

## the separation

proofs ([[zheng]]) handle: authentication, integrity, randomness, metering.
mudra handles: confidentiality, key agreement, private computation, key distribution.

these are orthogonal concerns. proofs verify and charge; mudra hides and shares.

## modules

the [seven module contracts](specs/README.md) separate authority, encryption,
agreement and private computation. [private recovery](specs/private-recovery.md)
composes those primitives with BBG/Inf/Zheng so a wallet can recover without
disclosing its selected records to the service. the
[client-light recovery proposal](specs/props/private-recovery-index.md) develops
compact discovery and continuously maintained encrypted wallet state.
