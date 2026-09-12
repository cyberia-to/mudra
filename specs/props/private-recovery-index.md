---
tags: mudra, privacy, recovery, proposal
crystal-type: spec
crystal-domain: crypto
status: draft
date: 2026-09-12
---
# private recovery index

## objective

make opening or restoring a private wallet depend on the wallet result requested,
rather than downloading global chain history. retain private values, recipient
selection and independent spending authority under the
[private-recovery contract](../private-recovery.md).

the proposed direction is **compact private discovery plus continuously
maintained encrypted wallet state, with proofs of its complete construction**.
this is a composition proposal. its implementation, full security argument and
end-to-end performance remain to be established.

## separate the four user results

let N be global notifications, K own lifetime receipts, U own live notes,
A own assets and B a public padded capacity independent of the true count.

| requested result | proposed client work | work elsewhere |
|---|---|---|
| open a prepared wallet | padded balance/policy summary + verification/private lookup | continuously maintained complete state |
| prepare one spend | bounded private coin selection + selected notes/openings + verification | private query over authenticated live state |
| export all live notes | at least U note records, paged/padded | complete live-note index |
| export all receipt history | at least K records, paged/padded | durable archive |

these are target cost shapes, including private-query construction, not claims
of O(1) whole-wallet recovery. hiding activity volume requires fixed quotas or
an explicit permitted capacity class. zero-match wallets use the same observable
schedule and response shape as others in that class.

seed restore has two different latency paths. with a prepared state, retrieve
and verify it plus a bounded tail. without preparation, a service first builds
the state from retained history or a qualified pre-existing private index.
that server delay is part of the unprepared restore benchmark.

## proposed pipeline

### 1. accepted payments carry recoverable delivery data

keep CSIDH for recipient agreement/payload access. add a separately qualified
encrypted discovery clue where required by the retrieval profile. the sender
binds clue, ciphertext, output commitment and recipient key record in admission.

this lets the service evaluate an FHE-friendly discovery predicate. the wallet
performs CSIDH only on recovered candidates, rather than every global output.
it does not require evaluating CSIDH itself homomorphically.

discovery adds bytes and keys. a compact address may reference an authenticated
key record containing the CSIDH key and clue public key; private directory
resolution then becomes part of the sender's privacy/cost contract. neither
the key record nor the clue bandwidth is free.

### 2. compact matches at the service

use a bounded epoch and obtain encrypted relevance bits for its notifications.
homomorphically compress matching indices or payloads to a padded capacity B.
the client decrypts/decodes this compact result. exact encoding and failure
bounds belong to the candidate protocol.

**first candidate: SophOMR's published compact extraction.** it establishes
that client decoding can be independent of global N at fixed parameters and
capacity, at a substantial server cost. compare its payload compression with
indices-only extraction followed by private payload retrieval. an encrypted
IBLT/sketch is an alternative once relevance bits already exist.

UnifOMR's dense partial decryptions cannot simply be dropped into a sparse
sketch. moving the relevance test from client to server costs homomorphic
range tests/bootstraps or requires a different clue construction. benchmark
that complete step before claiming a faster hybrid.

### 3. maintain private state while the wallet is offline

a worker applies complete epoch results and current spent-state transitions to
an encrypted wallet index. it retains encrypted live-note references, opening
material, asset summaries, key epochs, covered history and policy state.
spent-state handling keeps this wallet's note/nullifier associations private.
plain per-wallet nullifier lists are not an acceptable shortcut.

the schematic proof relation is:

```text
state_e = ApplyPrivate(
    state_(e-1),
    complete_delivery_board(epoch_e),
    canonical_spent_changes(epoch_e),
    encrypted_discovery_material,
    selected_profile
)
```

Zheng proves the actual ciphertext program, with Inf's complete-query semantics
and BBG-authenticated input coverage. each step binds its predecessor, canonical
root/range, program/profile and encrypted state commitment. recursive composition
can reduce client history verification; it does not reduce the worker's required
discovery or private state-update computation by itself.

this layer needs a concrete encrypted update/compaction algorithm, including
how nullifiers are derived or privately checked and how openings are retained.
FHE primitives and sparse discovery alone do not implement that state machine.

CSIDH-encrypted payloads alone also do not let the worker compute balances.
one candidate is an additional FHE-encrypted recovery record containing the
needed note fields, with admission proving consistency with the payload,
recipient and output commitment. private spent checks need a separately
specified, restricted nullifier capability that grants no spending authority.
record/key size, cross-scheme consistency proofs and safe ciphertext refresh
are explicit design costs. homomorphic CSIDH decryption is not assumed away.

