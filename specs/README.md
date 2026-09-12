---
tags: mudra
crystal-type: entity
crystal-domain: crypto
---
# mudra reference

canonical specification for seven cryptographic primitives. each module proves a property. each module has its own security assumption. they share no cryptographic code with each other.

## modules

| module | proves | security assumption | spec |
|--------|--------|-------------------|------|
| [[seal]] | confidentiality | lattice KEM profile; FIPS 203 for standard ML-KEM | [[seal]] |
| [[stealth]] | confidentiality | CSIDH (isogeny class group) | [[stealth]] |
| [[veil]] | confidentiality | LWE | [[veil]] |
| [[quorum]] | distribution | information-theoretic (SSS) + hash (VSS) | [[quorum]] |
| [[delay]] | delay | sequential computation (isogeny or class group) | [[delay]] |
| [[order]] | ordering | collision resistance (hemera) | [[order]] |
| [[place]] | position | speed-of-light bound + geometric consistency | [[place]] |

## authority and recovery contracts

[identity](identity.md) binds proof-based authority to exact actions and
versioned policies. [private recovery](private-recovery.md) defines private
discovery, authenticated history coverage and recoverable wallet state.
these are composition contracts for the existing primitives.

private recovery protects values, ownership and selected records against
an untrusted service. its client-light design candidates live in
[the recovery-index proposal](props/private-recovery-index.md).

## arithmetic profiles

the native design uses three algebras:

- **nebu** (F_p, Goldilocks) — native seal candidate, quorum, order, place
- **jali** (R_q, polynomial ring) — veil
- **genies** (F_q, isogeny) — stealth, delay

shared secrets from genies-based modules enter the selected hemera/KDF profile.
standard ML-KEM retains its specified modulus, polynomial degree and hashes.
FHE/PIR profiles likewise declare their own arithmetic; native-field reuse is
an optimization to qualify, not a compatibility assumption.

## dependency map

```
nebu (F_p)    jali (R_q)    genies (F_q)
  ↓              ↓              ↓
hemera (hash: commitments, key derivation, domain separation)
  ↓
mudra
├── seal        selected KEM profile; native candidate uses nebu/jali
├── stealth     uses genies (group action), hemera (secret hashing)
├── veil        uses jali (R_q ciphertexts, bootstrapping), hemera (FHE-friendly hash)
├── quorum      uses nebu (Lagrange interpolation), hemera (VSS commitments)
│               uses seal or stealth (encrypted share distribution in DKG)
├── delay       uses genies (isogeny VDF), hemera (input hashing)
├── order       uses hemera (hash chain, merkle clock)
└── place       uses hemera (RTT commitment), delay (challenge timing)
```

## internal cross-dependencies

- quorum → seal or stealth (encrypted share delivery in DKG round 2)
- place → delay (VDF on challenge-response prevents pre-computation)
- order → delay (VDF time extraction for ordering tiebreak)

all other modules are independent.
