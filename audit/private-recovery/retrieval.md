# Private wallet discovery with exceptionally little client work

Research snapshot: 2026-09-12. Primary papers and authors' artifacts; no Cyber workspace changes or new performance measurements. This concerns retrieval and recovery only. Mudra's proof authorization and CSIDH design remain the inputs to the integration, rather than being replaced with conventional signatures.

## Decision supported by the evidence

**There are credible ways to remove UnifOMR's linear client stream.** The strongest immediate experiment is compact, epoch-bounded OMR using SophOMR, combined with an authenticated checkpoint of the wallet's live state. InstantOMR and Oblivious Signaling offer different continuous-service tradeoffs. None supplies complete, authenticated, permanently recoverable wallet state by itself.

Keep the variables separate:

- **N**: global notifications in the queried history/window; **ΔN**: new global notifications since the checkpoint.
- **k**: the wallet's true notifications in that window; **F**: false-positive candidates; **B**: padded extraction capacity, requiring a bound on `Pr[k+F>B]`.
- **U**: currently live, unspent wallet notes. Usually U can be much smaller than all historical receipts k.
- **R**: registered recipients; **P**: payload bytes; **ν**: fixed inbox capacity.

Enumerating all recovered notes requires at least their output size: Ω(kP) for receipt history, Ω(U) records for an explicit live-note snapshot. A constant-size balance/proof can avoid enumeration, but cannot make delivery, authenticated state construction and spend-data availability disappear. Those are different requested outputs.

| Approach | Client work and communication | Where the work moves | Recovery boundary |
|---|---|---|---|
| UnifOMR | O(N) compact detection stream plus batch PIR | Linear detection per recipient; reusable public preprocessing | Excellent server-time tradeoff, but not the minimum client work |
| SophOMR | Compact B/P-dependent result; client decoding independent of N at fixed parameters, superlinear in B | Homomorphic match predicate and compression over N per recipient | Strong baseline for client-light historical recovery |
| InstantOMR | Compact result, then decrypt/decode | Per-message TFHE detection and digest maintenance per recipient | Low wake-up latency when the service has already processed the whole unseen window |
| Oblivious Signaling | O(ν) inbox read/decrypt | All recipient inboxes updated on delivery | Current FIFO retains only ν newest messages; requires an independent durable recovery path |
| Hintless/preprocessed PIR | Privately fetch known indices; query construction may still be substantial | Database preprocessing and query evaluation | Cannot discover unknown relevant indices without an additional layer |
| Verified private checkpoint | Target U live records plus small catch-up, instead of all receipt history | Someone constructs and durably stores complete state | Must establish freshness, coverage, availability and unseen-message recovery |

## Compact OMR is an existing option

The OMR lineage is original OMR → GOMR/PerfOMR → SophOMR, with InstantOMR emphasizing streaming and UnifOMR exchanging compactness for faster server execution. Their common critical step is obtaining an **encrypted relevance indicator**, not merely an encrypted clue-decryption fragment.

