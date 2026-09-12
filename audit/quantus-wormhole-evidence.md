# Quantus Wormhole privacy: source review, 2026-09-12

Research only. No Cyber source changes, on-chain transactions, proof benchmarks, or new cryptographic audit were performed. This is a review of public primary sources and pinned source code; “implemented” below does not mean the reviewed WASM was independently matched against a live node.

## Revisions and deployment boundary

| Source | Pinned revision | Meaning |
|---|---|---|
| chain main | `662ef6d1dea8f3572776b8db9da1f57283744fd2` | Current checked-out implementation |
| chain v1.0.1 | `f1176cea6a6d08ea437710dcd45cae6717b773df` | Released source; relevant runtime, Wormhole, frame-system and dependency manifest files have zero diff against reviewed main |
| qp-zk-circuits main | `e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99` | Workspace version 4.4.0, qp-plonky2 =1.5.6 |
| qp-zk-circuits v4.3.0 | `b224e6dcbb89c03f90fbd26f68674c3b557ef36f` | Chain pins Wormhole crates =4.3.0; reviewed statement and wrapper constraints match main |
| improvement-proposals main | `b19530aff9cd82d4d49f9cc45f3463db1e2c4da7` | QIP5 still explicitly Draft |

**Do not say that “mine-only” establishes disabled transfers/privacy.** The September 8 announcement says mining is the only way to *get* QTC at launch and that liquidity/trading follows weeks later. It does not announce a runtime call restriction. [Official announcement](https://www.quantus.com/blog/weekly-update-09-08-2026/).

The current runtime and release source include Wormhole at pallet index 20 without `disable_call` or `disable_unsigned`. Their system config inherits `SolochainDefaultConfig` and does not override `BaseCallFilter`; the vendored default is `Everything`. Both private- and public-batch unsigned dispatch methods exist. This is positive evidence of enabled **release source**, not live-chain execution evidence. No live runtime-code hash or successful mainnet privacy transaction was verified in this subtask.

Sources: [runtime registration](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/runtime/src/lib.rs#L301-L308), [config inheritance](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/runtime/src/configs/mod.rs#L101-L138), [Everything default](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/frame-system/src/lib.rs#L465-L466), [release registration](https://github.com/Quantus-Network/chain/blob/f1176cea6a6d08ea437710dcd45cae6717b773df/runtime/src/lib.rs#L301-L308), [dispatch methods](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L583-L653).

The version distinction matters: chain [Cargo.toml:279-284](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/Cargo.toml#L279-L284) pins 4.3.0, whereas circuits [Cargo.toml:23-41](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/Cargo.toml#L23-L41) is 4.4.0. `git diff v4.3.0 HEAD` shows no changes to the leaf circuit, public-input layout, private/public wrapper constraint files, or Wormhole verifier implementation. Main adds ownership/airdrop circuits and changes proving/parsing paths/dependencies; those newer paths cannot automatically be attributed to chain 4.3.0.

## What is actually proved

For a real leaf the witness contains a secret, recipient-specific transfer count, a deposit leaf, Merkle siblings/positions/depth, and block-header preimage. The circuit binds:

1. Deposit recipient to `H(H(wormhole_salt || secret))`.
2. Nullifier to `H(H(nullifier_salt || secret || transfer_count))`, using the same secret and count as the deposit.
3. Deposit leaf `(recipient, count, asset, quantized input amount)` to the 4-ary Poseidon tree root.
4. That root to the header preimage, and the header to the public block hash.

On-chain checks then require native asset 0, the configured fee (currently 4 bps), a stored nonzero block hash matching the given height, unused distinct nullifiers, and proof verification under the embedded verifier. The recent-hash retention setting is 4096 blocks. These layers are necessary together: an arbitrary valid Merkle tree by itself is insufficient, and the circuit by itself does not track spent state.

Sources: [shared witness/root/header/nullifier bindings](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/circuit/src/circuit.rs#L243-L331), [leaf structure](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/circuit/src/zk_merkle_proof.rs#L42-L124), [chain validation](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L1125-L1200), [retention](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/runtime/src/configs/mod.rs#L76-L78), [fee](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/runtime/src/configs/mod.rs#L810-L815).

The leaf has **22 public field elements**: asset, two output amounts, fee, nullifier (4), two exit addresses (4 each), block hash (4), height, and individual input amount. The leaf is deliberately **non-ZK**. It is an intermediate proof intended to remain in the trusted local/private proving boundary; publishing it or sending it to an untrusted prover is not the protocol's privacy-preserving workflow.

[Exact public-input layout](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/inputs/src/lib.rs#L25-L43), [non-ZK leaf and ZK wrapper configs](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/common/src/circuit.rs#L372-L424).

**Value conservation is currently enforced across the entire PrivateBatch segment**, not separately per leaf:

`sum(outputs) * 10000 <= sum(authenticated real inputs) * (10000 - fee_bps)`.

The intermediate individual input amounts and their total are hidden by the PrivateBatch wrapper and not forwarded. The implementation masks dummies, bounds the fee complement, and range-checks the field difference to exclude negative modular wrap under the 64-leaf bound. Real nullifiers must be pairwise distinct inside the circuit as well as in chain settlement. QIP5 still describes the older per-leaf inequality and calls input amount private, so QIP5 is not an exact description of the current leaf interface.

[Actual aggregate fee constraints](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/private_batch/circuit/circuit_logic.rs#L335-L371), [in-circuit nullifier uniqueness](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/private_batch/circuit/circuit_logic.rs#L407-L435), [draft specification](https://github.com/Quantus-Network/improvement-proposals/blob/b19530aff9cd82d4d49f9cc45f3463db1e2c4da7/qip-0005.md#L1-L55).

## Privacy boundaries and batching

| Observation / adversary | Hidden by the intended local PrivateBatch workflow | Still visible / limitation |
|---|---|---|
| Passive chain observer | Explicit link from a spent deposit to its exit; spend secret and inclusion path | Deposit recipient, amount, time and ordinary account balance; exit addresses, amounts, submission time, anchor hash/height and nullifiers |
| Individual leaf receiver | No ZK privacy guarantee at this layer | Individual input amount is explicitly public; leaf is non-ZK, so treat its entire proof as sensitive |
| PublicBatch aggregator | Private witness and mapping from deposits to exits, conditional on the ZK proof's security | Whole PrivateBatch public inputs, client batch boundaries, arrival metadata/IP absent separate protection |
| Amount/timing analyst | Cannot simply read a source-address field from the proof | Amount matching, repeated addresses, known payments, mining patterns and low traffic can narrow candidates; no measured worst-case anonymity guarantee established |
| Observer counting inputs | Real versus dummy nullifier slots are hidden by random dummy nullifiers and local permutation | Fixed capacity and output sparsity remain visible; no guarantee of seven unrelated users, equal plausible candidates, or hidden output amounts |
| Network observer or RPC operator | Circuit does not provide transport anonymity | Requested account/leaf identifiers and submission/network timing are outside the circuit statement; wallet/network path requires separate analysis |

Deposits are not confidential notes whose values are hidden on-chain. The [zk-tree stores whole leaves publicly](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/zk-tree/src/lib.rs#L231-L261). On-chain account balances also remain visible. A spent Wormhole source is not debited, so its visible balance is not the owner's remaining spendable private balance. Recovery must determine which derived deposit nullifiers have been spent; the wallet-side agent covers that mechanism. The [September 9 whitepaper](https://www.quantus.com/whitepaper/v0.4.1/#private) itself explicitly acknowledges visible balances and amount/timing inference.

PrivateBatch is the **one ZK layer**, configured for row blinding. It recursively verifies the local leaves, pads them, shuffles leaf placement, sums matching exit addresses, zeros duplicate slots, replaces dummy nullifiers with hashes of fresh random preimages, and separately permutes the output nullifier list. Thus a public nullifier position is not a public leaf/exit position. Real leaves share one recent header; the cumulative tree allows an older deposit to be witnessed at that header, so the public header need not reveal the deposit's creation block.

[Canonical config](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/common/src/circuit.rs#L382-L403), [shuffle and independent permutation](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/private_batch/prover/lib.rs#L354-L380), [exit deduplication and nullifier routing](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/private_batch/circuit/circuit_logic.rs#L373-L467), [root finalized into header](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/zk-tree/src/lib.rs#L302-L311).

**PublicBatch is aggregation for cost, not a second anonymity mixer.** It is non-ZK, takes already-ZK PrivateBatch proofs, and forwards their exits/nullifiers in order, preserving each client segment. Default chain build is **7 leaves/private batch and 53 private batches/public batch**, both build-time configurable. Multiplying 7 by 53 yields the supported batch capacity, not 371 equally plausible senders for every payment. PrivateBatch may contain several deposits owned by a single wallet. Even within it, distinct non-deduplicated output slots retain the two-slots-per-leaf layout; this is another reason not to claim that every output association is erased.

[Default build dimensions](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/build.rs#L31-L52), [PublicBatch forwarding](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/public_batch/circuit/circuit_logic.rs#L244-L316), [chain reconstitutes segment boundaries](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L549-L578).

### Knapsack / privacy-score caveat

QIP5's [fuzzy-knapsack passage](https://github.com/Quantus-Network/improvement-proposals/blob/b19530aff9cd82d4d49f9cc45f3463db1e2c4da7/qip-0005.md#L278-L285) is an intuition/analogy, not a reduction proving this transaction distribution is hard to deanonymize. An NP-hard worst-case subset problem does not establish hardness for the actual low-dimensional, correlated, fee-constrained, time-stamped payment instances. This is an analytical inference, not a demonstrated attack. No numeric privacy-score implementation was found in the reviewed circuit/QIP sources. Any wallet/explorer score must be evaluated separately for its model, assumed candidate set, side information, and calibration; do not report it as cryptographic security bits or a probability of anonymity. The [formal spec explicitly leaves transaction unlinkability/game-based security open](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/formal/SPEC.md#L208-L213).

## Spend uniqueness, change and settlement caveat

The nullifier is per `(secret, transfer_count)`, not per withdrawal amount or destination. Changing destination/amount cannot authorize a second spend of the same deposit. A wallet must include desired change among the same batch's outputs, potentially to a fresh Wormhole address which creates a new deposit leaf. Leaving entitlement unallocated does not permit later reuse of its old nullifier. Private segments are settlement units: chain rejects a segment with repeated/already-used nullifiers while it may settle other valid segments in a PublicBatch.

[Persistent and intra-bundle uniqueness](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L670-L725), [segment settlement semantics](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L763-L778), [credit paired with new leaf](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L1323-L1356).

A concrete correctness caveat visible in current/released code: it marks nullifiers spent first; a failed exit credit (e.g. below existential deposit) is skipped with `ExitMintFailed`, retaining the spent nullifier. The whole requested value is not retriable after that. This behavior is explicit implementation, not a newly proven exploit. Wallet preflight matters. [Marking](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L804-L814), [failed credit behavior](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/pallets/wormhole/src/lib.rs#L857-L877).

## Evidence strength: real engineering, not a completed security theorem

There is substantial implemented constraint/settlement logic and adversarial regression coverage. Source contains aggregate-fee and maximum-width wrap regressions, wrong block/fee rejection, dummy-nullifier tests and invalid-permutation tests. Those tests were **inspected, not executed** in this subtask. [PrivateBatch regressions](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/aggregator/src/private_batch/circuit/tests/circuit_logic.rs#L441-L690).

The public [EigerWormholeAudit.pdf](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/audits/EigerWormholeAudit.pdf) is **Draft Report v0.1, 2026-03-20**. Executive summary: 35 findings, no critical finding, one high (unsigned-proof resource exhaustion), one medium (miner fee derivation), four low and informational findings. It reviewed chain `f838f70…`, circuits `2246b4e…`, Plonky2 `df2367f…` (PDF pp.6-7), not September main. Its remediation page (p.8) is explicitly awaiting fix commits. Current source does show concrete subsequent defenses such as a proof byte cap, exact encoding check, cheap pool checks and full pre-dispatch verification. This supports “there was review and there are implemented repairs”; it does not support “auditors revalidated every September change.” The later May Substrate/node audit is a different scope and must not substitute for a current Wormhole cryptographic audit.

Formal verification is explicitly partial. `Trusted.lean` contains `leaf_proof_sound` and `private_batch_proof_sound` **axioms**, with verifier/FRI/Fiat-Shamir/recursion soundness and field-level faithfulness still external obligations. `Security.lean` says probability bounds and a knowledge extractor are out of scope. `Hash.lean` calls out that its `CollisionResistant` assumption is perfect injectivity, stronger than computational collision resistance; useful deterministic reduction lemmas should not be advertised as a full concrete cryptographic security proof.

[Trusted obligations](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/formal/WormholeSpec/Trusted.lean#L60-L92), [security scope](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/formal/WormholeSpec/Security.lean#L1-L23), [perfect-injectivity caveat](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/formal/WormholeSpec/Hash.lean#L69-L80).

The canonical profile has a **100-bit minimum configured security floor**. It is not evidence for a universal 128/256-bit end-to-end or quantum privacy guarantee. Node batch verifiers are generated at build time from pinned circuit dependencies; generation environment/toolchain/artifact origin remain trust boundaries. The threat model explicitly requires locally rebuilt prover data and trusted artifact generation, with network proof senders untrusted. [Security floor](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/inputs/src/lib.rs#L40-L43), [artifact trust model](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/wormhole/THREAT_MODEL.md#L13-L80).

## Performance claims

No timing measurement or proof-generation benchmark was run. The September 9 whitepaper reports roughly **266 KB for a 371-transfer PublicBatch** and derives roughly **430 QTPS** from block-space assumptions; treat those as the authors' stated capacity figures, not a reproduced sustained end-to-end benchmark or observed mainnet throughput. Public-input count and build dimensions are inspectable, while device proving latency, memory, arrival rate and network conditions are separate constraints. [Whitepaper scaling section](https://www.quantus.com/whitepaper/v0.4.1/#scalable).

## Commands and checks actually executed

All repository reads used the owned temporary clones under `/tmp/quantus-review-20260912`.

```sh
git clone --depth 1 https://github.com/Quantus-Network/chain /tmp/quantus-review-20260912/chain
git clone --depth 1 https://github.com/Quantus-Network/qp-zk-circuits /tmp/quantus-review-20260912/qp-zk-circuits
git clone --depth 1 https://github.com/Quantus-Network/improvement-proposals /tmp/quantus-review-20260912/improvement-proposals
git rev-parse HEAD
git log -1 --format='%H%n%cI%n%s'
git status --short
git ls-remote --tags origin '*4.3*' '*4.4*'
git fetch --depth 1 origin tag v4.3.0
git diff --stat v4.3.0 HEAD
git diff v4.3.0 HEAD -- wormhole/circuit/src/circuit.rs common/src/circuit.rs wormhole/aggregator/src/private_batch/circuit/circuit_logic.rs wormhole/inputs/src/lib.rs wormhole/verifier/src/lib.rs
git ls-remote --tags origin 'v1.*'
git fetch --depth 1 origin tag v1.0.1
git diff --stat v1.0.1 HEAD -- runtime/src/configs/mod.rs runtime/src/lib.rs pallets/frame-system/src/lib.rs pallets/wormhole/src/lib.rs Cargo.toml
git show v1.0.1:runtime/src/configs/mod.rs
git show v1.0.1:runtime/src/lib.rs
git show v1.0.1:Cargo.toml
pdftotext -layout audits/EigerWormholeAudit.pdf /tmp/quantus-wormhole-audit.txt
pdftotext -layout audits/quantus-substrate-audit-20260513.pdf /tmp/quantus-chain-audit.txt
```

Additional actual reads were `rg` searches, `nl -ba ... | sed -n ...` line inspection, and web opens/searches of the official repositories, September 8 announcement, mining guide and September 9 whitepaper. All three clone working trees were clean after review. No cargo tests, Lean checks, cryptanalysis, network transaction submission, benchmark, or live on-chain code comparison was performed.
