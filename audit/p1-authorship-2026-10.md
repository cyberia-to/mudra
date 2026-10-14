---
tags: mudra, audit, privacy, zheng, veil
date: 2026-10-14
---
# P1 private authorship — implementation evidence, 2026-10-14

Statement: [specs/authorship.md](../specs/authorship.md). Code:
`src/authorship.rs` (feature `authorship`), tests
`src/authorship_tests.rs`, bench `examples/p1_bench.rs`. This closes
the second blocker named in `audit/p1-signal-content-commitment-gap.md`
(mudra PR #3, "zheng's private execution / blinding protocol"). zheng's
veil profile (envelope profile 2) now exists and proves the P1 relation.
The first blocker is open: no cybergraph/foculus clause yet binds the
statement into the signal. specs/authorship.md § "what the enclosing
protocol defines" lists the four pieces.

Stand `~/cyber/.stands/PW`: mudra `feat/no-legacy-proofs` 5b1b95f
(mudra#20: nox 0.3 / zheng 0.4 pins) + this branch, zheng `release/0.4`
aa54bdd, lens `release/0.2`, nox `release/0.4`, hemera / strata `main`,
honeycrisp `main` 9cbd480. Apple M4 Max, rustc 1.95.0, `--release`.
**Load average 40–75 from other agents during the bench, and up to 130
during the tests.**

## tests

`cargo test --release --features authorship`: 30 passed (19 existing +
11 new); `cargo test --release`: 19 passed. Clippy: no warnings in the
new files.

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

`ZHENG_BACKEND={metal,cpu} cargo run --release --features authorship
--example p1_bench -- 12 16 20 24 32` (`P1_REPS=3`, medians). The tree is
built over at most 2^16 members (`set ms`, native, the set holder's
cost). `key ms` = program + relation compile + verifying key, once per
depth. Prove includes veil's own relation compile, since veil has no held
proving key.

| d | members in tree | set ms | key ms | cycles | prove Metal | prove CPU | verify (held vk) | proof bytes |
|---|---|---|---|---|---|---|---|---|
| 12 | 4,096 | 481 | 259 | 768 | 812 ms | 937 ms | 47–49 ms | 292,085 |
| 16 | 65,536 | 7,467 | 320 | 984 | 963 ms | 990 ms | 52–58 ms | 293,205 |
| 20 | 65,536 | 7,201 | 379 | 1,200 | 965 ms | 1,004 ms | 56–58 ms | 292,341 |
| 24 | 65,536 | 7,312 | 439 | 1,416 | 1,315 ms | 1,375 ms | 74–75 ms | 418,757 |
| 32 | 65,536 | 7,099 | 549 | 1,848 | 1,342 ms | 1,414 ms | 85 ms | 419,365 |

Reading:
- **≤ 1 s up to d = 20** (1,048,576 authorities) on Metal, and at the
  edge on CPU, under load. The spec's default depth is 20.
- Metal buys little (≤ 15%). The latency audit (soft3
  `audit/proof-latency-2026-10.md`) found the same: the veil prover's
  hiding commitment (sequential constant-time XOF coins, single-threaded
  leaf hashing) is not device-accelerated.
- Proofs are 292–294 KB (d ≤ 20) and 419 KB (d ≥ 24), far over the
  64 KB profile-1 goal. Size and prove time step together between 20
  and 24 (the committed size class doubles), not per level.
- The native set build is ~110 µs per member (each node hashes two
  digest nouns structurally: 15 hemera calls plus one permutation). It
  is set-holder work, once per root.

## wanted from zheng (not changed here; another branch owns veil internals)

- a held proving key for veil (`veil::ProvingKey`, like
  `succinct::ProvingKey`): every proof recompiles the relation today
  (`key ms` above is the cost);
- veil proof size: 292–419 KB is the wire cost of every private signal;
- a seeded-coins entry or public parser, so downstream zk tests can run
  zheng's per-component χ² rather than a word-level one.
