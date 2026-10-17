---
tags: mudra, privacy, zheng, identity
crystal-type: entity
crystal-domain: crypto
alias: P1 private authorship, signal authorship proof, edge public author private
---
# authorship — P1: the edge is public, the author is private

P1 (`cyber/launch.md` core 6, corrected 2026-09-22): the network sees
that particle p links particle q and the axon weight. It does not see which
neuron signed. The gap is recorded in
`audit/p1-signal-content-commitment-gap.md` (mudra PR #3): the signal
names its author (`ν`, `neuron[32]` per link), and nothing bound
[identity](identity.md)'s private authority profile to a signal. This page
is that binding's statement. Implementation: `src/authorship.rs`,
feature `authorship`.

## objects

All values are Goldilocks elements (`p = 2^64 − 2^32 + 1`), canonical
(`< p`). A digest is four elements, the 32-byte particle read as four
little-endian u64 limbs.

| object | definition |
|---|---|
| secret `s` | four elements. Derived from 32 bytes of key material `k` as `hemera("mudra-p1-secret-v1" ‖ k)`, each 8-byte LE limb reduced mod p. Never on the wire. |
| authority commitment `a` | `H([D_AUTH s])` |
| authority set | Merkle tree of depth `d` over `2^d` leaves, leaf `i` = the commitment of member `i` or `EMPTY = [0 0 0 0]`. Node = `H([L R])`. Root `R`. |
| path of leaf `i` | siblings `σ_0..σ_{d−1}` from the leaf level up; directions `b_j` = bit `j` of `i` (0: the running node is the left child) |
| nullifier scope `S` | a digest the enclosing protocol fixes per context (see § what the enclosing protocol defines) |
| nullifier `N` | `H([D_NULL [s S]])` |
| content commitment `C` | a digest of the signal without its author fields, defined by the signal codec (§ enclosing protocol) |

Domain atoms: `D_AUTH = 0x5031_4155_5448` ("P1AUTH"),
`D_NULL = 0x5031_4e55_4c4c` ("P1NULL").

`H` is nox's hash pattern (15) as zheng compiles it
(`zheng/rs/src/execution/hash.rs`). It has two steps:
1. Take the structural digest of the noun: an atom is hemera's tree leaf of
   its 8 LE bytes, and a pair is hemera's tree node of its halves' digests
   (`nox::data::hash`).
2. Apply one plain hemera `StepSponge` permutation with that digest in the
   rate. The first four lanes are `H`.

A digest enters a noun as `[[d0 d1] [d2 d3]]`.

Depth: `1 ≤ d ≤ 32`. Default **d = 20** (1,048,576 authorities). That is the
largest depth measured within the ≤ 1 s device gate
(`audit/p1-authorship-2026-10.md`). Depths 24 and 32 cost about 1.4 s.

## statement

```text
public   d, R, C, S, N
secret   s, σ_0..σ_{d−1}, b_0..b_{d−1}
relation a = H([D_AUTH s])
         cur_0 = a
         cur_{j+1} = H(b_j = 0 ? [cur_j σ_j] : [σ_j cur_j])
         cur_d = R
         N = H([D_NULL [s S]])
```

C is bound as a public input: the proof is a signature of knowledge on C.
A proof made for one content commitment, set root, scope or nullifier
verifies for no other.

## program

The relation is the zheng execution of one nox program `P_d`, fixed by `d`.
Prover and verifier build it identically (`authorship::program`).

Notation: `axis n = [0 n]`, `quote x = [1 x]`, `compose(x, y) = [2 [x [1 y]]]`,
`cons(x, y) = [3 [x y]]`, `branch(t, y, n) = [4 [t [y n]]]`,
`eq(x, y) = [9 [x y]]`, `hash x = [15 x]`,
`secret = [16 [[1 1] [1 0]]]` (call pattern, tag 1, check `0`),
`dig(f) = cons(cons(f0, f1), cons(f2, f3))`.

Public input order is `R0..R3, C0..C3, S0..S3`. zheng builds the subject
last input first, `[i11 [i10 … [i0 0]]]`, so input `k` is axis
`2^(11−k+2) − 2`.

```text
A = cons(dig(R), cons(dig(C), cons(dig(S), dig(secret))))
      → [R [C [S s]]]                                  R=2 C=6 S=14 s=15
B = cons(cons(hash(cons(quote D_AUTH, axis 15)), path),
         cons(axis 2, cons(axis 6, hash(cons(quote D_NULL, cons(axis 15, axis 14))))))
      path = cons(cons(dig(secret), secret), … cons(cons(dig(secret), secret), quote 0))
             (d entries, level 0 the head)
      → [[a path] [R [C N]]]
L = cons(hash(branch(axis 13, cons(axis 2, axis 12), cons(axis 12, axis 2))), axis 7)
      one level on [cur [[σ b] rest]]
C' = cons(compose(axis 2, L ∘ … ∘ L  (d times)), axis 3)
      → [[root 0] [R [C N]]]                           root=4 R=6 C=14 N=15
D = cons(eq(axis 4, axis 6), cons(axis 15, axis 14))
      → [flag [N C]]
P_d = compose(A, compose(B, compose(C', D)))
```

The secret stream, in consumption order (left before right in every cons):
`s0..s3`, then for `j = 0..d−1`: `σ_j` (four limbs), `b_j`.

The control flow does not depend on secrets: both branch arms are a cons
of two axes. Every execution of `P_d` therefore has one cost, `cycles(d)`
(768 / 984 / 1,200 / 1,416 / 1,848 at d = 12 / 16 / 20 / 24 / 32).

## proof and verification

Profile: zheng zero-knowledge profile **veil** (envelope profile 2),
default `HidingParams`. Its bits and its zero-knowledge statement are in
`zheng/specs/soundness.md` § zk profile. The proof is the veil proof
bytes alone; the statement is not carried.

The verifier holds the trusted root `R` of a committed authority set and
computes `C` from the signal itself. It derives the zheng statement:

```text
program  = encode(P_d)
input    = R ‖ C ‖ S
output   = 0 ‖ N ‖ C
cycles   = cycles(d)
budget   = 2^24
```

It accepts iff every limb is canonical and `veil::verify` accepts with a
verifying key derived for `P_d`. The output's leading 0 is the root check:
a non-member's execution outputs a nonzero flag, and no derived statement
accepts it.

The prover refuses before proving when the secret and path do not open
to `R`, when a direction is not 0/1, or when a limb is non-canonical.
Proving randomness comes from the OS (`veil::prove`).

## leakage contract

What the verifier, and anyone relaying the proof, learns:
- `d, R, C, S, N`, and that the author's commitment is a leaf of the set
  at `R`. The anonymity set is the set's members: a small or sparse set
  leaks accordingly.
- The proof length. It varies with the prover's randomness, not with the
  witness: 292–294 KB at d ≤ 20 and 418–420 KB at d = 24–32. Two
  positions of one statement give the same length distribution (test
  `proofs_from_two_positions_of_one_statement_are_indistinguishable`).
- `N` links the proofs of one author within one scope `S`. This is by
  design: it detects a second signal where a scope allows one. Across
  scopes, nullifiers are unlinkable under the PRF assumption on `H`.

What it does not learn: `s`, `a`, the position `i`, the siblings and the
directions. The zero-knowledge property is zheng's veil statement
(honest-verifier statistical zero knowledge, BCS transform with salted
leaves). The tests here check no secret, commitment or sibling limb in
the proof bytes, check that the statement carries public values only,
and check two-position indistinguishability (Welch t and χ²).

Not covered: the prover is not constant time (zheng states it). Timing
on a shared machine is a side channel outside this contract. Edges and
axon weights are public under P1. Who holds a position and who owns the
energy (amount, ICBS position) are P2 (`stealth.md`, `veil.md`), not
this statement.

## what the enclosing protocol defines

mudra fixes the statement. cybergraph, foculus and bbg must still define
the following before a signal carries it:

1. **The authority set.** Which committed set a signal proves against:
   a bbg dimension holding authority commitments `a`, how a neuron
   registers `a` (an authorized transition per [identity](identity.md)),
   and which root a verifier trusts (finalized state root → set root).
2. **`C`.** The signal's content commitment with `ν` and every per-link
   `neuron[32]` removed (`cybergraph/specs/signal.md`,
   `foculus/specs/signal-codec.md`).
3. **`S`.** The nullifier scope. Proposed: `H(network ‖ step)` for a
   signal chain (one signal per author per step), or `H(network ‖ epoch)`
   for rate limits. The enclosing protocol picks one.
4. **The codec.** `ν` replaced by `(R, N, proof)`, and the verifier
   procedure above added to signal admission.

## parameters

| d | members | prove (Metal / CPU) | verify | proof |
|---|---|---|---|---|
| 12 | 4,096 | 0.81 / 0.94 s | 49 ms | 292 KB |
| 16 | 65,536 | 0.96 / 0.99 s | 52 ms | 293 KB |
| 20 | 1,048,576 | 0.97 / 1.00 s | 56 ms | 292 KB |
| 24 | 16.7 M | 1.32 / 1.38 s | 74 ms | 419 KB |
| 32 | 4.3 G | 1.34 / 1.41 s | 85 ms | 419 KB |

These were measured on 2026-10-14 under load average 40–75 from other
processes; see `audit/p1-authorship-2026-10.md`.
