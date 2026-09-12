# UnifOMR evidence and integration assessment

Review date: 2026-09-12. Scope: the full 56-page paper, the implementation URL it identifies, and the public VecBatchPIR backend. Production source was preserved. This is source/document analysis, not a reproduced UnifOMR benchmark or a new cryptographic audit. Downloads and scratch calculations were kept in `/tmp/mudra-optimality-20260912`; the immutable sources and their hashes below allow retrieval without that temporary directory.

## Main conclusion

UnifOMR is a plausible way to make a wallet's **private notification discovery much cheaper**. It does not eliminate work proportional to chain history: it replaces downloading/decrypting full clues with a compact linear stream of encrypted partial-decryption results, then privately retrieves the matching payloads. A detector still processes all clues in the requested board for each recipient. Most importantly for a reliable node/wallet, its privacy guarantee against a malicious detector does **not** establish complete or correct recovery from that detector.

It belongs in an optional, replaceable notification/retrieval service. It does not replace encrypted notes, spend authorization, nullifier tracking, authenticated state, membership witnesses, data availability, finality, or consensus. Integrating it safely into Neptune/Cyber would require work at those boundaries, and a publicly reproducible implementation first.

## Pinned evidence

- Paper: Ben Fisch, Zeyu Liu, Eran Tromer, Yunhao Wang, *UnifOMR: Oblivious Message Retrieval with Near-optimal Concrete Efficiency*. Paper date May 9, 2026. ePrint metadata says received May 9, approved May 10, ACM CCS 2026. [Metadata](https://eprint.iacr.org/2026/910).
- Version history contains one PDF version, `20260509:014621`. [Version listing](https://eprint.iacr.org/archive/versions/2026/910), [immutable PDF](https://eprint.iacr.org/archive/2026/910/1778291181.pdf).
- Current and versioned PDFs both SHA256 `59612f4139a54b563e45aed94b4744395a1531dedd2bcd189a33df5a84f552b1`; 11,787,443 bytes, 56 pages. Local files: `unifomr.pdf`, `unifomr-versioned.pdf`, and extracted `unifomr.txt` in the scratch directory, not committed here. Tables 1 and 2 were also rendered and visually inspected. The ePrint record licenses the paper CC BY; this report attributes and analyzes that version.
- Paper reference [77] and §7.2 identify `ZeyuThomasLiu/ObliviousMessageRetrieval`. Its public default/master HEAD is **`816a05c2195a36b0efec8f1e1c79d3f8abbbf357`**, April 15, 2022. Its only other advertised branch, `lib`, is **`9d696a4803931fe13bbdecc0d2b891c329636abe`**, also April 15, 2022. Both are old OMR, not identifiable UnifOMR. [Pinned README](https://github.com/ZeyuThomasLiu/ObliviousMessageRetrieval/blob/816a05c2195a36b0efec8f1e1c79d3f8abbbf357/README.md).
- The author's current publication page links UnifOMR to the paper, without a separate code link. GitHub repository search for UnifOMR and the author's repository list did not reveal the new artifact. **No runnable authors' UnifOMR artifact found through those checked public routes** is the defensible conclusion; this is not a claim that none exists anywhere. [Author publication page](https://zeyuthomasliu.github.io/).
- Batch PIR backend: `mhmughees/vectorized_batchpir`, HEAD **`1a81869e28ca2258f07e32feef063413633e3bae`**, June 22, 2023. [Pinned source](https://github.com/mhmughees/vectorized_batchpir/tree/1a81869e28ca2258f07e32feef063413633e3bae). This is the upstream scheme named by the paper, not a pinned copy of the authors' modifications or experiment scripts.

## Protocol and what replaces wallet scanning

Paper §§3.1, 6.1–6.3, Algorithm 1 (pp.10–11, 28–33):

1. Recipient publishes a **clue public key**; sender encrypts a zero vector with the RLWE clue encryption scheme and attaches that clue to the independently encrypted message/note payload.
2. Recipient creates a fresh, independent BFV keypair and encrypts the coefficients of the clue secret under it. These encryptions form the **detection key** uploaded to the detector. The detector gets neither plaintext clue secret nor note spending key.
3. Detector scans every clue in the chosen board, homomorphically performs the linear part of clue decryption, and packs many results into BFV polynomial coefficients. It omits the expensive homomorphic range-check circuit. Modulus switching makes the response smaller.
4. Recipient decrypts those packed results and performs the range checks locally, learning candidate message **indices**. This still entails work linear in the number of records, but a tiny, SIMD-packed representation rather than N expensive full clue decryptions.
5. Recipient sends a padded fixed-capacity batch PIR query for those indices. The detector answers without learning which indices were requested. Recipient decrypts/authenticates actual payloads and rejects false positives.

The construction therefore requires **two request/response rounds** with its one-round PIR backend. Ordinary PIR alone cannot do step 3: initially the wallet does not know which record indices are its own. [Scheme and proof, pp.28–32](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=28).

At fixed security parameters the detection part sends approximately `2 * ell * N * log2(Q')` bits, plus rounding/serialization. The paper's parameters imply about **6.5 bytes per board record** for Param1 and **10 bytes** for Param2, before PIR. These calculations match the detection digest sizes 3328 KB and 5120 KB at N=2^19 when KB is read as KiB. Thus “wallet no longer scans the chain” is an imprecise slogan: it no longer downloads/decrypts every complete notification, but still consumes a linear compact stream.

## Threat model, unlinkability, quantum scope

| Property | Supported scope | What it does not establish |
|---|---|---|
| Recipient privacy | Computational, against malicious/colluding other parties under the paper's transcript model | Anonymous network transport, hidden IP/session/billing identity, hidden requested epoch/window or public capacity |
| Correctness/completeness and false positives | Honest-but-curious parties for the base construction; small per-message error probabilities; bounded pertinent count | Malicious detector returning every correct result or even processing the canonical board |
| Strong detection-key unlinkability | Detection key cannot be linked to one of two clue keys even when the adversary knows clue-key generation randomness; fresh detection keypair is independent | Operational unlinkability if registration, connections, account IDs or key reuse reveal relationships |
| Full-key unlinkability | Paper remarks that fresh clue/detection keys can be regenerated without cryptographic association | Automatic discovery of an unbounded sequence of old derived addresses/keys after seed restore |
| Payload security | Payloads are assumed separately encrypted; application can discard false positives after decryption | Payload confidentiality/authentication provided by OMR itself; proof that a clue matches a valid note |
| Quantum security | RLWE-based clue encryption and BFV; overall privacy additionally requires a suitable PIR backend | A separately quantified end-to-end quantum security level, or security of unrelated signatures/consensus |

The formal definitions quantify PPT adversaries; the paper does not give a separate QPT theorem or distinguish classical versus quantum bit estimates in its table. Param1's advertised computational level is 110 bits; Param2's is >128 bits, attributed to LWE-estimator. These are parameter estimates, not an independent validation of the exact distributions, proof implementation, backend, and entire wallet stack. [Definitions and assumptions, pp.10–21](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=10), [Theorem 6.1, pp.31–32](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=31).

The security proof uses a fixed-size second-round query even following an adversarial first response. A wallet must not introduce a response-dependent visible abort, count, direct fallback request or payment action without analyzing the new leakage. Appendix E is explicit: single-server **integrity remains open**, and its privacy is “CPA-like” because post-retrieval recipient behavior is excluded. It discusses stronger CCA-like security and IND-CPA-D hazards as future work. This is a material qualification of “malicious server privacy,” not a contradiction of the stated theorem. [Appendix E, p.56](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=56).

The paper mentions a prior compiler adding resistance to malicious senders/recipients at about 20% overhead. Its main evaluated construction uses the standard model; that statement does not make the reported timings a verified implementation of the stronger construction. The cited extension also does not solve malicious-detector omission.

## Costs, parameters and what was actually measured

Table 2's representative task is **one recipient retrieval**, N=524,288 total messages, true pertinent count and stated cap k=kbar=50, payload P=612 bytes. It is not 524,288 users, simultaneous requests, or a wallet with 50 accounts. The distribution/count of other recipients is not a benchmark parameter reported there.

| Metric | UnifOMR Param1 | UnifOMR Param2 |
|---|---:|---:|
| Configured computational security | 110-bit | >128-bit |
| Per-message false-negative target | 2^-30 | 2^-128 |
| Per-message false-positive target | 2^-15 | 2^-22 |
| Detector runtime, reported | 25 s | 41 s |
| Digest, paper's KB units | 4163 KB | 5955 KB |
| Recipient runtime, reported | 28 ms | 54 ms |
| Detection/PIR key material, paper's MB units | 31 MB | 48 MB |
| Clue public key | 2491 bytes | 2576 bytes |
| Per-message clue | 2477 bytes | 2565 bytes |
| Communication rounds | 2 | 2 |

These are author-reported results, **not reproduced here**. Table 2 and its header use KB/MB without formally defining decimal/binary units; the detection-size arithmetic above indicates binary KB at least for that component. Do not equate “4 MB digest” with complete first-time restore communication: key upload is listed separately, and bootstrap/public-parameter handling is not fully specified in that number. [Tables 1–2, pp.35–37](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=35).

Hardware/reproducibility qualifications:

- §7.2 says Google Cloud **n2-standard-16 with 16GB RAM** for representative comparisons and **n2-standard-128 with 512GB** for VecBatchPIR at larger sizes. Google's documented n2-standard-16 has **64GB**, not 16GB; the paper's smaller-machine memory description is inconsistent. It does not pin CPU model/frequency or an image/toolchain in that passage. [Google machine table](https://docs.cloud.google.com/compute/docs/general-purpose-machines#n2_machine_types).
- Text frames comparisons as single-core/single-threaded; §D.4 calls parallelizing the combined scheme future work. An instance's 16/128 vCPUs must not be silently interpreted as the number of threads used. Whether all combined components were timed on identical hardware needs the unavailable experiment scripts.
- Authors report ≥10 repetitions, standard deviation <5%, and averages. Large-N SophOMR comparisons beyond 2^20 extrapolate measured smaller runs; not every quoted speedup comes from running the full large comparison instance.
- Parameter sweep: N=2^16…2^23 with k=500, P=612; k=125…16000 with N=2^21, P=612; P=612…3060 with N=2^21, k=500. Thus the 20×–1080× range mixes different settings and baselines. Batch PIR contributes about 50–92% of detector runtime.
- Latency tests in §D.4 emulate 30ms network delay and 1Gb/s; <8s at N=2^16, kbar=50 and <2s for one message are specific author-reported scenarios. The 25s bulk detector number is neither a mobile restore wall-clock guarantee nor sub-second incremental latency.

Sender/global-storage cost matters. For Param1, each 612-byte note carries a 2477-byte clue, roughly four times the payload size. At N=2^19 the payloads alone are 306 MiB; clues add **1.209 GiB**; their uncompressed sum is **1.508 GiB** before indexes, transport framing and homomorphic preprocessing. This arithmetic is in `unifomr-arithmetic.json`. Sender clue-generation latency is not tabulated in Table 2; no value was invented. The sender must obtain the recipient's roughly 2.5KB clue public key, generate randomized RLWE ciphertext correctly and make the clue durably available.

## Storage, preprocessing and the public PIR implementation

The detector stores the public board and per-client detection keys. Common clue transformations into NTT form can be reused across recipients; the paper excludes that shared NTT work from one formula for the per-recipient core calculation. This is amortization of preprocessing, not an elimination of per-recipient processing. More recipients require more encrypted linear evaluations and PIR requests; key storage also grows with registered recipients. Exact deployment memory cannot be inferred solely from serialized board/key sizes.

The chosen backend is **VecBatchPIR**, not PIRANA, SealPIR, Piano or YPIR. §D.3 says PIRANA had implementation errors on some desired parameters and OCC code was unavailable, explaining the choice. A different backend may change communication, memory, latency and security assumptions. §E deliberately excludes client-preprocessing PIR that would require the wallet to first stream the whole database or use heavy FHE. [Preprocessing, p.33](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=33), [backend choice, pp.52–55](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=52).

The public upstream VecBatchPIR source demonstrates costs omitted by a simplistic “only query plus answer” model:

- Server constructor populates raw data, hashes/replicates into buckets, balances them, and prepares PIR servers. [batchpirserver.cpp:3–20](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/src/batchpirserver.cpp#L3-L20).
- It iterates over the whole database and places records into candidate buckets; default is three hash functions and cuckoo factor 1.2. [hashing:90–104](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/src/batchpirserver.cpp#L90-L104), [parameters](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/header/database_constants.h#L8-L14).
- Per-bucket server preparation rounds, converts/merges, and NTT-preprocesses the database. [server.cpp:19–35](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/src/server.cpp#L19-L35).
- Demo transfers a full public index-to-bucket map from server to client before measuring query/response. That map is O(N) metadata in this implementation. Publicly reproducible hashing may permit different handling, but the actual wallet bootstrap must account for it. [main.cpp:178–207](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/src/main.cpp#L178-L207), [map generation](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/src/batchpirserver.cpp#L97-L104).
- Demo data are synthetic and scenarios fixed; its existing correctness checks are not blockchain/authenticated-note tests. Its README explicitly describes the implementation as research code not vetted for production. [README:28–55](https://github.com/mhmughees/vectorized_batchpir/blob/1a81869e28ca2258f07e32feef063413633e3bae/README.md#L28-L55).

These are observations about the pinned **upstream** backend. Without the authors' UnifOMR artifact, they cannot be asserted to be byte-for-byte their implementation or exactly the costs included/excluded in their table.

## Capacity, false positives and safe recovery

Keep distinct quantities: `T` true pertinent messages, `F` false-positive candidates, and `B` the actual padded batch PIR capacity. For lossless candidate retrieval, require `T + F <= B`, in addition to individual detection correctness and PIR correctness.

The paper's Definition 3.1 completeness statement (p.12) conditions on the count of true pertinent messages not exceeding kbar. Algorithm 1 line 27 (p.30) pads candidate set S to kbar, but randomly truncates it if it is larger. Table 2 labels k=kbar=50. §6.3 separately recognizes a PIR workload proportional to `k + N*epsilon_p`. Together these leave a **capacity-accounting ambiguity** in the printed pseudocode/benchmark presentation. It was not possible to resolve exactly how their experiment allocates false-positive slack because the new artifact was not found. This review does **not** label the paper or its implementation broken based on that ambiguity.

Useful independent arithmetic: at N=2^19, T=50, Param1's stated bound gives `E[F] <= (N-T)*2^-15 ≈ 16`. This is an upper bound, not a measurement or a claim that exactly 16 false positives occur. Under an additional model of independent false positives occurring at that bound, at least one false positive has probability about 0.999999887; this conditional calculation illustrates why B=T needs special justification. It does not establish the actual distribution of the authors' experiment.

An integration must allocate a separate overflow failure budget

`delta_capacity = Pr[T + F > B]`

and an aggregate recovery budget including detection failures, PIR failures and completeness guarantees. A per-note epsilon_n is not automatically a whole-wallet failure bound; a basic union bound contributes at most `T * epsilon_n` before the other terms. If an independent-binomial approximation is used for capacity, justify it and use a tail bound, not only expected F. Without independence, marginal bounds still give an expectation and weaker Markov-type bounds.

Never silently discard candidate notes and mark a range completely restored. Options include fixed public capacity with conservative slack, predetermined partitioning into smaller epochs, or a padded overflow protocol whose extra traffic has an explicit leakage policy. After authenticated payload decoding the wallet rejects false positives. A visible error-dependent retry/fallback can change the privacy model, so correctness and transcript privacy must be designed together.

## Seed restore, ongoing updates, and Neptune/Cyber integration

**Full seed restore:** deterministically recover the same historical clue secret keys/derivation versions from the seed, generate fresh independent detection keys, and process all relevant historical authenticated epochs since a conservative wallet birthday. Old detection-key cache is an optimization, not cryptographically required. Unknown birthday/derived-address history still requires a discovery strategy. Every historical note needs a recoverable clue and payload. An OMR add-on cannot retroactively discover old note formats without clues while maintaining the same protocol guarantees.

**Ongoing updates:** persist the verified last epoch/canonical block hash, restored notes, keys and synchronization status; retrieve only newly appended epochs. A detector may preprocess clues and partial digests while the wallet is offline. Nonetheless the client must come online to decode candidate indices and send the second-round PIR query. Appendix D.4 explicitly notes that with millions of accumulated messages this remaining PIR step can take tens of seconds; UnifOMR is not the same streaming-latency trade-off as InstantOMR. Small-window padding and update frequency trade bandwidth/server cost against timing leakage.

**Authenticated state and spendability:** discovering an encrypted note is only the first step. The wallet must also authenticate its inclusion in a canonical state, obtain/update its membership witness, check its nullifier/spent status, apply consensus/finality rules and keep durable rollback information. OMR does not inherently maintain a Merkle tree, mutator set, chain light client or account balance. Payloads can carry needed data, and witnesses can be fetched privately, but that is a further protocol with its own sizes and trust assumptions.

**Completeness and data availability:** authenticating each returned note with a Merkle path proves that returned notes exist; it does not prove that no pertinent note was omitted. Even binding requests to a published board root and N does not make the detector's evaluation honest. A production design needs a verifiable complete evaluation or another explicitly stated trust/availability model and recovery path. Multiple servers or selective rescans can be useful mitigations, but are not automatically a proof of completeness.

**Reorganizations:** bind epochs, clue/payload indexes and replies to chain identity, block range and canonical root. Handle reorgs by rolling back recovered state and replaying the replacement suffix. Reusing a numeric record index across two forks without authenticating its root is insufficient. These are integration requirements inferred from blockchain use, not features supplied by the paper.

**Sender/transaction compatibility:** define a versioned, bounded clue format, clue key distribution and seed derivation, authenticated encrypted note format, and durable consensus/data-availability commitment to the clue. Decide whether the protocol enforces correct clue-to-note binding, relies on honest senders, or supports an alternate recovery path. The paper's clue generation is independent of payload and its baseline correctness assumes honestly generated clues. No existing Neptune/Cyber transaction automatically gains discoverability from deploying a detector.

**Boundary with Mudra:** UnifOMR should not change signature verification, key ownership, spend authorization, ledger execution or consensus. Its favorable connection to post-quantum designs is the lattice-based notification privacy machinery, not a replacement signature primitive. Baseline trial decryption remains a reference recovery method where its data and keys are available; remote detection is an optimization until equivalent correctness/completeness is established.

## How strong is “near-optimal”?

Theorem 5.1 constructs PIR using strongly detection-key-unlinkable OMR with essentially the same online runtime and communication, plus public preprocessing. That is a relative primitive reduction under a specified property/model. Its stated security reduction has a 1/N^2 loss; direct equally tight batch-PIR reduction is left open. Appendix E also explicitly leaves proving that PIR does not imply OMR open, despite stronger informal language earlier in the paper.

The 50–92% batch-PIR runtime fraction says UnifOMR overhead is close to the **chosen backend's current measured cost**. It does not prove universal concrete optimality across different PIR schemes, preprocessing models, multi-server trust, hardware, streaming workloads, or weaker unlinkability requirements. A future faster backend can change which component dominates. [Reduction and qualifications, pp.22–25](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=22), [open questions, p.55](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=55).

## Executed checks and reproduction limits

Executed: downloaded current/versioned PDFs with `curl`; checked identical `shasum -a 256`; `pdfinfo`; full text extraction with `pdftotext -layout`; section/algorithm/proof/appendix reads; table render with `pdftoppm` and visual inspection; shallow cloned the linked OMR and VecBatchPIR repositories; inspected both advertised OMR branches via `git ls-remote`/`git fetch`; recorded commit SHAs with `git log`; checked the author's site and public repository list/search; inspected backend preprocessing, public-map exchange and fixed demo scenarios. A bounded Python arithmetic check wrote `unifomr-arithmetic.json`.

Attempted backend configuration:

```sh
cmake -S /tmp/mudra-optimality-20260912/unifomr-batchpir \
  -B /tmp/mudra-optimality-20260912/unifomr-batchpir-build \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
```

It reached `find_package(SEAL 4.1 REQUIRED)` and failed because SEAL 4.1 is not installed. No test executable or timing benchmark was run. Installing a separate cryptographic build stack to run the old backend demo would still not reproduce UnifOMR without the missing new artifact and experiment parameters. Reported performance remains the paper's evidence, clearly separated from our executed checks.
