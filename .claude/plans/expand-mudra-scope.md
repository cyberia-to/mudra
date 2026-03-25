---
status: draft
created: 2026-03-25
---
# expand mudra scope: clock, vdf, location

## problem

bbg reference specifies algorithms that have no implementation home:
- VDF (verifiable delay function) — rate-limits signal creation
- hash chain — per-neuron prev links, equivocation detection
- merkle clock — compact causal DAG commitment
- step counter — monotonic logical clock per neuron
- deterministic tiebreak — H(A) < H(B) for concurrent signals
- location proof — RTT mesh + classical MDS → 3D coordinates

these are all cryptographic primitives that prove properties:
- vdf proves delay (physical time without clocks)
- clock proves ordering (causal, logical, physical)
- location proves position (geometric consistency)

they belong in mudra alongside existing modules (kem, ctidh, tfhe, threshold).

## changes

### 1. CLAUDE.md — update modules section

add three new modules to the modules list and security boundaries table:
- vdf/ — verifiable delay function (sequential squaring assumption)
- clock/ — hash chain + merkle clock + step counter + tiebreak
- location/ — RTT + MDS location proof (speed-of-light bound)

### 2. Cargo.toml — update description

from: "post-quantum cryptographic primitives: KEM, dCTIDH, TFHE, threshold"
to:   "cryptographic primitives: KEM, dCTIDH, TFHE, threshold, VDF, clock, location"

### 3. README.md — update

expand single-line description to reflect full scope.

### 4. reference/ — create specs for new modules

- reference/vdf.md — VDF specification (from bbg/reference/sync.md)
- reference/clock.md — hash chain + merkle clock + step + tiebreak
- reference/location.md — RTT + MDS location proof

### 5. src/lib.rs — declare modules

add module declarations (empty for now, spec before code).

## what does NOT change

- existing module specs (kem, ctidh, tfhe, threshold) — untouched
- Cargo.toml dependencies — no new deps needed
- security boundary principle — each module independent
