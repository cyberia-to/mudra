# Mudra identity readiness and Quantus reuse assessment

Date: 2026-09-12. Status: research and executable interoperability evidence.
This report does not accept a new identity profile or change runtime behavior.
The user's neuron harness and Mudra changes in another thread were preserved.

This is the earlier implementation snapshot. The revised
[strategic comparison](signature-optimality.md) governs the design recommendation;
temporary implementation failures below are not algorithm-selection criteria.

## Decision

Quantus has reusable implementation work: its ML-DSA library is a credible
independent test backend and a candidate for a versioned production signing
adapter. Its Wormhole implementation demonstrates real hash-preimage spending,
local ZK wrapping and recursive aggregation. Its privacy/restore tradeoff is
useful to study, with narrower guarantees than a fully confidential ledger.

Mudra currently supplies classical secp256k1 derivation and ADR-036 signatures.
It needs explicit identity lifecycle and action-authentication work before
supporting recoverable identities whose IDs survive key rotation. Its proposed
proof-based identity path is substantially incomplete. Replacing its signature
primitive alone does not resolve those requirements.

The strongest concrete results from this review are:

- 54 Quantus/OpenSSL ML-DSA-65/87 interoperability cases pass, including
  deterministic byte equality and cross-backend rejection of altered data.
- Mudra's normal tests pass; its optional `prove` build fails, and direct
  arithmetic probes expose canonicality differences from the native verifier.
- The current Quantus mobile wallet sends full address sets to its indexer.
- Its incremental UTXO cursor can skip deposits if the indexer trails RPC
  by more than its 180-block overlap. This is a source-derived counterexample;
  a live wallet/indexer reproduction was not executed.

## Sources and reproducibility

| Repository / component | Reviewed revision |
|---|---|
| Mudra | `d6eab0d251d7521c65940e1b3c5aab86abcb6af2` plus observed working changes |
| Neuron | `34bbff9dbf36421a45c5c283e6c1336dcafc997a` plus ongoing harness changes |
| qp-rusty-crystals | `3b1464f751d536aba023a53df2a4f4533ab94f62`, dilithium 4.1.1 |
| Quantus chain | `662ef6d1dea8f3572776b8db9da1f57283744fd2` |
| Quantus chain release v1.0.1 | `f1176cea6a6d08ea437710dcd45cae6717b773df` |
| qp-zk-circuits main | `e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99`, 4.4.0 |
| qp-zk-circuits used by chain | `b224e6dcbb89c03f90fbd26f68674c3b557ef36f`, v4.3.0 |
| quantus-apps | `e843b06b49e4c208f7b4a8c603a91f4578780e96` |
| quantus-cli | `b98083bf29a3bee5a121affd723431d3654c3247` |
| quersi | `7d79352af27a1c91a6d4f6778a94c41657e506c9` |
| Quantus docs | `384472d79c3cae193e455f612e5cd9b2d8e98779` |

Detailed evidence: [Mudra](mudra-identity-readiness.md),
[circuits and chain](quantus-wormhole-evidence.md),
[wallet and recovery](quantus-wallet-evidence.md),
[executable signature check](quantus-mldsa-crosscheck/README.md).
Source hashes in the Mudra appendix identify mutable files more precisely than
HEAD. Temporary upstream clones were kept outside the Cyber repositories.

