# Local signature decisions recovered from source and history

Research snapshot: 2026-09-12. Source inspection and tests did not change production files or lockfiles. This is a bounded reconstruction of recorded decisions, not a new signature selection or cryptographic certification.

## Finding

There really was an explicit ML-DSA choice in project documents. It was subsequently removed, explicitly, in favor of proof-based authentication. The surviving native target is **Hemera(secret) as identity + a Zheng proof of a programmable lock condition**. The current implemented reusable signing primitives are **secp256k1 ECDSA / ADR-036**, for migration and browser compatibility. The newly accepted neuron working specs preserve that existing profile rather than claiming the proof-native target is already implemented.

The records do not support saying "we already finalized Falcon/ML-DSA/SLH-DSA as Cyber's optimal production signature." They do support saying "we considered ML-DSA, then deliberately chose to have proofs subsume signatures." No benchmark or proof establishing global optimality accompanies that architectural choice.

## Exact chronology

Historical references use `repository commit:path:line`; retrieve with `git -C REPOSITORY show COMMIT:PATH | nl -ba`. These are historical file locations, not current workspace paths.

| Date (UTC) | Recorded decision | Exact evidence | Present status |
|---|---|---|---|
| Before 2026-03-02 | CyberPatch document specified ML-DSA / Dilithium3, BLAKE3-derived neuron ID and Kyber768 transport. | `cyber 035bb5dcec3b74ad5a107ef7f2689dde12050e89:graph/cyber/patch/spec.md:235`, identity at line 251. | Superseded document; do not repeat its parameter/security numbers as current facts. |
| 2026-03-02 03:50 | Remove CyberPatch's independent BLAKE3/Dilithium choices; inherit protocol crypto. | `cyber 32961eb07ca0cedd8ac15fa33345136ab5e74f95`, title: `fix: align patch spec crypto to project stack — drop BLAKE3/Dilithium`. Commit body explicitly says the independent primitives conflicted with the project's choices. | Explicit rejection of that subsystem's standalone choice. This replacement still deferred the concrete signature scheme to protocol level. |
| 2026-03-02 16:04 | Introduce signatureless native identity: Hemera(secret), proof of preimage knowledge, programmable lock scripts. | `cyber df8dfa3cf56d23e423715afe26191d857f6c042c:graph/cyber/identity.md:9`; comparison table line 22; programmable locks line 63; rationale/tradeoff line 94. Commit title explicitly names signatureless authentication. | Earliest located explicit native target decision. |
| 2026-03-16 05:40 | Add Mudra `sig = ML-DSA` for P2P messages and validator attestations, while on-chain authentication remains proofs. | `cyber 8797ff52126a7f105dc692b12cf3f153ccee4960:graph/mudra.md:18`; lifecycle line 32; on-chain/off-chain distinction line 41. | Genuine documented proposal/selection, subsequently removed. No specific ML-DSA parameter set or implementation was introduced here. |
| 2026-03-16 05:51 | Remove `sig` and `vrf` from Mudra: proofs subsume both, including off-chain messages. | `cyber 815d41800edac99bb52f7b7bd467d02c01fe7efc:graph/mudra.md:14`; separation line 24; remaining module table line 31. Commit title: `docs: remove sig and vrf from mudra, proofs subsume both`. | Explicit supersession, eleven minutes after the ML-DSA entry. |
| 2026-05-12 | Consolidate identity into Cybergraph's canonical primitive documents. | `cybergraph c5803ac5e9a1b46d5cedd968bb7bc60d1f56ebb7:reference/identity.md:9`. | Relocation, not a new conventional signature selection. |
| 2026-06-10 | Move signatureless identity from Cybergraph into Mudra because it is a cryptographic primitive. | Cybergraph removal `1f7d901c0a175d3158dde7efb14e76892869bc7b`; Mudra addition `7ff861ee83b3d9a9167680af3d12b1b73c9ed603`. Both commit bodies explicitly describe the same hash-preimage identity. | Current `mudra/specs/identity.md` descends directly from this design. |
| 2026-07-03 | Add secp256k1 legacy-key bridge: existing Cosmos owner signs exact legacy-address → native-neuron binding once; native target remains proof-based. | Mudra spec commit `2ecdda4dbcd39a4d21b57c822e865748ef39fc8f`; implementation `c756e9f` (legacy-key bridge). Current `/Users/master/cyber/mudra/specs/bridge.md:11`, `:18`, `:55`, `:80`. | Implemented native signature verification; full claim proof planned. |
| 2026-08-06 | Add fresh domain-scoped secp256k1 identities and general ADR-036 signing for browser/lytics clients. | Mudra `6fcbc6f608f26b8d61ef415a2af3ddfdfeaa0608`; `/Users/master/cyber/mudra/src/domain.rs:6`, `:18`, `:29`. | Implemented, deliberately distinct from legacy migration. Commit says browser wallets/small JS libraries already speak secp256k1 and dependency cost matters. |
| 2026-08-11 | Change proof vocabulary to Zheng rather than generic STARK wording. | Mudra `ed69ed2d252174b6b9c50d2b20759f6991e21645`. | Terminology change, not proof completion. |
| Current concurrent work | `cyber-secp256k1-hemera-v1` is the supported native profile; proof-native subjects require an explicit separate profile. | `/Users/master/cyber/neuron/specs/identity.md:4`, `:21`; `/Users/master/cyber/soft3/specs/neuron.md:6`, `:23`, `:25`. | Files say `status: accepted`, but both are currently untracked in another ongoing user thread. They are working contracts, not historical committed evidence of a finalized PQ scheme. |

