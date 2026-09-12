---
tags: mudra, crypto, privacy, recovery
crystal-type: spec
crystal-domain: crypto
status: contract
---
# private recovery

private recovery is the composition contract for discovering received notes,
verifying wallet state and retaining the ability to spend after device loss.
it uses Mudra's primitives through Cybergraph/Inf, BBG and Zheng. it introduces
no eighth primitive and selects no OMR, PIR or mailbox implementation by default.

## protected information and adversary

the private profile protects note values, recipient/owner bindings and the
client's selected records against a service observing the entire public ledger
and its own requests/responses. a delegated prover is part of that service.
sender/server manipulation, cross-request correlation and declared server
collusion are included in the profile's threat model.

the service receives neither plaintext view keys nor spend keys. a secret in
a prover witness is known to that prover, even if its proof is zero knowledge.
an explicitly trusted viewing service is a different disclosure profile.

every profile defines public leakage: canonical roots/ranges, traffic schedule,
public capacity classes and response framing. selection-dependent counts,
errors, pagination and retries must be hidden or bounded by that contract.
network anonymity/cover traffic is specified by the transport profile. a
cryptographic private query alone cannot conceal IP addresses and timing.

speed cannot be obtained by silently publishing amounts, recipient tags,
nullifier query lists or reusable view material. exact graph-aggregate releases
must undergo the enclosing protocol's disclosure analysis, including isolated
updates and an adversary that knows other contributions.

## keys and accepted delivery

the receiver has independently domain-derived authority, discovery, payload
and recovery material under versioned profiles and epochs. disclosure of a
viewing/delivery secret must not confer exclusive spending authority.

each accepted private payment has durably available recovery data sufficient
to reconstruct its note and required openings. sender-generated randomness,
nonce, payload and inclusion references cannot be regenerated from a recipient
seed alone. off-chain delivery supplies an authenticated durable archive before
the payment is considered recoverable under this profile.

the selected delivery construction binds:

- the intended recipient's authenticated discovery/encryption key record;
- the encrypted note and its commitment, network, profile and key epoch;
- any clue, tag or index update used to discover that same note;
- the canonical admitted transaction and durable data association.

profile-defined admission checks establish this relation without exposing the
recipient. malformed or inconsistent delivery data cannot bypass the advertised
recovery guarantees and error bounds for accepted private payments.
an adversarial sender cannot force an identity-revealing recovery fallback.

private directory lookup is part of delivery when a compact address resolves
to a larger recipient key record. the record's authenticity and the lookup's
recipient-selection privacy are separate requirements.

## authenticated scope and computation

the client pins the network, accepted checkpoint, state/board root, complete
ordered range and cardinality, program/profile, request keys/query and response.
the board includes all admitted records in that domain and binds clues to
payloads. a derived index proves its complete construction from that board.

for an OMR/PIR profile, proofs cover both encrypted detection/compaction and
the private payload response against the same board. commitments authenticate
data; homomorphic computation, MPC or another qualified construction supplies
the private operations. an opening proof alone does not compute over hidden
data or establish completeness of a selection.

the query relation is defined by [Inf's coverage contract](../../inf/specs/proof.md#complete-input-coverage)
and [Cybergraph's private read path](../../cybergraph/docs/private-retrieval.md).
partitioned or recursive proofs bind contiguous coverage, one canonical history
and exact combination of results. authenticated empty results cover the same
declared domain as nonempty ones.

recursive state proofs include a verified initial state and its complete
starting scope. a valid transition from an unconstrained initial ciphertext
does not establish a correct balance. the client authenticates the accepted
root through the selected consensus/light-client protocol and freshness policy;
a recovery proof alone cannot establish that a server's root is the latest one.

## correctness, capacity and state

complete execution preserves the retrieval algorithm's own errors. a profile
specifies whole-recovery bounds for cryptographic failure, missed detections,
false-positive overflow, decoding and proof failure over its maximum lifetime
scope. per-message error estimates must be composed across that scope.

response capacity is a public profile parameter independent of the actual
match count. padding, private overflow handling and continuation preserve
selection privacy. truncation cannot advance a complete-recovery cursor.
unavailable capacity or data produces an explicit incomplete status.

finding incoming notes is separate from computing spendable state. the latter
also verifies current canonical inclusion, policy and cross-epoch spent status.
archived roots and recursive validity proofs do not replace retained ciphertexts,
openings or data availability. epoch reset cannot forget an older note's later
spend when answering a current balance query.

## recovery modes and durable progress

| mode | required result |
|---|---|
| incremental update | verified changes since a durable canonical cursor |
| prepared seed restore | private discovery of a precomputed encrypted state/checkpoint and its authenticated tail |
| unprepared seed restore | complete discovery over the declared retained history/key epochs |
| balance summary | verified totals at a pinned state root, with private asset/count framing |
| spend preparation | privately selected note data/openings and current spendability |
| full export | all requested notes/history, explicitly proportional to returned data |

the seed and public protocol constants must locate every advertised key epoch,
directory entry and durable recovery object. backup-dependent or previously
registered fast paths state that prerequisite explicitly. a secret unbacked
cursor, server-issued identifier or interactive acknowledgement cannot be the
sole means of finding paid funds after device loss.

only verified complete progress advances the corresponding cursor. results,
root, policy/key epoch and cursor are committed atomically. retries are
idempotent; reading a notification does not destroy its only recoverable copy.
reorganizations roll back dependent state and replay against canonical roots.

## resource contract

reports distinguish global history N, own historical notes K, own live notes U,
queried epochs E and public padded capacity B. client download/CPU/peak memory,
verification, sender overhead, server per-user work, reusable preprocessing,
key storage, archive size and time until a spend is possible are separate costs.

the client-light target makes prepared recovery depend on selected wallet data
and succinct proofs rather than enumerating global history. full export costs
at least the returned data volume. an unprepared recovery reports its server
discovery delay separately; an existing checkpoint cannot be assumed into
existence for a wallet the service never processed.

workers may precompute state while clients are offline. their encrypted state
and proofs must be transferable between providers under the same verification
rules. replication, retention and refusal recovery are availability obligations.
no performance claim follows merely from moving work to a worker or adding
a succinct proof.

## acceptance

qualify each profile against an independent complete scan and adversarial
delivery/omission cases. verify cold, warm, prepared, unprepared, restart,
reorg, key rotation, overflow and provider-loss behavior. compare transcripts
for different recipients/match counts under the declared leakage model.

freeze the security/error parameters before comparing performance. report
end-to-end recovery and spend-ready latency with hardware, network conditions,
server work/storage and proof construction included.

candidate client-light architectures are tracked in
[the recovery-index proposal](props/private-recovery-index.md).