The [September 8 announcement](https://www.quantus.com/blog/weekly-update-09-08-2026/)
announced launch on September 9, mining as the initial way to acquire QTC,
and later liquidity. Relevant v1.0.1 release source enables Wormhole calls.
This review did not compare the live chain's runtime WASM with that release
or submit a mainnet transaction. Source readiness and live deployment
equivalence remain distinct claims.

## 1. Mudra and durable identity

Four identifiers have different lifecycle requirements:

| Identifier | Stability rule | Authentication role |
|---|---|---|
| Neuron subject ID | Same logical subject across restart/device/network bindings | Names the subject; possession of the bytes grants no authority |
| Signing key / key ID | Can change on rotation, revocation or algorithm migration | Authorizes a defined statement under a named profile |
| Foreign chain address | Follows that chain's address/derivation rules | Referenced under an explicit foreign domain |
| Request / operation ID | Same value for retries of one admitted action; fresh for a new action | Enables durable deduplication; authorization is separately verified |

The accepted current neuron profile is `cyber-secp256k1-hemera-v1`:
`NeuronId = Hemera(valid compressed SEC1 public key)`. Same seed/path/profile
reconstructs the same ID. Moving a worker/device preserves the subject.
Changing its public key changes this ID; retaining the old subject through a
new key requires a separately specified recovery/rotation profile.
[Current identity contract](../../neuron/specs/identity.md).

Mudra's current tested surface includes BIP-39/BIP-32 legacy recovery,
domain-derived secp256k1 keys, Cosmos addresses and ADR-036 verification.
Default tests passed **18 library + 2 vector tests**; no-default-feature
library tests passed **15**. Neuron's structural identity tests passed **3**.
These checks do not establish a complete host authorization integration.

Observed blockers and boundaries:

1. `--features prove` fails E0063: the demo initializes `zheng::Statement`
   without its current `bbg_root`. Its implemented circuit is `a*b+c`; full
   claim computation and action binding are explicitly unfinished.
2. The emulated ECDSA reference accepts `s=n+1`, high-S signatures and a
   noncanonical SEC1 coordinate `x=p+1` in cases where the native k256 path
   rejects them. These are reproduced parser/arithmetic equivalence gaps,
   not a demonstrated forgery of a completed ZK claim system.
3. `Claim::decode("address 😀 00 00")` panics on UTF-8 byte slicing.
4. Existing native/domain-ID tests do not freeze literal derived ID bytes.
   Domain normalization is a caller responsibility. The derivation calls the
   currently experimental Hemera inverse-16 profile directly.
5. Mudra's identity proposal describes `H(secret)` and a 64-byte address;
   accepted neuron implementation uses `H(pubkey)` and 32 bytes. The proposed
   preimage statement must bind the authorized message and replay domain.
   Hash preimage resistance, proof soundness/knowledge properties, zero
   knowledge and the concrete transcript/hash configuration all remain
   security assumptions.

The ongoing neuron harness already introduces a default-deny Authority port
and versioned action/request records. Complete its Mudra-backed adapter in
that thread. This review does not replace or implement that concurrent work.

**Recommended lifecycle direction, proposed rather than accepted:** give
a persistent subject an immutable inception/root commitment, and maintain
versioned authorized key/policy bindings separately. An authorized transition
binds the subject, old revision, new keys/profile, recovery policy and relevant
scope. Revisions/revocations and accepted transitions are durable state.
Recovery requires a declared trust mechanism and recoverable state/history;
having a stable 32-byte label alone is insufficient. Optional domain-scoped
aliases can reduce cross-service correlation of a globally stable identity.

For requests, persist the canonical action and its nonce/ID before dispatch,
then persist its original receipt atomically. Do not derive the retry ID from
signature bytes: randomized signatures and key rotation can change those bytes
without creating a new business action. Authentication must bind the same
canonical subject/network/action/revision/nonce that deduplication protects.

## 2. Which Quantus signature components to reuse

The base library implements **pure ML-DSA**, standardized by
[FIPS 204](https://csrc.nist.gov/pubs/fips/204/final). It has additive 44/65/87
features, a no-std core, context strings up to 255 bytes, deterministic and
hedged signing, key-import checks, zeroization and test vectors.
This is an implementation of a standard primitive, independent of adopting
Quantus consensus, addresses, wallet or Wormhole circuits.

| Parameter set | NIST category | Public key | Signature |
|---|---:|---:|---:|
| ML-DSA-65 | 3 | 1,952 bytes | 3,309 bytes |
| ML-DSA-87 | 5 | 2,592 bytes | 4,627 bytes |

The chain's `SignatureWithPublic` carries signature **and** public key; its
enclosing SCALE enum adds one byte. Thus the signature field is **5,262 B for
65 / 7,220 B for 87**, before other extrinsic fields. Wormhole uses a separate
proof-authorized spending path. [Wire variants](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/primitives/dilithium-crypto/src/types.rs#L51-L74),
[combined encoding](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/primitives/dilithium-crypto/src/scheme_macro.rs#L64-L90).

Sizes were checked in the executable against the actual APIs. The chain
supports both profiles and uses `QUANTUS_EXTRINSIC` as its FIPS signing context.
Cyber must define its own context and action envelope; raw ML-DSA compatibility
does not make a Substrate extrinsic a Cyber authorization record.
[Chain context](https://github.com/Quantus-Network/chain/blob/662ef6d1dea8f3572776b8db9da1f57283744fd2/primitives/dilithium-crypto/src/signing_context.rs).

Executed evidence on macOS arm64:

- Quantus debug all-features suite: **164 unit, 1 integration, 11 doc tests**.
- Quantus release all-features suite: **161 unit, 8 integration, 11 doc tests**;
  release mode additionally enables stack-zeroization probes.
- These include vendored NIST ACVP keygen/sign/verify suites at all three levels.
  Their ACVP harness covers the internal interface, with 25/30/15 vectors per
  parameter set. This is bounded conformance testing, not NIST module validation.
- Our independent OpenSSL 3.6.2 comparison passed **54 external-interface
  cases** at 65/87, with matching keys and deterministic signatures, both
  verification directions, hedged signing and negative mutations. Empty-file
  messages were excluded from cross-backend execution because of an OpenSSL
  CLI limitation; local empty-message round trips passed.

The [Neodyme audit](https://neodyme.io/reports/Quantus_Dilithium_HDWallet.pdf)
dated December 15, 2025 reviewed Dilithium and HDWallet through commit
`8bbe920dc5bfe88fa40028eb1ea9bae4e39a600c`. Its findings table records one
critical domain-separation defect, two high and other issues as resolved;
two low constant-time findings were acknowledged. Hardware attacks and
third-party dependencies were outside scope. This is useful prior review,
with an explicit gap to the September 4.1.1 implementation and new variants.

There is a concrete licensing discrepancy: repository-root LICENSE contains
Apache-2.0, while the actual dilithium crate manifest and its own LICENSE say
GPL-3.0. Inspect those files before embedding it in a distributed binary.
[Crate manifest](https://github.com/Quantus-Network/qp-rusty-crystals/blob/3b1464f751d536aba023a53df2a4f4533ab94f62/dilithium/Cargo.toml).

**Adoption recommendation:** use the isolated Quantus/OpenSSL harness now as
an interoperability baseline. Add a versioned Mudra ML-DSA adapter only after
the envelope/key-lifecycle profile is fixed. Keep 65 and 87 explicit; category
5 is available when that is the selected target, with larger keys/signatures.
Preserve an independent implementation in CI rather than testing two wrappers
over the same signer. Review the post-audit diff, external FIPS contexts,
malformed inputs and target-specific custody/zeroization before a release.
The conventional secp256k1 bridge can retain its own separate profile.

The HD-wallet crate is useful reference material for **hardened-only**
derivation of ML-DSA keys and separate Wormhole secrets. A BIP-44-shaped path
does not imply public child derivation or complete wallet discovery. Its
network coin types and profile choices should remain Quantus-specific.
Threshold signatures and Wormhole proof-authentication require separate
reviews; passing this base ML-DSA check does not validate either system.

## 3. Quantus privacy model

Wormhole uses publicly recorded deposits into hash-derived addresses and
proves the right to create exits without revealing which deposit was spent.
The proof binds the secret, authenticated deposit/count, root/header,
nullifier and outputs/fee constraints; chain state enforces uniqueness.
Every exit can itself become a new Wormhole deposit.

| Property | Current mechanism / limitation |
|---|---|
| Deposit-to-exit linkage | Hidden cryptographically by the intended local ZK workflow, subject to proof/hash assumptions |
| Amount confidentiality | Deposit and exit amounts remain public; batching hides individual intermediate input amounts |
| Address confidentiality | Deposit and exit addresses are public; proofs omit the source address linkage |
| Remaining private balance | Requires the owner's derived nullifiers; spent source accounts retain their visible balance |
| Amount/time analysis | Can narrow or identify candidates using unusual sums, timings, address reuse and external knowledge |
| Indexer privacy | Current mobile requests expose full sets of queried addresses and exact nullifier hashes |
| Transport anonymity | Needs a separate mechanism; ZK does not conceal IP or query timing |

The architecture has a useful distinction that also applies to Cyber workers:
individual leaf proofs are **non-ZK** and sensitive. The wallet's local
**PrivateBatch** is ZK and hides their witnesses/individual amounts, pads and
shuffles slots. A remote **PublicBatch** aggregator receives already protected
private batches and preserves their public segment boundaries. Therefore
delegating PublicBatch is a different disclosure decision from delegating
leaf/PrivateBatch work.
[Configurations](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/common/src/circuit.rs#L372-L424).

Current defaults are 7 leaves per PrivateBatch and 53 private segments per
PublicBatch. The 371-transfer capacity does not imply 371 equally plausible
senders; a private batch can contain one user's deposits. Public amounts,
segment boundaries and correlations constrain practical anonymity.
The fuzzy-knapsack analogy in QIP5 supplies intuition, without a quantified
anonymity bound for actual payment distributions.

This is a valid engineering tradeoff for queryable accounts and source-link
privacy. Confidential amounts, private queries and sustained shielded state
are additional requirements. The designers explicitly discuss the tradeoff
in their [Wormhole/shielded-pool discussion](https://research.quantus.com/t/wormhole-x-shielded-pool/8).
Its comparisons of future wallet protocols are claims to examine separately;
this report does not treat them as a proof that privacy and fast recovery
cannot coexist.

The [formal package](https://github.com/Quantus-Network/qp-zk-circuits/blob/e77ba5c70da7dc7b3d3da82c84e0fb8e20b40c99/formal/WormholeSpec/Trusted.lean#L60-L92)
explicitly assumes leaf/private proof soundness. Quantitative security,
extraction and the unlinkability game remain outside the proved portion.
Its configured minimum security floor is 100 bits, so ML-DSA category 5
cannot be reused as a claim about the whole Wormhole system. The public Eiger
Wormhole report is an older draft with an unfinished remediation page; later
source includes repairs, without establishing full review of the current code.

## 4. Recovery, balance queries and updates

For ordinary accounts, the wallet can query balance/nonce by derived address.
For Wormhole funds, it must derive addresses, discover their public deposits,
derive per-deposit nullifiers, exclude spent nullifiers and sum remaining
amounts. An ordinary account balance query is insufficient: exit settlement
does not debit the original Wormhole address.

For the mobile wallet's supported deterministic account/receive/change paths,
seed-based recovery can reconstruct the secrets needed to rediscover deposits
from available chain/indexer history. It avoids downloading every global
encrypted note for trial decryption. Recovery still requires complete history,
the correct derivation/profile/passphrase and a discovery policy that reaches
the addresses used. Arbitrary externally supplied CLI secrets and paths outside
the app's discovery range do not become recoverable merely by importing the
same mnemonic.

The app currently sends full addresses in grouped GraphQL `to.id _in` queries
and exact spent-nullifier hashes. That exposes which receive/change addresses
belong to one querying client. The CLI also offers hash-prefix transfer
queries, but hashing publicly enumerable addresses does not hide them by
itself; prefix length, bucket occupancy and repeated query intersection matter.
The app's reviewed path uses exact queries.

Current CLI reward-collection callers use **8 hex characters / 32 bits** of
unsalted BLAKE3. With a public dictionary of `N` candidates, expected unrelated
matches are approximately `(N-1)/2^32` under uniform hashing. At a hypothetical
million candidates this is only 0.00023: a prefix will usually identify a
single known address. This estimates a dictionary attack, not the live
network population or an unconditional anonymity guarantee.
[Prefix units](https://github.com/Quantus-Network/quantus-cli/blob/b98083bf29a3bee5a121affd723431d3654c3247/src/subsquid/hash.rs#L5-L32),
[current caller](https://github.com/Quantus-Network/quantus-cli/blob/b98083bf29a3bee5a121affd723431d3654c3247/src/collect_rewards_lib.rs#L325-L342).

The app's gap limit is **20 consecutive unused indices**, scanning external
and change branches from zero on each reload. It downloads history and checks
spent status in pages/groups of 300. Both cold restoration and warm private
reload retain approximately `O(A + T)` local work for `A` scanned addresses
and `T` retained incoming transfers; warm network work benefits from caching.
Balance display trusts indexer completeness and spent answers. Generating a
spend adds authenticated Merkle-witness checks, a separate assurance boundary.
The app's mnemonic bridge currently supplies no optional BIP-39 passphrase,
so importing a CLI passphrase wallet is not automatically equivalent.

**Correctness counterexample, derived from source:** if RPC is at height 1000
while the indexer has only reached 700, a successful query can advance the
wallet cursor to `1000 - 180 = 820`. Later queries use `blockHeight > 820`, so
deposits that the indexer subsequently adds at 701 through 820 are omitted.
The same cursor advance can occur after an empty response. Explicit cache
clearing allows a rescan. The current flow does not establish a verified
indexer watermark or prove response completeness.

This should be corrected by advancing a durable cursor only to an indexed,
verified/common chain point, retaining overlap, detecting reorgs and testing
indexer lag, omission and restart. The app's once-per-minute balance polling
and cache rules are implementation scheduling choices, not synchronization
latency guarantees. See the [wallet evidence](quantus-wallet-evidence.md)
for discovery gaps, polling, reorg and prefix-query details.

No cold-restore benchmark, mobile proving latency, sustained mainnet throughput
or p50/p95 sync latency was measured here. The current whitepaper derives
about **430 QTPS** from a **266 KB / 371-transfer** aggregate and block-space
assumptions. Those figures measure a different resource from wallet recovery.
[Authors' capacity calculation](https://www.quantus.com/whitepaper/v0.4.1/#scalable).

## 5. Next implementation sequence

1. Finish the existing neuron authority adapter against the current canonical
   action and host/vault boundary; preserve fail-closed behavior.
2. Specify stable-subject inception, key/profile rotation, recovery, revocation
   and domain-separated aliases. Freeze byte encodings and derivation vectors.
3. Repair Mudra's bounded parsing and native/reference canonicality; make the
   proof feature compile, then replace the arithmetic demo with the actual
   action-bound computation if pursuing proof authentication.
4. Add the chosen ML-DSA profile behind a replaceable Mudra signer/verifier
   boundary, retaining the independent interoperability fixture in CI.
5. For privacy, specify what the chain, aggregator, indexer, RPC and network
   observer can each see. Mark leaf/private proving as witness-sensitive work
   in worker placement contracts; remote public aggregation can have a
   different disclosure policy.
6. Define and test seed restoration and state synchronization explicitly:
   many derived addresses, sparse indices, full history, indexer lag, reorg,
   missing results, cache loss and restart. Measure time to first displayed
   balance separately from complete verified balance and spend readiness.

The signature decision can proceed on established ML-DSA interoperability
evidence. The identity lifecycle, Hemera profile assurance and private state
recovery remain separate release obligations with concrete next tests.