## What the surviving choice was optimizing

The rationale recorded in `/Users/master/cyber/mudra/README.md:7` and `/Users/master/cyber/mudra/specs/identity.md:18` is architectural:

- Reuse the hash/VM/proof stack already needed by Cyber; avoid a separate curve or lattice signature subsystem for native authentication.
- Express ownership, multisig, timelocks, delegation and recovery as lock programs (`identity.md:63`).
- Prove more than ownership: include state, stake, nullifier and application predicates (`identity.md:102`).
- Aim to amortize larger individual proofs through recursive composition (`identity.md:94`).
- Use the same proof mechanism for authentication, integrity, randomness and metering (`README.md:17`).

These are design intentions. The listed constraint counts/proof sizes and "only collision resistance" claim are not present-day qualification evidence. Preimage authentication also requires the appropriate hash preimage assumptions, knowledge soundness and witness privacy of the proof system, and exact message/domain binding. The spec's `64 raw bytes` address at `identity.md:43` is stale relative to the implemented 32-byte profile. `/Users/master/cyber/hemera/rs/src/params.rs:33` explicitly marks full-round security unestablished and inverse-16 experimental.

Falcon/FN-DSA, ML-DSA and SLH-DSA occur in the native identity comparison table (`identity.md:22`), but the conclusion is to replace a separate signature scheme, not to choose one row. `/Users/master/cyber/cybics/crypto/signatures.md:13` is an educational comparison, with the proof-based alternative at line 25; it is not a Cyber protocol parameter decision.

## Keep four meanings separate

| Meaning | Current evidence | Consequence |
|---|---|---|
| Native subject / NeuronId | `/Users/master/cyber/mudra/src/claim.rs:92`: Hemera(compressed public key); working neuron spec line 21 names its versioned secp profile. | H(pubkey) is an identifier, not authentication. H(secret) cannot silently replace existing bytes. |
| Key/address representation | `/Users/master/cyber/mudra/specs/bridge.md:36`; `/Users/master/cyber/mudra/src/domain.rs:44`, `:73`. | BIP39/32 Cosmos keys recover existing accounts; domain-derived browser keys serve a separate use case. Network, domain and display HRP are distinct. |
| Signature/proof authorizing an operation | `/Users/master/cyber/mudra/src/claim.rs:125`, `:137`, `:155`; working neuron spec line 45. | `verify_arbitrary` checks the signed bytes but explicitly leaves key→subject/address binding to the caller. An application must bind subject, action, network, policy/revision and replay context. |
| Request idempotency identifier | Concurrent `/Users/master/cyber/neuron/engine/src/neuron.rs:246`: domain `neuron/request-id/1` over neuron+nonce; authorization statement uses its own domain at line 198. | A request key deduplicates work; it is not a key, identity proof or signature. |

