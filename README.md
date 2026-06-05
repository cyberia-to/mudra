# mudra

post-quantum cryptographic primitives for [[neurons]]. mudra (मुद्रा — seal/gesture in Sanskrit) is to [[neurons]] what [[hemera]] is to [[particles]]: hemera gives content its identity and integrity (hashing, commitment, tree proofs); mudra gives agents their confidentiality and privacy (encrypting, exchanging keys, computing privately, distributing keys).

[[hemera]] answers: what exists, and how to verify it. mudra answers: who acts, and how to protect them.

## why no signatures or VRF

in a proof-native system, [[stark]] proofs replace both. a neuron proves `H(secret) = address` in zero knowledge — this IS a signature, just a more powerful one. every digital signature is a special case of a zero-knowledge proof of knowledge. similarly, a VRF computes `output = H(secret, input)` and proves correctness — the proof system handles this directly.

what proofs provide that signatures cannot: composability (prove arbitrary statements, not just key ownership), chargeability (every proof is metered), and universality (one mechanism for authentication, integrity, randomness, and metering).

the [[call]] mechanism in [[nox]] (pattern 16) makes this concrete: a neuron proves knowledge of its secret key without revealing it, both on-chain and off-chain. every message is proved and charged for — proof of delivery replaces signed delivery.

## the separation

proofs ([[zheng]]) handle: authentication, integrity, randomness, metering.
mudra handles: confidentiality, key agreement, private computation, key distribution.

these are orthogonal concerns. proofs verify and charge; mudra hides and shares.

## modules
