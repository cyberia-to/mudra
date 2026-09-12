---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# veil — compute without decrypting

perform computation on encrypted data without seeing the plaintext. each FHE
profile fixes its scheme, encoding, arithmetic, parameters and evaluation keys.
the native-field candidate targets TFHE-style computation with Goldilocks
ciphertext arithmetic; its security and correctness require separate
qualification.

BFV/BGV, TFHE and private-retrieval backends have distinct ciphertext/key
formats and noise models. sharing lattice assumptions does not make them
interchangeable. [private recovery](private-recovery.md) selects a complete
retrieval profile and proves the actual encrypted operations it uses.

## interface

```
keygen() → (SecretKey, PublicKey)
encrypt(pk: &PublicKey, message: bool) → Ciphertext
decrypt(sk: &SecretKey, ct: &Ciphertext) → bool

// homomorphic operations
add(ct1: &Ciphertext, ct2: &Ciphertext) → Ciphertext
mul(ct1: &Ciphertext, ct2: &Ciphertext) → Ciphertext

// noise management
bootstrap(ct: &Ciphertext, bsk: &BootstrapKey) → Ciphertext
programmable_bootstrap(ct: &Ciphertext, lut: &[u64], bsk: &BootstrapKey) → Ciphertext
```

## algebra

```
Ring R_q = F_p[x] / (x^n + 1)
  q = p = 2^64 - 2^32 + 1       Goldilocks prime
  n = profile-selected power of two
  negacyclic NTT requires 2n | (p - 1), hence n ≤ 2^31
```

polynomial multiplication via native nebu NTT:

```
polynomial multiply in R_q:
  NTT(a), NTT(b)  → O(n log n) field operations
  pointwise(â,b̂) → O(n) field operations
  INTT(ĉ)        → O(n log n) field operations

exact operations, preprocessing reuse and proof costs depend on the backend
```

## why q = Goldilocks

the candidate reuses the proof system's scalar field for suitable ring
arithmetic and NTTs. packing, rotations, key switching, rounding and bootstrap
correctness still require their own operations and proof relations. the
selected backend determines whether that field choice improves total cost.

standard or independently qualified FHE profiles retain their own parameters;
they are not rewritten into Goldilocks solely to reuse arithmetic.

## hemera compatibility

the profile specifies the exact Hemera permutation, zero convention and
encrypted evaluation algorithm. a low-degree proof constraint for an inverse
does not supply a low-depth homomorphic inverse operation. encrypted evaluation
may require exponentiation, a lookup or bootstrapping, each with its own cost.

sharing a field can reduce conversion overhead. multiplicative depth, noise,
bootstraps and proving work must be established for the full selected circuit.
this specification assigns no unmeasured Hemera/FHE speedup.

## parameter constraints

- security: fix dimension, secret/error distributions, modulus and evaluation-key assumptions; evaluate the complete classical/quantum attack model
- noise: specify decoding and failure bounds over the complete evaluation and lifetime scope
- key switching: specify decomposition, rounding, error growth and any circular/key-dependent-message assumption
- bootstrapping: specify the actual scheme's lookup encoding, keys and correctness relation; NTT roots alone do not define a bootstrap

## programmable bootstrapping

programmable bootstrapping evaluates a profile-defined function during noise
refresh. the scheme must define its plaintext domain, lookup representation,
precision and failure probability. native NTT arithmetic is an implementation
building block; it does not by itself establish bootstrap correctness or cost.

## usage in cyber

- private cyberlink computation: evaluate tri-kernel updates on encrypted cyberlinks
- private neuron inference: run nox programs on encrypted inputs, produce encrypted outputs
- encrypted detection and compaction for a separately qualified private-recovery profile
- hashing encrypted data with a qualified Hemera evaluation circuit

## dependencies

- nebu: Goldilocks field arithmetic, NTT (polynomial multiplication in R_q)
- hemera: profile-selected hash and domain separation
- zheng: proving the actual FHE program, including profile-specific arithmetic and conversions
