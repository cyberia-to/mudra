# Local signature performance reference

Observed **2026-09-12** on Apple M4 Max, 16 logical CPUs, 48 GiB memory,
macOS arm64, OpenSSL **3.6.2** default provider. The full OpenSSL build output
and raw host values are in [openssl.txt](openssl.txt) and [host.txt](host.txt).

This measures conventional key generation/signing/verification through OpenSSL.
It does not measure Quantus's Rust provider, a Mudra adapter, circuit execution,
private proofs, blockchain throughput, confirmation latency or phone behavior.

## Method

Executed three sequential trials of:

```text
openssl speed -elapsed -seconds 1 ML-DSA-65 ML-DSA-87 SLH-DSA-SHA2-128s SLH-DSA-SHA2-128f SLH-DSA-SHA2-256s SLH-DSA-SHA2-256f
```

Each trial gives each algorithm one second per keygen/sign/verify phase.
OpenSSL selects its benchmark message and signing path; this experiment does
not sweep payload sizes, contexts, keys or deterministic versus hedged mode.
No `-multi` worker count or CPU affinity was set. The machine was not isolated
from other workspace activity and no power/thermal controls were applied.

`1000 / operations_per_second` gives average milliseconds per operation within
one phase. The table uses the median of the three phase averages. The min/max
columns range over those three averages; they are **not** individual-operation
latency percentiles. OpenSSL's printed rates are rounded, especially for slow
SLH variants with only a few signatures per phase. These are short reference
measurements, not a robust tail-latency or sustained-throughput characterization.

## Observed results

| Algorithm | Keygen, ms | Sign, ms | Sign trial range, ms | Verify, ms |
|---|---:|---:|---:|---:|
| ML-DSA-65 | 0.101 | 0.472 | 0.449–0.519 | 0.094 |
| ML-DSA-87 | 0.141 | 0.582 | 0.506–0.631 | 0.138 |
| SLH-DSA-SHA2-128s | 20.408 | 140.845 | 136.986–151.515 | 0.150 |
| SLH-DSA-SHA2-128f | 0.297 | 7.479 | 6.873–7.893 | 0.537 |
| SLH-DSA-SHA2-256s | 22.371 | 312.500 | 263.158–322.581 | 0.457 |
| SLH-DSA-SHA2-256f | 1.437 | 25.907 | 24.038–30.303 | 0.757 |

All 18 algorithm/trial combinations completed successfully. This verifies that
the selected benchmark paths ran; it does not independently audit cryptography.
The separate [Quantus/OpenSSL cross-check](../../quantus-mldsa-crosscheck/README.md)
was executed earlier the same day and establishes bounded interoperability.

## Artifacts and reproduction

Raw `trial-{1,2,3}.stdout.txt` files contain OpenSSL's result tables;
`trial-{1,2,3}.stderr.txt` contain phase counts and elapsed times.
[samples.csv](samples.csv) preserves all 18 computed observations;
[summary.csv](summary.csv) preserves the median and signing ranges.

From the Mudra repository root:

```text
nu audit/signature-optimality/benchmarks/summarize.nu
nu audit/signature-optimality/benchmarks/run.nu
```

The first command regenerates the checked-in CSVs from the original raw logs.
The second runs new measurements into ignored `generated/` without replacing
the original evidence. It requires Nushell, OpenSSL with these algorithms and
macOS `sysctl`; adapt host metadata collection explicitly on other platforms.
The saved summarizer intentionally reads the original logs; to summarize a new
run, copy it into `generated/` and run that copy there.

The summarizer was executed successfully against these logs. No additional
production source or tests were changed for this benchmark.
