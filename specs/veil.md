---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# veil — compute without decrypting

perform computation on encrypted data without ever seeing the plaintext. fully homomorphic encryption (TFHE) with ciphertext modulus q = Goldilocks prime (p = 2^64 - 2^32 + 1) — same field as the entire stack. native proving through zheng, zero non-native arithmetic penalty.

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
  n ≥ 1024                       polynomial degree (128-bit LWE security at this modulus)
  two-adicity: 32                supports n up to 2^32
```

polynomial multiplication via native nebu NTT:

```
polynomial multiply in R_q:
  NTT(a)          → n F_p multiplications (native nebu)
  pointwise(â,b̂) → n F_p multiplications
  INTT(ĉ)        → n F_p multiplications

total: 3n native Goldilocks multiplications
proof: 3n degree-2 F_p constraints (no non-native arithmetic)
```

## why q = Goldilocks

| property | q = 2^32 | q = Goldilocks |
|----------|---------|---------------|
| R_q polynomial multiply | non-native in F_p proofs | native nebu NTT |
| proof overhead per multiply | ~10-20 extra F_p constraints (carries) | 0 extra constraints |
| NTT domain | 2-adic roots mod 2^32 | 2-adic roots mod p (two-adicity 32) |
| modular reduction | bit mask (free on CPU) | sparse reduction (~4 cycles) |
| proving bootstrapping | catastrophic overhead | native |

## hemera compatibility

hemera's x^-1 S-box reduces multiplicative depth 5.4x:

```
x^7 S-box (64 rounds): 192 multiplicative depth
x^-1 S-box (16 rounds): 40 multiplicative depth

FHE noise ∝ multiplicative depth
hemera under FHE: 5.4× less noise → fewer bootstraps
```

computing hemera homomorphically becomes practical. hash and FHE share the same field — zero conversion overhead.

## parameter constraints

- security: n ≥ 1024 for 128-bit LWE security with modulus p ≈ 2^64 (lattice estimator)
- noise: discrete Gaussian σ chosen for correctness at target depth. hemera x^-1 relaxes σ
- key switching: decomposition base B divides p - 1 = 2^32 × (2^32 - 1). highly composite — many valid bases
- bootstrapping: test polynomial evaluates at n-th roots of unity mod p. these roots exist (n | 2^32, 2^32 | p-1)

## programmable bootstrapping

evaluates an arbitrary lookup table during noise reduction. the test polynomial maps n-th roots of unity to LUT entries. combined with native Goldilocks NTT, programmable bootstrapping enables general computation under encryption at native proving cost.

## usage in cyber

- private cyberlink computation: evaluate tri-kernel updates on encrypted cyberlinks
- private neuron inference: run nox programs on encrypted inputs, produce encrypted outputs
- FHE-friendly hashing: compute hemera on encrypted data (practical with x^-1 depth reduction)

## dependencies

- nebu: Goldilocks field arithmetic, NTT (polynomial multiplication in R_q)
- hemera: FHE-friendly hash (x^-1 S-box, 40 multiplicative depth)
- zheng: native proving of FHE operations (zero non-native overhead)
