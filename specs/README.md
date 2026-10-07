---
tags: mudra
crystal-type: entity
crystal-domain: crypto
---
# mudra reference

canonical specification for four confidentiality primitives. each module proves a property, each has its own security assumption, they share no cryptographic code. none is implemented yet — the implemented code mudra used to carry (identity, the legacy-key bridge) is now [[neuron]]'s `neuron-auth` crate; time and position moved to [[foculus]] (see the [README](../README.md)).

## modules

| module | proves | security assumption | spec |
|--------|--------|-------------------|------|
| [[seal]] | confidentiality | lattice KEM profile; FIPS 203 for standard ML-KEM | [[seal]] |
| [[stealth]] | confidentiality | CSIDH (isogeny class group) | [[stealth]] |
| [[veil]] | confidentiality | LWE | [[veil]] |
| [[quorum]] | distribution | information-theoretic (SSS) + hash (VSS) | [[quorum]] |

## composition contracts

[private recovery](private-recovery.md) defines private discovery, authenticated history coverage and recoverable wallet state over the four primitives. its client-light design candidates live in [the recovery-index proposal](props/private-recovery-index.md). authority — who may act — is not mudra's: [[neuron/specs/proof-authority|proof-based authority]] and [[neuron/specs/local-authority|local authority]] in neuron.

## arithmetic profiles

the native design uses three algebras:

- **nebu** ($\mathbb{F}_p$, Goldilocks) — native seal candidate, quorum
- **jali** ($R_q$, polynomial ring) — veil
- **genies** ($\mathbb{F}_q$, isogeny) — stealth

shared secrets from genies-based modules enter the selected hemera/KDF profile. standard ML-KEM retains its specified modulus, polynomial degree and hashes. FHE/PIR profiles likewise declare their own arithmetic; native-field reuse is an optimisation to qualify, not a compatibility assumption.

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
└── quorum      uses nebu (Lagrange interpolation), hemera (VSS commitments)
                uses seal or stealth (encrypted share distribution in DKG)
```

the one internal cross-dependency: quorum → seal or stealth (encrypted share delivery in DKG round 2). [[foculus/specs/delay|delay]] (foculus) may use quorum for DKG of group parameters; that is foculus's dependency on mudra, not the reverse.
