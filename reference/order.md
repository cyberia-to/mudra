---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# order — establish event sequence

four mechanisms that establish temporal structure without consensus. each proves a different ordering property. together they produce a deterministic total order over any signal DAG.

## hash chain

per-neuron sequential history. immutable, verifiable, fork-evident.

```
interface:
  chain_append(prev_hash: H, signal: Signal) → H
  chain_verify(chain: [H]) → bool
  chain_detect_equivocation(s1: Signal, s2: Signal) → Option<EquivocationProof>
```

each neuron chains signals via the `prev` field:

```
s1 ← s3 ← s5 ← s8
  prev(s3) = H(s1)
  prev(s5) = H(s3)
  prev(s8) = H(s5)
```

properties:
- immutable: cannot insert, remove, or reorder past signals
- verifiable: any peer walks the chain, checks H(s_n) == s_{n+1}.prev
- fork-evident: two signals with same prev = cryptographic equivocation proof
- O(1) append, O(1) equivocation detection, O(n) full chain verification

dependencies: hemera (H)

## merkle clock

compact causal DAG commitment. replaces vector clocks (O(neurons)) with O(1) comparison.

```
interface:
  mc_create(seen_signals: DAG) → MerkleClock
  mc_compare(a: MerkleClock, b: MerkleClock) → Ordering  // Equal | Divergent
  mc_divergence(a: DAG, b: DAG) → DivergencePoint
  mc_merge(a: DAG, b: DAG) → MerkleClock
```

each signal carries `merkle_clock = H(root of causal DAG of all signals the neuron has seen)`.

```
comparison:   O(1) — single hash equality (equal = in sync)
divergence:   O(log n) — walk DAG to first difference
merge:        union of both DAGs → H(merged root) — deterministic
```

the DAG is the set of all signals transitively reachable through `prev` links and cross-references. the merkle clock is the hash of this DAG's root.

properties:
- O(1) sync check (are we in sync? compare two hashes)
- O(log n) divergence detection (where did we diverge?)
- deterministic merge (union of DAGs, rehash)
- compact (one hemera hash regardless of DAG size)

dependencies: hemera (H, Merkle tree operations)

## step counter

monotonic logical clock per neuron. gap-free.

```
interface:
  step_next(current: u64) → u64
  step_verify_sequence(steps: [u64]) → Result<(), MissingSteps>
```

properties:
- per-source monotonic: step(s_{n+1}) = step(s_n) + 1
- gap-free: missing step = missing signal = detectable
- independent of wall-clock time
- combined with hash chain: step provides logical ordering, prev provides cryptographic binding

## deterministic tiebreak

resolves concurrent signals (same VDF epoch, no causal relationship) into a total order.

```
interface:
  tiebreak(a: Signal, b: Signal) → Ordering
  total_order(signals: DAG) → [Signal]
```

three-level ordering rule:
```
1. causal:     A in B's deps → A before B
2. VDF time:   A.vdf_time < B.vdf_time → A before B (if not causally related)
3. hash:       H(A) < H(B) → A before B (concurrent, same VDF epoch)
```

properties:
- deterministic: every participant computes identical order
- no negotiation, no leader, no external timestamps
- total: every pair of signals has a defined order
- O(1) per comparison

dependencies: hemera (H for hash comparison), mudra::delay (VDF time extraction)

## composed: deterministic total order

given a signal DAG, all four mechanisms compose into a single deterministic sequence:

```
total_order(dag: SignalDAG) → Vec<Signal>

algorithm:
  1. topological sort by causal dependencies (hash chain + merkle clock)
  2. within each causal tier: sort by VDF proof time
  3. within same VDF epoch: sort by H(signal) lexicographic
  result: unique total order, identical on all participants
```

this is the ordering that CRDT merge (local sync) and foculus (global sync) both consume. the ordering layer is scale-independent — same algorithm at local and global.