BBG does not select or implement a signature scheme: `/Users/master/cyber/bbg/rs/src/signal.rs:36` explicitly delegates semantic/authentication validation to Cybergraph. `/Users/master/cyber/bbg/specs/privacy.md:24` distinguishes private ownership proofs from the public `auth signature` path at line 29 without naming a family. A 64-byte intent signature field in `bbg/specs/native-state.md:56` likewise is not evidence of selecting a PQ signature.

## Zheng readiness relevant to that decision

1. **The old Mudra proving entry point is not native authentication.** `/Users/master/cyber/mudra/src/proof/prove.rs:18` says full claim assembly remains; line 82 proves only `a*b+c`; line 35 builds an unbound statement. The current `prove` feature does not compile because that initializer omits `bbg_root` (reproduced below). Native secp verification is available; a complete Zheng proof of the migration claim is not.

2. **Legacy Zheng `commit/verify` must not be used to conclude that the claimed identity/action was authorized.** `/Users/master/cyber/zheng/README.md:5` explicitly states these remain legacy statement checks. `/Users/master/cyber/zheng/rs/src/folding/fold.rs:343` and `:391` contain passing attack regressions: an all-zero witness and a genuine but unrelated quote witness respectively verify against arbitrary chosen statement values. Both tests still pass in this snapshot. Absorbing statement bytes into a transcript binds the proof to those bytes; it does not establish the missing relation between witness, program and statement. The documented residual is `/Users/master/cyber/zheng/specs/decider.md:111`.

3. **A newer public execution path does establish a bounded relation, but exposes the witness.** `/Users/master/cyber/zheng/rs/src/execution/statement.rs:93` derives the relation from the canonical program; `:119` pins public inputs/outputs/cost; `:153` serializes the program, inputs, outputs, cost and budget; `:241` rederives the relation at verification. `/Users/master/cyber/zheng/rs/src/execution/proof.rs:5` explicitly says this path discloses witness-dependent information. It cannot privately authenticate a reusable secret merely by running a preimage lock through the public API.

4. **Concurrent private-proof work is real and must not be omitted.** The untracked `/Users/master/cyber/zheng/rs/src/execution/private.rs:42` prepares the private relation and witness; `/Users/master/cyber/trisha/rs/ccs.rs:93` generates an actual randomized upstream Triton proof of a generated CCS checker, and `:119` checks the expected program/input/output before native proof verification. The untracked `/Users/master/cyber/joy/rs/zk_execution.rs:39` binds statement/build/program metadata; `:83` independently prepares the expected relation and checker. These are distinct from the legacy folded API. `/Users/master/cyber/zheng/specs/ccs-execution-backends.md:3` calls this an owner-authorized release-repair contract. This research inspected the path but did not run a Triton proof or certify it. It has not supplied a versioned Mudra identity/action authorization profile.

5. **The native authorization statement still needs a complete application contract.** `mudra/specs/identity.md:49` only shows a preimage assertion. Its anonymous-link sketch includes source/target/nullifier/state public inputs at line 108, but it is not an encoded operation authorization protocol with network and replay rules. The new neuron spec requires exact action/network/prog binding at line 45; the generic Zheng execution statement has no implicit notion of a neuron or network. A Mudra profile must explicitly put those meanings into the verifier's expected statement/program and reject substitutions. This is a concrete missing integration contract, not evidence that proof-based authentication is impossible.

## Commands and observed results

All commands used `--locked --offline` and a separate target directory; no source changes were made.

```sh
cd /Users/master/cyber/zheng
cargo test --locked --offline --target-dir /tmp/zheng-signature-research-target \
  -p zheng --lib attack_ -- --nocapture
```

Result: **2 passed, 0 failed, 175 filtered**. These are intentionally passing attack demonstrations, not evidence of production security:

- `folding::fold::tests::attack_zeroed_constant_wire_satisfies_universal_instance`
- `folding::fold::tests::attack_satisfying_but_meaningless_witness_passes_for_any_statement`

```sh
cargo test --locked --offline --target-dir /tmp/zheng-signature-research-target \
  -p zheng --lib execution::budget_tests::public_cost_budget_and_selected_branch_are_authenticated \
  -- --nocapture
```

Result: **1 passed, 0 failed, 176 filtered**. This current working-tree regression checks public execution claim binding; it is not a private identity proof test.

