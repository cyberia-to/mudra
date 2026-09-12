---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# seal — encrypt for a recipient

seal provides lattice key encapsulation for recipient encryption. a sender
encapsulates a fresh shared secret under an authenticated recipient public key.
the recipient can be offline. authenticated payload encryption is a separate
operation using keys derived from that secret.

## interface

```text
keygen(profile, randomness) → (SecretKey, PublicKey)
encap(profile, pk, randomness) → (Ciphertext, SharedSecret)
decap(profile, sk, ct) → profile-defined result
```

the profile defines key/ciphertext encoding, validation, decapsulation behavior
and failure handling. standard ML-KEM includes implicit rejection for invalid
ciphertexts; an API must preserve its specified behavior. application errors
must not introduce a ciphertext-validity or decryption oracle.

keys and ciphertext envelopes identify their profile/version. a peer-key
binding identifies the recipient, network, purpose and key epoch. payload
encryption authenticates the envelope and output/operation association.

## standard ML-KEM profiles

FIPS 203 ML-KEM uses modulus 3329, polynomial degree 256 and module ranks
2, 3 or 4. standard profiles preserve the prescribed algorithms, distributions,
compression, hashes and chosen-ciphertext transform.

| profile | NIST category | public key | ciphertext | shared secret |
|---|---|---|---|---|
| ML-KEM-512 | 1 | 800 B | 768 B | 32 B |
| ML-KEM-768 | 3 | 1184 B | 1088 B | 32 B |
| ML-KEM-1024 | 5 | 1568 B | 1568 B | 32 B |

[FIPS 203](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf)
defines these profiles. no system-wide quantum-security bit count follows from
the category of one primitive. the enclosing Cyber protocol explicitly selects
its profile; this module assigns no new default.

## native-field construction

the earlier Goldilocks sketch, modulus 2^64 - 2^32 + 1, degree 64 and rank 4,
is a separate candidate construction. those choices do not instantiate FIPS 203.

a native profile must separately specify distributions, noise/error bounds,
message encoding, seed expansion, compression, validation, chosen-ciphertext
security, decapsulation behavior and parameter estimates. a noisy public-key
encryption equation alone is insufficient to define a secure KEM.

the native profile remains unselected until those requirements are met.
standard ML-KEM sizes and security categories cannot be assigned to it.
[Recorded discrepancy](../audit/signature-optimality/mudra-design.md#seal-a-consequential-parameter-ambiguity).

## relationship to stealth

| property | seal | stealth |
|---|---|---|
| recipient can be offline | yes, with a published authentic key | yes, with a published authentic key |
| obtaining a pairwise secret | sender encapsulates; recipient decapsulates | each applies its secret to the peer key |
| additional delivery material | KEM ciphertext | per-payment stealth publishes an ephemeral public key |
| static pairwise agreement | requires an encapsulation | two existing keys suffice |
| security/size/cost | selected KEM profile | selected class-group-action profile |

public-key publication is a prerequisite for both. comparative performance
requires the same hardware and separately identified operations/profiles.

## usage and dependencies

seal supplies private particle delivery, encrypted operation parameters and
share delivery for quorum. shared-secret possession is distinct from
[proof-based authority](identity.md).

arithmetic and hashing follow the selected profile. standard ML-KEM uses its
standard ring and hashes. a native candidate may use nebu/jali and hemera after
separate qualification; field reuse does not establish security or wire
compatibility.
