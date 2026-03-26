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
| [[seal]] | confidentiality | Module-RLWE (NIST FIPS 203) | [[seal]] |
| [[stealth]] | confidentiality | CSIDH (isogeny class group) | [[stealth]] |
| [[veil]] | confidentiality | LWE | [[veil]] |
| [[quorum]] | distribution | information-theoretic (SSS) + hash (VSS) | [[quorum]] |
| [[delay]] | delay | sequential computation (isogeny or class group) | [[delay]] |
| [[order]] | ordering | collision resistance (hemera) | [[order]] |
| [[place]] | position | speed-of-light bound + geometric consistency | [[place]] |

## two arithmetics

mudra operates over three arithmetics:

- **nebu** (F_p, Goldilocks) — seal, quorum, order, place
- **jali** (R_q, polynomial ring) — veil
- **genies** (F_q, isogeny) — stealth, delay

shared secrets from genies-based modules are hashed into Goldilocks via hemera. the foreign field is contained — it enters, hemera normalizes, nebu continues.

## dependency map

```
nebu (F_p)    jali (R_q)    genies (F_q)
  ↓              ↓              ↓
hemera (hash: commitments, key derivation, domain separation)
  ↓
mudra
├── seal        uses nebu (scalar KEM), hemera (key derivation)
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
