# Fresh checks — 2026-09-16

Companion evidence for [the readiness assessment](../readiness-2026-09-16.md).
Host: macOS Apple Silicon, rustc 1.95.0 (Homebrew). These are local working-tree
checks, including uncommitted source. No source/dependency fix was made during
the assessment. This run did not repeat the whole P13 GUI/storage/kill-test suite
or establish production cryptographic security.
Test stdout retains its output with trailing blank lines removed.

| Check | Result | Raw evidence |
|---|---|---|
| Neuron default workspace | 45 passed, zero failed/ignored, exit 0 | [stdout](neuron-tests.stdout.log), [stderr](neuron-tests.stderr.log), [exit](neuron-tests.exit) |
| Neuron all features | 48 passed, zero failed/ignored, exit 0 | [stdout](neuron-all-features.stdout.log), [stderr](neuron-all-features.stderr.log), [exit](neuron-all-features.exit) |
| Mudra default | 19 unit + 1 compatibility + 2 vectors = 22 passed | [log](default-tests.log) |
| Mudra no default features | 16 unit + 1 compatibility = 17 passed | [log](no-default-tests.log) |
| Mudra `prove` all targets | Failed: missing `Statement.bbg_root`, E0063 | [log](prove-check.log) |
| Mudra malformed Unicode claim | Reproduced parser panic, exit 101 | [command/output](malformed-claim-check.json) |

Feature suites overlap; their counts must not be added as distinct tests. No
Mudra optional proof tests ran because compilation failed. Existing upstream
Fjall warnings appear in Neuron logs; no fresh Clippy success is claimed here.

Commands, from each repository root:

```sh
# Neuron
cargo test --workspace --locked --offline --jobs 2
cargo test --workspace --all-features --locked --offline --jobs 2

# Mudra (all three used the isolated target directory below)
CARGO_TARGET_DIR=/tmp/neuron-mudra-readiness-20260916/target cargo test --locked --offline --jobs 2 -- --test-threads=2
CARGO_TARGET_DIR=/tmp/neuron-mudra-readiness-20260916/target cargo test --no-default-features --locked --offline --jobs 2 -- --test-threads=2
CARGO_TARGET_DIR=/tmp/neuron-mudra-readiness-20260916/target cargo check --features prove --all-targets --locked --offline --jobs 2
```

[Input snapshot](input-snapshot.json) records Mudra file hashes and six repository
heads/statuses. [Final comparison](final-snapshot-comparison.json) confirms those
36 recorded file hashes and six head/status observations were unchanged during
Mudra checks. Neuron's [base revision](neuron-head.txt), [dirty status](neuron-dirty.txt)
and [compiler version](neuron-toolchain.txt) identify its working-tree scope.

[P13 source comparison](p13-source-comparison.json) independently checks all 94
Neuron and 42 Mudra files recorded in the earlier
[source manifest](../../../soft3/audit/neuron-cell/source-manifest.json): every
listed file matches. It does not certify unlisted files, the full transitive
dependency closure, or reproduction from committed heads alone.

The current Neuron suites include real CLI processes, synthetic before/after
commit errors and panic/drop/reopen worker recovery. BBG's separately recorded
process-kill and disk-fault tests were not rerun. No current C01–C58 completeness,
concurrent-worker race coverage, network finality or private recovery result is
inferred from these passing local suites.