```sh
cd /Users/master/cyber/mudra
cargo test --locked --offline --target-dir /tmp/mudra-id-research-target \
  --features prove --lib --test vectors
```

Result: **exit 101**, Rust `E0063`, missing `bbg_root` in `Statement` at `src/proof/prove.rs:36`.

Earlier same-day native Mudra checks are preserved in [the readiness report](../mudra-identity-readiness.md): 18 library + 2 vector tests pass by default, 15 library tests without default features. That earlier review's external emulation probe is not an actual claim proof and must not be described as one.

## Search coverage and limits

Current source/docs and hidden plan files were searched in `mudra`, `soft3`, `cyber/research`, `bbg`, `zheng`, `crystal`, and `cybics`, excluding `.git`, build artifacts, dependencies and today's new Quantus audit. A supplemental current Markdown search in `cyb` found no conventional PQ-family matches. Scheme terms included Lamport signatures, Winternitz/WOTS, Falcon/FN-DSA, ML-DSA/MLDSA/Dilithium, SLH-DSA/SLHDSA/SPHINCS and hash-based signatures. The original broad Lamport hits were logical-clock/consensus references, not signatures.

Additionally, `git log --all -G... -- '*.md' ':(exclude)audit/**'` searched retained histories of `mudra`, `cyber` (all Markdown, including old graph/root locations), `bbg`, `soft3`, `zheng`, `crystal`, and `cybics`; relevant commits were opened with `git show`. `git log --all --follow -- specs/identity.md` traced Cybergraph→Mudra movement. This deeper history is what recovered the subsequently removed ML-DSA selections; a current-tree-only search would have missed them.

No surviving finalized Lamport/WOTS/XMSS/LMS/Falcon/SLH-DSA production selection was found in this coverage. Generic XMSS/LMS references occur in `/Users/master/cyber/cybics/crypto/features.md:146`. Today's `/Users/master/cyber/mudra/audit/quantus-identity-privacy.md` at commit `3dd6f3d` is new comparative research, not historical evidence that its conditional ML-DSA suggestion was already chosen.

## Snapshot heads and concurrent changes

| Repository | HEAD at inspection |
|---|---|
| mudra | `3dd6f3d2d7d71454fe702eefa2f85331ad9c459d` |
| cyber | `b1b0a9ec6f172eb267d8440cf31c798e72eee0f4` |
| cybergraph | `c9836aeb6dd215e58fc19cc04a90eabef76dff77` |
| bbg | `a25039d54d0671c70c11aed2c9efc8266c371e8b` |
| soft3 | `4a36269070315823b85b0698919a59011f2db6f6` |
| neuron | `34bbff9dbf36421a45c5c283e6c1336dcafc997a` |
| zheng | `0cc40aa10b89ff06e3082a70e340e36bcb5e4ce9` |
| crystal | `a23781cce1847e44cf4f17abf50dc240399b3da5` |
| cybics | `e39086850f513541625744dcdffb8c2eef2e868b` |
| trisha | `e97b5439bf45bde1e4804c6c2cb6b217000495bb` |
| joy | `d065814258f4d2fd86c3132c0435b5a2c99f9727` |

Mudra had pre-existing modifications to Cargo.toml, Cargo.lock, claim.rs, domain.rs and vectors.rs. Neuron/Soft3 identity work and Zheng/Trisha/Joy private execution work are concurrent and extensively dirty/untracked. Working-tree line references above describe the inspected snapshot, not a claim that those lines exist at the listed HEADs. None of that work was edited, staged, reverted or committed by this research.

## Priority implications for the parent decision

1. Start from the recovered native target; do not accidentally promote today's conventional-signature comparison into a replacement architecture without an explicit decision.
2. Separate the product question "which cheap transport/operation signature do we need now?" from the native proof-based ownership target. The project has already used that separation for browser secp256k1 and legacy migration.
3. Before claiming native proof authentication ready, define the exact versioned identity/action profile and qualify the real private proving/verifying route. Legacy folded Zheng verification and the public witness-disclosing certificate are unsuitable substitutes.
4. Preserve existing H(pubkey) identities. A future H(secret), lock-program commitment or PQ-key identity needs an explicit profile and authorized migration/recovery policy, not reinterpretation of existing 32-byte values.