the state proof starts from verified initialization with complete history
coverage. a checkpoint without that base case cannot certify a balance.

### 4. retrieve a summary first, spend data on demand

derive recovery lookup material from the seed/profile, retrieve the current
encrypted checkpoint privately and validate its canonical freshness/coverage.
show a verified balance summary before downloading the full historical wallet.
privately request the bounded note set needed for a spend, with a proof that
the returned notes meet the requested policy/value condition at that root.

summary delivery, private coin selection and proof verification are all included
in latency. the provider must retain and replicate the actual spend data.
an encrypted root without available leaves cannot support a payment.

## candidate comparison

| candidate | gain | price / missing piece | decision |
|---|---|---|---|
| compact OMR + proved private state | client avoids global notification stream | per-recipient FHE/proving work, key storage, state-update design | primary experiment |
| streaming OMR / encrypted inbox | precompute while client sleeps | per-recipient update work; some published inboxes evict old messages | compare after durable baseline |
| oblivious append index + private reads | move discovery to payment admission | private atomic append/counters, unknown senders, write abuse, access-pattern privacy | longer-term alternative |
| private balance summary + lazy spend data | avoid downloading all U notes just to open | complete encrypted state and private coin-selection service | compose with primary experiment |
| direct sender-to-recipient channel | cheap delivery for known peers | offline first contact, availability and seed recovery still need an archive | optional accelerator |

TEE-only, plaintext-view-key and noncolluding-server constructions have different
trust models. they are useful research comparisons, not silent replacements
for the selected untrusted-service privacy contract. multiple workers under one
operator do not establish noncollusion.

## what the literature already demonstrates

SophOMR's final-paper example has N=524,288, 50 matches and 612-byte payloads:
**263 KB digest, 12 ms client decode, 1,098 s server, 114 MB detection key**.
at N=65,536 the digest remains 263 KB and decode is 14 ms. the latter variation
is measurement noise, not a claim that more history makes decoding faster.
these are author measurements before Cyber proofs or state recovery, with the
paper's own security/error parameters: one thread on Google n2-standard-8,
Intel Ice Lake, five-run averages. its roughly 2^-30 per-message miss
target cannot be relabeled a whole-wallet negligible-loss guarantee.
the published privacy model is passive-adversary. proving execution adds
integrity; active-service transcript privacy requires a separate composition
argument under the recovery contract.
[Final paper](https://www.usenix.org/system/files/usenixsecurity26-lee-keewoo.pdf).

naive deployment repeats work for every recipient. a linear extrapolation of
that baseline at 1,000 global notes/s requires about 2.1 core-equivalents of
that hardware per recipient,
before Cyber proofs or stricter parameters. this makes private routing/index
maintenance a necessary scaling investigation alongside the client-light
prototype. [Cost arithmetic and limitations](../../audit/private-recovery/README.md).

Tachyon provides a related proof-carrying wallet-state direction. its initial
outline uses out-of-band secret delivery and exposes nullifiers to the syncing
service; recovery and correlation need further protocol work. borrow the
separation of state maintenance from client wake-up, with Cyber's own privacy
and post-quantum requirements.
[Primary proposal](https://seanbowe.com/blog/tachyon-scaling-zcash-oblivious-synchronization/).

full candidate evidence and caveats are in the
[recovery research](../../audit/private-recovery/README.md).

## experiment and acceptance sequence

1. define a canonical admitted-note fixture and independent complete scan.
   test adversarial clue/payload mismatch, false detections and capacity limits.
2. reproduce compact extraction; separately measure sender cost, detection-key
   upload, server per-recipient CPU/RAM and client query/decode work.
3. implement an encrypted, nonconsuming recovery-state machine with proved
   updates and private lookup. exercise concurrent senders, lost responses,
   provider replacement, stale snapshots, omitted epochs and reorgs.
4. compare prepared summary restore, live-note export, on-demand spend preparation
   and unprepared historical restore. include all proof construction and
   verification; do not report decode time as spend-ready latency.
5. optimize the measured bottleneck: epoch size, batching, shared public
   preprocessing, payload-versus-index compression and backend acceleration.

for fixed wallet output/capacity and security parameters, increasing global
history must not introduce a client enumeration step. prepared recovery is a
candidate for the scorecard's strongest client-work grade only when its full
lookup/decode/verification/spend path demonstrates that property. report server
work per recipient, retained bytes and queueing under load beside that grade.
