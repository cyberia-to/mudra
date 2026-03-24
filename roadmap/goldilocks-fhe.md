---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: draft
---
# goldilocks FHE parameters

mudra's FHE scheme must use q = Goldilocks prime (p = 2⁶⁴ − 2³² + 1) as the ciphertext modulus. this single decision determines whether FHE operations prove natively or suffer 10-20× non-native arithmetic penalty.

## the choice

TFHE typically uses q = 2³² or q = 2⁶⁴ for efficient modular reduction via bit masking. choosing q = Goldilocks instead:

| property | q = 2³² | q = Goldilocks |
|---|---|---|
| R_q polynomial multiply | non-native in F_p proofs | **native nebu NTT** |
| proof overhead per multiply | ~10-20 extra F_p constraints (carries) | **0 extra constraints** |
| NTT domain | 2-adic roots mod 2³² | **2-adic roots mod p (two-adicity 32)** |
| max polynomial degree | 2³² | **2³²** (same) |
| modular reduction | bit mask (free on CPU) | sparse reduction (2⁶⁴ ≡ 2³² − 1, ~4 cycles) |
| proving bootstrapping | catastrophic overhead | **native** |

## R_q = F_p[x]/(x^n + 1)

with q = p, the polynomial ring R_q is a quotient of the polynomial ring over Goldilocks. all ring operations decompose into nebu field operations via NTT:

```
polynomial multiply in R_q:
  1. NTT(a)         → n F_p multiplications (nebu)
  2. pointwise(â,b̂) → n F_p multiplications (nebu)
  3. INTT(ĉ)        → n F_p multiplications (nebu)

total: 3n nebu multiplications
proof: 3n degree-2 F_p constraints
no non-native arithmetic, no carry propagation, no range checks
```

nebu's NTT uses primitive 2³²-th root of unity in Goldilocks. supports n up to 2³² — vastly exceeding any FHE parameter set.

## hemera-2 compatibility

hemera-2's x⁻¹ S-box reduces multiplicative depth by 5.4× compared to hemera-1 (x⁷):

```
hemera-1: 64 partial rounds × 3 sequential muls = 192 multiplicative depth
hemera-2: 16 partial rounds × ~2.5 sequential muls = 40 multiplicative depth

FHE noise ∝ multiplicative depth
hemera under FHE: 5.4× less noise → fewer bootstraps needed
```

computing hemera homomorphically (for FHE-friendly hashing) becomes practical with hemera-2 + Goldilocks parameters. the hash and the FHE scheme share the same field — zero conversion overhead.

## parameter constraints

choosing q = Goldilocks constrains TFHE parameters:

- **security level**: n (polynomial degree) must be chosen for 128-bit security against LWE with modulus p ≈ 2⁶⁴. standard lattice estimator gives n ≥ 1024 for this modulus size
- **noise distribution**: discrete Gaussian with σ chosen for correctness at depth D. hemera-2's reduced depth (40 vs 192) relaxes σ requirements
- **key switching**: decomposition base B must divide p − 1. Goldilocks: p − 1 = 2³² × (2³² − 1), highly composite — many valid bases
- **bootstrapping modulus**: for programmable bootstrapping, the test polynomial evaluates at n-th roots of unity mod p. these exist (n | 2³², and 2³² | p−1)

## what this enables

with q = Goldilocks, the Wav language (R_q convolution) and all FHE operations in mudra prove natively through zheng/WHIR. no non-native arithmetic. the proving cost of FHE bootstrapping drops from "catastrophic" to "proportional to computation" — same cost model as any other nox program.

the ring-aware jet library in Wav (ntt_batch, key_switch, gadget_decomp, noise_track) operates over native Goldilocks NTT. no field conversion at any point in the pipeline.

see [[hemera-2]] for hash depth reduction, [[ring-aware-fhe]] for proving optimization, [[Wav]] for the convolution language
