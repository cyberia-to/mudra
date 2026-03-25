---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# delay — prove that time passed

proves minimum wall-clock time elapsed between two events. inherently sequential — cannot be parallelized. physical time without clocks, without NTP, without trusted timestamps. verifiable delay function (VDF).

## interface

```
vdf_prove(input: H, T: u64) → VDFProof
vdf_verify(input: H, T: u64, proof: VDFProof) → bool
```

- `input`: hemera hash of previous signal (or any commitment)
- `T`: minimum sequential steps (difficulty parameter)
- `proof`: the output plus auxiliary data for fast verification

## mechanism

repeated squaring in a group where the order is unknown to the prover:

```
y = x^(2^T) mod N
```

computing y requires T sequential squarings. verifying y requires O(log T) operations with Wesolowski or Pietrzak proof.

for Goldilocks field: the VDF operates over a class group or RSA group — the field itself has known order, so repeated squaring in F_p alone is not a VDF. two candidate constructions:

**option A: class group VDF (transparent)**
- group: class group of imaginary quadratic field
- no trusted setup (group order unknown by construction)
- slower evaluation, but transparent
- aligned with cyber's no-trusted-setup philosophy

**option B: RSA-based VDF (with setup)**
- group: Z/NZ where N = pq (RSA modulus)
- requires trusted setup or MPC for N generation
- faster evaluation
- trusted setup conflicts with cyber's design

**option C: isogeny-based VDF (via genies)**
- group: commutative group action on supersingular curves (same F_q as stealth)
- no trusted setup (group order unknown by construction)
- shares algebra with stealth — one field for both privacy and time proofs
- isogeny walk is inherently sequential (cannot parallelize endomorphism computation)

recommendation: option C (genies) or option A (class group). both transparent, no trusted setup. option C reuses genies infrastructure already needed for stealth.

## properties

| property | value |
|----------|-------|
| sequentiality | T sequential squarings, cannot parallelize |
| verification | O(log T) group operations |
| proof size | O(1) group elements (~256 bytes) |
| security | unknown group order assumption |

## usage in cyber

**signal rate limiting:**
```
signal.vdf_proof = vdf_prove(H(prev_signal), T_min)
```
each signal requires T_min wall-clock time since its predecessor. flooding N signals costs N × T_min sequential time.

**equivocation detection:**
two signals with same `prev` require computing VDF twice from the same input. total VDF time exceeds elapsed time → detectable.

**physical time ordering:**
between causally independent signals, VDF proof timestamps create a partial physical time ordering without any clock synchronization.

## parameters

T_min is per-neuron configurable:
- phone: longer T_min (slower hardware, less signals per unit time)
- workstation: shorter T_min (faster hardware)
- the VDF adapts to the hardware — honest nodes set T_min proportional to their computational capacity

## dependencies

- genies: F_q group action (isogeny-based VDF) or class group arithmetic
- hemera: input hashing (H(prev_signal))
- nebu: Goldilocks field arithmetic (proof verification folds into zheng accumulator)
- mudra::quorum: optional DKG for group parameter generation

## open questions

- class group vs RSA group (recommendation: class group for transparency)
- exact T_min calibration methodology
- VDF ASIC resistance considerations
- integration with zheng constraints (proving VDF verification inside a circuit)
