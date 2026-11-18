---
tags: mudra, audit, privacy, zheng, veil
date: 2026-10-14
---
# P1 private authorship — implementation evidence, 2026-10-14

Statement: [specs/authorship.md](../specs/authorship.md). Code:
`src/authorship.rs` with the program builder in
`src/authorship_program.rs` (feature `authorship`), tests
`src/authorship_tests.rs`, bench `examples/p1_bench.rs`. This closes the
second blocker named by the P1 gap audit of mudra PR #3 (unmerged, so that
page is not in this tree): "zheng's private execution / blinding
protocol". zheng's veil profile (envelope profile 2) now exists and proves
the P1 relation.
The first blocker is open: no cybergraph/foculus clause yet binds the
statement into the signal. specs/authorship.md § "what the enclosing
protocol defines" lists the four pieces.

Stand `~/cyber/.stands/PW`: mudra `feat/no-legacy-proofs` 5b1b95f
(mudra#20: nox 0.3 / zheng 0.4 pins) + this branch, zheng `release/0.4`
aa54bdd, lens `release/0.2`, nox `release/0.4`, hemera / strata `main`,
honeycrisp `main` 9cbd480. Apple M4 Max, rustc 1.95.0, `--release`.
**Load average 40–75 from other agents during the bench, and up to 130
during the tests.**

That stand predates the held proving key. zheng aa54bdd is zheng#64; the
`veil::ProvingKey` this branch now proves with arrived in zheng#66
(`feat/fast-path-2`, merged into `release/0.4`:
`zheng/rs/src/execution/veil/key.rs`). The tests and the first
measurement table below are from the first prover, which recompiled the
relation on every proof; the held-key figures are the ones measured on
zheng#66 and recorded in zheng `audit/fast-path-2-2026-10.md` §2.

## tests

`cargo test --release --features authorship`: 30 passed (19 existing +
11 new); `cargo test --release`: 19 passed. Clippy: no warnings in the
new files.

**That run is from the first prover (zheng aa54bdd = zheng#64), not from
the committed one.** Three of the tests below read the prover's coins —
`proofs_are_randomized`,
`proofs_from_two_positions_of_one_statement_are_indistinguishable` and
`no_secret_commitment_or_path_limb_appears_in_the_proof` — and zheng#66
moved coin derivation to counter mode, which is where those coins come
from. The committed prover (`veil::ProvingKey` / `veil::prove_with_key`)
has no recorded suite run: the zheng#66 stand exercised prove and verify
through `examples/p1_bench` at d = 20 and 32 only. The numbers in the
table below (the Welch t, the two-sample χ², the window scan) are
therefore the first prover's. Re-running
`cargo test --release --features authorship` on a zheng#66 stand and
recording it here beside this run is owed before the leakage contract is
read as measured on the committed code.

| test | what it establishes |
|---|---|
| `native_tree_and_paths_agree` | the native tree, the empty-leaf padding and the path recomputation agree; depth and size bounds are enforced |
| `every_member_proves_and_the_cost_is_the_same_at_every_position` | completeness at all 5 positions. Each proof's zheng statement (outputs and cycles) equals the one the verifier derives with the depth's constant cost, so the native `H` twin matches zheng's compiled pattern 15 |
| `a_secret_outside_the_set_is_refused_before_proving` | outsider, another member's path, a moved sibling, a flipped direction and a non-boolean direction are all refused |
| `a_non_member_proof_made_around_the_native_check_does_not_verify` | a prover that skips the native check gets a nonzero root flag from the program, and verification fails |
| `a_proof_answers_only_its_own_statement` | other root, content, scope or nullifier (incl. another member's), another depth or key, and non-canonical limbs are rejected |
| `tampered_proof_bytes_are_rejected` | every header byte and ~100 body positions flipped: none accepted; a truncated proof fails |
| `the_statement_carries_only_public_values` | the derived zheng statement is program, R‖C‖S, 0‖N‖C, constant cycles and budget, with no secret, commitment or sibling limb |
| `no_secret_commitment_or_path_limb_appears_in_the_proof` | no 8-byte window of the proof (any alignment, LE or BE) equals a secret, commitment or sibling limb |
| `proofs_from_two_positions_of_one_statement_are_indistinguishable` | one author's commitment at positions 1 and 6 gives one statement from two witnesses. 24 proofs each: length means 164,937 vs 164,960 B (Welch t −0.13); top-4-bit histogram of 494,796 vs 494,866 aligned words gives χ² two-sample 6.1 (reject > 44.3) |
| `proofs_are_randomized` | two proofs of one witness differ, so there is no linkable proof tag |
| `secrets_are_redacted_in_debug_output` | `Debug` of `Secret` and `Path` shows no limb; non-canonical limbs are refused |

zheng's own zk test (`execution/veil/zk_tests.rs`) runs the χ² per proof
component on seeded coins. Its parser and `Coins::seeded` are
crate-private, so the mudra test reads the proof as words with OS
randomness: weaker, and it does not replace zheng's.

Two things the first run found (fixed, recorded because they bind the
spec):
1. zheng builds the subject from public inputs **last first**
   (`ExecutionStatement::inputs` reverses), so input `k` sits at list
   position `11 − k`. The axis helper in zheng's tests documents the
   other order.
2. Pattern 15 as compiled is the structural digest **plus one
   StepSponge permutation** (`execution/hash.rs`, `Builder::hash`). The
   bare `nox::data::hash` tree digest is not the program's `H`.

## measurements

### the first prover (relation recompiled per proof, zheng aa54bdd)

`ZHENG_BACKEND={metal,cpu} cargo run --release --features authorship
--example p1_bench -- 12 16 20 24 32` (`P1_REPS=3`, medians; the bench
took its backend from a `cyber-lens-gpu` dev-dependency, which the branch
has since dropped — see `examples/p1_bench.rs`). The tree is built over at
most 2^16 members (`set ms`, native, the set holder's cost). `key ms` =
program + relation compile + verifying key, once per depth. Prove includes
veil's own relation compile, since veil had no held proving key then.

| d | members in tree | set ms | key ms | cycles | prove Metal | prove CPU | verify (held vk) | proof bytes |
|---|---|---|---|---|---|---|---|---|
| 12 | 4,096 | 481 | 259 | 768 | 812 ms | 937 ms | 47–49 ms | 292,085 |
| 16 | 65,536 | 7,467 | 320 | 984 | 963 ms | 990 ms | 52–58 ms | 293,205 |
| 20 | 65,536 | 7,201 | 379 | 1,200 | 965 ms | 1,004 ms | 56–58 ms | 292,341 |
| 24 | 65,536 | 7,312 | 439 | 1,416 | 1,315 ms | 1,375 ms | 74–75 ms | 418,757 |
| 32 | 65,536 | 7,099 | 549 | 1,848 | 1,342 ms | 1,414 ms | 85 ms | 419,365 |

### the committed prover (held key, zheng#66)

Measured by zheng's fast-path-2 agent on this branch's head a6dd4ba plus
exactly the held-key change this branch now carries as 9483c6e (`Key`
holds `veil::ProvingKey::new(&program, INPUTS)`, `prove` calls
`veil::prove_with_key`); `P1_REPS=5`, load 27–28, same machine. Recorded in
zheng `audit/fast-path-2-2026-10.md` §2 and in soft3
`audit/proof-latency-2026-10.md`:

| d | prove, zheng#66 without a held key | prove, held key (Metal / CPU) |
|---|---|---|
| 12 | — | — |
| 16 | — | — |
| 20 | 488 ms Metal | **255** / 306 ms |
| 24 | — | — |
| 32 | 748 ms Metal | **396** / 477 ms |

A dash is a depth nobody has timed on zheng#66; the first prover's figures
for d = 12, 16 and 24 do not transfer to this one. Proof bytes and verify
time do not depend on the prover's key — 293 KB at d = 20 and 419 KB at
d = 32 came out of this run too — so the first table's columns for them
still stand.

Reading:
- The device gate is no longer what bounds the depth: d = 32 is 396 ms,
  well inside ≤ 1 s, and so is every depth below it on the same prover.
  Half the gain is the held key and half is zheng#66's counter-mode coins
  (965 → 488 ms at d = 20 before the key, 488 → 255 ms with it).
- The spec's default depth is 20 because the proof size steps between the
  measured depths 20 and 24, not because of latency: 292,341 bytes at
  d = 20 against 418,757 at d = 24, a step of 1.43x. d = 21, 22 and 23
  are not measured, so the exact depth of the step is not known.
- Metal bought little on the first prover (≤ 15%); with counter-mode coins
  and a held key it is ~20% ahead of CPU (255 vs 306 ms at d = 20).
- Proofs are 292–294 KB (d ≤ 20) and 419 KB (d ≥ 24), far over the
  64 KB profile-1 goal, and the held key does not move them.
- The native set build is ~110 µs per member (each node hashes two
  digest nouns structurally: 15 hemera calls plus one permutation). It
  is set-holder work, once per root.

## wanted from zheng (not changed here; another branch owns veil internals)

The held proving key asked for in the first draft of this page exists and
is what the branch proves with (zheng#66, `veil::ProvingKey` /
`veil::prove_with_key`). What is still wanted:

- veil proof size: 292–419 KB is the wire cost of every private signal;
- a seeded-coins entry or public parser, so downstream zk tests can run
  zheng's per-component χ² rather than a word-level one.
