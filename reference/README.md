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
| [[delay]] | delay | sequential squaring (unknown group order) | [[delay]] |
| [[order]] | ordering | collision resistance (hemera) | [[order]] |
| [[place]] | position | speed-of-light bound + geometric consistency | [[place]] |

## algebra

all modules operate over Goldilocks field (p = 2^64 - 2^32 + 1) where possible. same field as nebu, hemera, nox, zheng, bbg. stealth is the exception — it operates over supersingular elliptic curves, though shared secrets are hashed into Goldilocks via hemera.

## dependency map

```
nebu (field arithmetic)
  ↓
hemera (hash: commitments, key derivation, domain separation)
  ↓
mudra
├── seal        uses nebu (NTT for R_q), hemera (key derivation)
├── stealth     uses nebu (curve arithmetic), hemera (secret hashing)
├── veil        uses nebu (NTT for R_q), hemera (FHE-friendly hash)
├── quorum      uses nebu (Lagrange interpolation), hemera (VSS commitments)
│               uses seal or stealth (encrypted share distribution in DKG)
├── delay       uses hemera (input hashing), optionally quorum (group parameter DKG)
├── order       uses hemera (hash chain, merkle clock)
└── place       uses hemera (RTT commitment), delay (challenge timing)
```

## internal cross-dependencies

- quorum → seal or stealth (encrypted share delivery in DKG round 2)
- place → delay (VDF on challenge-response prevents pre-computation)
- order → delay (VDF time extraction for ordering tiebreak)

all other modules are independent.