**SophOMR, final USENIX 2026 paper:** homomorphic range checks produce sparse relevance bits; SIMD-aware algebraic compression reduces these and masked payloads to a capacity-bounded digest. Compression itself is deterministic under its sparsity precondition. Client decompression is superlinear in k, so “output-sensitive” does not mean exactly O(k). The paper states a passive-adversary threat model; it does not prove malicious-server completeness. [Final paper, §§1.2–1.3, 2.4, 4–5](https://www.usenix.org/system/files/usenixsecurity26-lee-keewoo.pdf).

| Author measurement, P=612 bytes, k=50 | N=65,536 | N=524,288 |
|---|---:|---:|
| Server | 157 s | 1,098 s |
| Client decode | 14.0 ms | 12.0 ms |
| Digest | 263 KB | 263 KB |
| Detection key | 114 MB | 114 MB |

These final-paper Tables 4/8 use a single-threaded Google n2-standard-8 Intel Ice Lake, five-run averages. They differ from UnifOMR's separately measured SophOMR baseline; do not mix them into a same-host speedup. Parameters target 128-bit lattice security, with roughly 30-bit per-message detection completeness. Keys/HE work are per recipient; public matrix/board preprocessing can be reused, but the private scan result cannot. [Final paper, §5 and Appendix A–B](https://www.usenix.org/system/files/usenixsecurity26-lee-keewoo.pdf#page=14).

**InstantOMR:** final Table 2 reports measured Primus-fhe **274.51 ms per global message per core**, **201 ms** client decoding, **825 KB** digest, **113 MB** detection key and **714 B** clue at N=65,536, k=50, P=612. **88.39 ms is an estimated TFHE-rs protocol result**, not a complete implementation measurement. Hardware: c3d-highcpu-360, 180 physical cores, 708 GB; the listed per-core row is explicitly one core. Its advantage is incremental processing while a wallet sleeps; a newly registered detector must still process historical N. It needs B in advance, RLWE with circular security, and permits malicious parties for transcript privacy while correctness assumes honest execution. False-positive load remains in the compactness formula. [Final paper, §§4–6, Theorem 5.1](https://www.usenix.org/system/files/usenixsecurity26-liang.pdf).

For R continuously served wallets, naive per-recipient detection costs roughly R·ΔN predicates per update period. A server does not get R private scan results from one public scan merely because clue preprocessing or memory reads are shared. Freshly restored keys may also require another sizable detection-key upload. These are deployment cost deductions, not author throughput benchmarks.

## FHE plus IBLT/sketch/compaction: what it actually buys

This is a real construction family. Once `Enc(b_i)` exists for `b_i∈{0,1}`, accumulate encrypted bucket counts, index sums, checksum sums and payload sums, weighted by `b_i`. Public bucket positions depend on i, so accumulating the sketch need not expose which entries matched. The client decrypts a small sketch and recovers its sparse entries.

Fleischhacker–Larsen–Obremski–Simkin's **Stacked IBLT** supports compression of a length-N encrypted vector with at most B nonzeros into `O(B + log(1/δ)·loglog(1/δ))` ciphertext cells, improving the preceding IBLT compression bound. Cells contain key/value/checksum data and have nonconstant bit width; counting ciphertext cells is not counting bytes. The result is a data structure/compression theorem, not a measured complete wallet or automatically verifiable retrieval protocol. [Paper, Theorem 1 and §5.2](https://cs.au.dk/~larsen/papers/SmallerIBLT.pdf).

**The barrier for a direct UnifOMR add-on:** its partial decrypted values are dense, including irrelevant clues; their useful property is that the client can range-test them. Sparse sketches cannot preserve all nonlinear range-test results by simply summing those noisy values. The server must privately evaluate that predicate first, or the clue/protocol must change. SophOMR and InstantOMR pay for precisely this step. Generic oblivious sorting/compaction can also pack matches but adds comparisons, circuit depth and routing work. These are construction-level deductions, not evidence of a new faster implementation.

A credible experiment therefore compares (a) SophOMR's SIMD compression, (b) encrypted IBLT after the same detector, and (c) compact **indices only**, followed by batch PIR. In (c), payload-width FHE work may fall, but the second round and PIR query construction return. Require fixed padding, deterministic authenticated epoch membership, checksums, a peeling/decoding failure budget and an overflow policy that does not silently discard notes or reveal response-dependent choices.

## A newer architecture: push into encrypted inboxes

**Oblivious Signaling (OSig), USENIX 2026**, replaces pull-time scanning with uniform encrypted inbox updates. For batch size g, tag length ℓ and R recipients, server cost is `O(R(gℓ+ν))` FHE operations per batch; client reads O(ν) slots. It targets frequent checks and sparse receipts. However, only the **newest ν deliveries** are guaranteed retained; older entries are evicted even during honest execution. Sharding reduces the effective anonymity set. [Final paper, §§2, 4, 7](https://www.usenix.org/system/files/usenixsecurity26-shuhan.pdf).

The prototype reports **97 μs** local decryption for ν=4 **short-message slots**, not full financial-note recovery; tested signals range from 2 to 20 bits. It reports ~127.8 MB server evaluation key per recipient, ~32.9 KB public key and ~16.6 KB sender signal/tag. Parameters use 128-bit lattice estimates but εp=2^-32 and per-batch collision budget about 2^-27.42. The theorem additionally assumes key/message indistinguishability, circular security and a wrong-key-decryption property. Malicious-state integrity and post-read leakage are excluded. [Final paper, §§3–7](https://www.usenix.org/system/files/usenixsecurity26-shuhan.pdf#page=10).

For Mudra this is a candidate wake-up/notification service **only with durable archived delivery and explicit overflow detection**. Putting only a pointer in the inbox still requires private, authenticated payload retrieval. Its tiny default inbox is not a seed-only recovery mechanism.

## PIR choices after the indices or checkpoint handle are known

**YPIR / HintlessPIR:** avoid a database-dependent client hint download; YPIR still performs reusable server preprocessing and scans the database for each query. Its published 32 GB example transfers 2.5 MB total for one small record, at 12.1 GB/s/core server throughput; these are server throughput and bandwidth, not client milliseconds. Packing keys travel with queries rather than requiring persistent per-client server state. Assumptions include LWE/RLWE with key-dependent/circular-security conditions. Cross-client batching amortizes memory reads, not all CPU instructions. [YPIR paper, §§1–4](https://www.cs.utexas.edu/~dwu4/papers/YPIR.pdf).

**SPIRIT, USENIX 2026:** a useful newer batch-PIR comparison, with ephemeral conversion state shared across one batch and no persistent per-client hints. But its headline server speed hides client work. At 2^21 records ×256 B, batch 256, it reports **2.8 s server /19.0 s client /12.4 MB total**, or **8.4 s /19.8 s /4.5 MB** with response compression. Client time includes rebuilding public random matrices from seeds. Hardware: Xeon Gold 5318Y, 512 GB. Security is explicitly semi-honest, under LWE, circular-secure HE conversion and a random-oracle matrix generator. It does not remove discovery's N-dependence; even its batch communication is `O_λ(√(kN))`, not O(k). [Final paper, Theorem 3.1, Tables 2/4](https://www.usenix.org/system/files/usenixsecurity26-zhang-zhou.pdf).

**Piano → Plinko:** trade a client hint for sublinear online work. Piano initially streams the whole database and stores roughly `O(√N log N)` hints. Plinko improves the tradeoff to `Õ(N/r)` query time for client storage r and polylogarithmic worst-case per-entry hint updates. Its normal bootstrap still streams N; it discusses FHE-generated hints as another expensive bootstrap model. A lost phone without a preserved hint must pay that bootstrap again; continuing updates depend on global changes, not just its U notes. They use symmetric/one-way-function assumptions; a PQ instantiation and quantum-proof model require separate qualification. [Piano authors' explanation](https://www.cs.cmu.edu/~csd-phd-blog/2024/piano-private-information-retrieval/), [Plinko, §§1, 5](https://eprint.iacr.org/2024/318.pdf).

None of these PIR protocols alone proves that an indexer returned **all** matching notes. An authenticated map can prove a particular key's value/absence, provided the canonical map construction and complete key-query set are established. An arbitrary indexer-supplied root is insufficient.

## Complete private checkpoints: the route from historical k to live U

An application checkpoint should contain encrypted live notes/spend data, spent-status context, derivation/profile versions, the last covered canonical epoch/root, and a commitment to the previous state. It must be recoverably addressed from the seed or a retained recovery descriptor. A backup's AEAD protects its contents; it does not prove the backup is newest, includes every receipt, or remains available.

A useful target is **O(U + Δk)** recovered records plus proof verification and private-query costs, rather than replaying k lifetime receipts. This is a proposed integration target, not a performance result from the retrieved papers. Discovery still processes ΔN somewhere. If a user has no previous complete checkpoint, historical matching has not disappeared: a service must construct it from N or an already maintained authenticated private state.

The inductive relation to prove is approximately:

`state_e = Apply(state_(e-1), all_matching_notifications(epoch_e), canonical_spent_changes(epoch_e))`.

Bind the proof to chain/genesis, exact epoch range/root/count, protocol and detector profile, previous checkpoint commitment, and new encrypted-state commitment. The recipient verifies the current canonical root/finality and the checkpoint's coverage. FHE evaluation over encrypted detection keys can in principle be proved without disclosing the wallet secret; efficiently proving the entire evaluation remains implementation work. This keeps authorization in Mudra/Zheng and treats retrieval correctness as a separate relation.

Tachyon's primary proposal provides the relevant **proof-carrying wallet-state** pattern: outsource ongoing state-proof maintenance, with protocol changes to nullifiers and accumulators. Its described path assumes out-of-band note delivery and explicitly loses blockchain-only seed recovery without additional storage infrastructure. It exposes note nullifiers to the service in the initial outline, with grouping/network leakage requiring further work. The proposed Orchard/Pasta-based machinery is not itself a PQ implementation to transplant into Mudra. Use the architectural separation as precedent, not an existing complete recovery component. [Sean Bowe, proposal and boundaries](https://seanbowe.com/blog/tachyon-scaling-zcash-oblivious-synchronization/).

A wallet restoring **only a balance** could receive a scalar plus a complete-state proof, then fetch spend data on demand. A wallet restoring all spendable notes pays Ω(U) output. Keep these UX contracts distinct. Full transaction history, encrypted off-chain receipts, labels and historical keys need a separate archive policy.

## Completeness and quantum boundaries that affect the choice

- Compactness is conditional: budget detection errors, `Pr[k+F>B]`, sketch failures and proof errors across all epochs. A per-message 2^-30 miss target is not a whole-wallet 2^-128 guarantee. Padding and retries need an explicit leakage policy.
- Server privacy is different from correct execution. SophOMR's stated threat model is passive; InstantOMR/OSig separately allow malicious transcript privacy but assume honest correctness; SPIRIT explicitly targets semi-honest security. A proof for every returned leaf does not certify no omitted leaves.
- There is a concrete active-security alternative if the trust model changes: **Private Signaling Secure Against Actively Corrupted Servers** proves all-signals-or-abort with two non-colluding servers. It reports a 33.57 KB signal digest at N=2^19 and about two minutes on 16 threads/LAN, but uses **ElGamal/DDH**, so its concrete construction is not PQ. This is evidence that completeness can be addressed, not a drop-in recommendation. [Primary paper, §§1, 3.3 and evaluation](https://eprint.iacr.org/2025/1056.pdf).
- Existing proof authorization and CSIDH payload/key design do not automatically satisfy an OMR scheme's clue predicate or FHE assumptions. A companion versioned clue must be bound to the encrypted notification; otherwise an honest-value transaction can still have an undiscoverable or misleading clue. That binding, canonical data availability and a recovery path are design requirements.
- Checkpoint freshness needs authenticated latest-state lookup; reorgs require rollback/replay. Deleting notifications after service acknowledgment is unsafe unless a durable recovery invariant replaces them. Authenticated roots prove integrity, not availability.

## Bounded next experiment

1. Freeze an epoch input/output contract and complete ground-truth local scan; choose the concrete no-loss probability target, maximum wallet activity and padding policy.
2. Run **SophOMR first**, because its published artifact already exercises compact extraction. Measure full first restore, incremental epochs, detection-key upload, memory and server CPU per active recipient; include varying B and U.
3. Add authenticated encrypted checkpoints; compare explicit U-note restoration with balance-plus-on-demand-spend-data restoration. Test omission, stale roots, reordered epochs, overflow, missing archive and reorgs.
4. Compare InstantOMR for continuous service; evaluate encrypted IBLT or indices-only extraction only if payload compression is a measured bottleneck. Treat OSig as an optional delivery accelerator with archival backing.
5. Compare PIR backends on the **actual checkpoint/witness payloads and client device**. Include query generation, bootstrap and updates; server-only benchmarks can reverse the apparent winner.

## Executed checks and pinned artifacts

Downloaded and text-extracted the cited PDFs, read the relevant schemes/security models/benchmark tables, inspected authors' repositories and parameter/decode code, and recorded hashes. No compilation, cryptographic tests, or performance benchmarks were run in this task. No live service was contacted beyond primary-source downloads. Reported numbers are authors' measurements, with explicit estimates marked above.

Authors' code:

- [SophOMR](https://github.com/keewoolee/SophOMR/tree/becd5bffbed527260e54036d89a4256161d41168), `becd5bffbed527260e54036d89a4256161d41168`; OpenFHE 1.4.0/NTL, optional HEXL. README warns that its ring-switching uses unsupported OpenFHE APIs. Smallest documented test is still 1–2 compute-minutes with prerequisites.
- [InstantOMR](https://github.com/xiangxiecrypto/tfhe-omr/tree/e5c62f2c34fb488de1242bd3907c7ec7d41cecc8), `e5c62f2c34fb488de1242bd3907c7ec7d41cecc8`; Rust Primus-fhe, separate TFHE-rs bootstrapping measurements. macOS explicitly untested; example caps N at 65,536. Full single-core benchmark is hours.
- [OSig](https://github.com/shuhanmirza/oblivious-signaling-usenix26/tree/a6cea3f47763b291061053d52ed0b41a599905f4), `a6cea3f47763b291061053d52ed0b41a599905f4`; default source stores two radix-4 shortints per slot, confirming a short-signal prototype.
- [SPIRIT](https://github.com/zhouzhangwalker/SPIRIT/tree/7e4c114807d5a83ae5cbad17c514df1dbc32dfc3), `7e4c114807d5a83ae5cbad17c514df1dbc32dfc3`; its optional randomized-hint benchmark mode explicitly skips real hint setup and cannot establish correctness. Any future reproduction must keep that distinction.

Downloaded PDF SHA-256:

```text
SophOMR final:  ab5ca4e85b13a41ee685aa6a7b5218c5234871c6838e93b3ecd0c114df59246a
Instant final:  368a18f16654e5a357b1fb16603102f4405dd5f1427eda5e1ecfd24e806804f4
OSig final:     651ea68308f847fd250e25a86edf3d95b875304cddc235ee4a611ab1a7183cac
SPIRIT final:   8e2fab0e0970e6cae267f2e73b90fc762ca243126ea1fcaab161474fcc5a2998
YPIR:           25e7ef02f4ec51ddae289f6d6ed19fb2a688023966c56b794d10c765a5173ada
Plinko:         09f872747e26f0435712fdc1fce9b9da096a6cf3670487bb41c2eb84b0bf7cad
Stacked IBLT:   893a70be4c6312b66314947c342c3493ce6b59fa9b238da5e503f5bd65df6a56
Active PS:      52b5da5e36ef663bc97360d207bd225fa299a55db5b4f1af4db732e21c6327e9
```
