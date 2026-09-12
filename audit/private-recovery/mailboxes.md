# Private wallet recovery through sender-assisted state

Research snapshot: 2026-09-12. Primary papers inspected; published measurements below were **not rerun**. No workspace changes. `N` = retained signals/notes, `M` = recipients, `k` = this wallet's matching notes, `L` = padded response capacity. Payload encryption, amount hiding and spending proofs are separate from mailbox privacy.

**Decision:** shifting discovery work from recovery to a durable, recipient-oblivious index is a credible architecture. The literature contains both concrete TEE prototypes and cryptographic existence results with small client work. None of the inspected systems is a ready-made combination of permissionless unknown senders, malicious-server completeness, post-quantum security, indefinite retention and seed-only restart. Do not equate a compact digest or PIR response with sublinear total client work.

## Candidates and actual costs

| Construction | What it contributes | Why it is not the complete wallet design |
|---|---|---|
| Scalable Private Signaling | Private linked-list append and private head lookup using TEE + ORAM; client receives only a padded page | Trusted enclave; consuming reads and local replay counter; correctness assumes semi-honest server |
| Multi-client ORAM, ASIACRYPT 2020 | Cryptographic private reads/writes with polylog client work, including mutually untrusted clients | Linear server work; access-grant setup; arbitrary writes are not authenticated, permissionless append |
| Express | Efficient private writes; malformed-write protection | Read mailbox is visible; additive cells can collide; reads clear storage |
| Talek | Hidden write/read patterns for shared, single-writer logs | Linear PIR requests/notification vector; retained rolling window; shared handles |
| Two-Face | Active-secure compact signal discovery without a persistent mailbox service | Full board scan at servers; match-count leakage; DDH, not PQ; client permutation cost needs qualification |
| Sabot / Alpenhorn | Private first-contact bootstrap | Interactive contact establishment, not archival note recovery |

### 1. Scalable Private Signaling: direct stateful design reference

[Jakkamsetti–Liu–Madathil, ePrint 2023/572, revision 2024-06-06](https://eprint.iacr.org/2023/572.pdf), §§2,5–7, Figs3–4. Recipient registration publishes an identifier; an unknown sender encrypts `(recipient, ledger location)` to the enclave. Encrypted linked lists, heads and verification keys live in ORAM. Abstract costs: send `O(log N + log M)`, retrieve `O(L log N + log M)`; the actual recursive PathORAM prototype uses `O(log²)` accesses. Client work is independent of board scanning, with `O(L)` returned indices.

Published benchmark: **under 70 ms send; under 6 s retrieval of 100 signals**, at 10M signals/1M recipients, Azure DC2s v2,2 vCPUs. These are enclave processing figures, not end-to-end seed recovery. Prototype uses RSA-OAEP, ECIES and ECDSA: **not PQ**. Its ideal model assumes anonymous transport; §5 assumes semi-honest servers for correctness despite integrity-protected ORAM.

Crucial recovery gap: Fig3 checks an exact client replay counter, advances that counter and consumes the inbox head before returning. Lost response or lost client state has no specified recovery API. Already-read history cannot be rediscovered through that head. A wallet needs a nonconsuming historical head, recoverable counters, idempotent replies and rollback protection.

### 2. Multi-client ORAM: cryptographic possibility, substantial server cost

[Chow–Fech–Lai–Malavolta, ASIACRYPT2020](https://eprint.iacr.org/2020/1551), §§1,5 and Table1. Unlike ordinary ORAM, it handles mutually untrusted clients with distinct read/write permissions and hides accesses from other clients as well as servers. The FHE construction gives `O(log N)` client computation/communication, ignoring security-parameter factors, but **O(N) server computation per access**. Multi-server DPF constructions also achieve polylog client costs under their stated homomorphic-secret-sharing/PRG assumptions. No runtime benchmark is reported in the inspected paper.

This supplies an existence result for moving client work to servers, not a deployed mailbox. The owner must grant access; publishing a write capability does not automatically enforce append-only behavior or a private atomic counter. §5.2 integrity is defined against malicious clients with honest servers; malicious-server privacy must not be relabeled complete authenticated recovery. A wallet adaptation needs an authenticated append operation and canonical index completeness.

The older [Blass–Mayberry–Noubir multi-client ORAM](https://eprint.iacr.org/2015/121.pdf), §2, instead assumes mutually trusting clients sharing a key. Its rewind analysis shows why remote state freshness matters; it is unsuitable as-is for arbitrary senders. Sharing one ORAM master secret with every payer would violate the wallet model.

### 3. Express: useful oblivious-write primitive

[Eskandarian et al., USENIX Security2021](https://www.usenix.org/system/files/sec21-eskandarian.pdf), §§1–4,7. Two noncolluding servers tolerate one malicious server and malicious clients; DPF writes and blind audits hide and constrain the selected mailbox. Published client overhead is about20 ms/5 KB per message. Server work remains linear in registered mailboxes.

The paper expressly **does not hide which client reads which mailbox**. Mailbox addresses are capabilities distributed to senders. Writes add rather than append; overlapping messages can clobber each other, and reads clear the cell. The suggested per-sender mailbox setup avoids ordinary collisions but cannot itself give an unknown sender a seed-recoverable historical inbox. Publicizing the write capability also removes its protection against targeted destructive writes. Private reads, retained pages, append allocation and archive authentication are additional protocols, not a matter of merely replacing `GET` with PIR.

### 4. Talek: good known-peer warm channel, not sublinear cold restore

[Cheng et al., ACSAC2020, ePrint revision2020-12-16](https://eprint.iacr.org/2020/066.pdf), §§4–9, Fig6. Anytrust servers protect access patterns; a secret shared log handle derives pseudorandom write locations, while IT-PIR hides reads. Logs have one writer. Exposing a handle reveals its writer's access pattern. Unknown sender bootstrap and recovering others' handles are not solved by deriving the wallet's own seed.

Servers retain a rolling window and discard old messages; long-offline clients may ask writers for sequence numbers and retransmissions. The implementation's PIR query generation and query size are **linear in retained database size**, as is its global interest vector. At 1M 1 KB messages: client PIR query 6888 µs, request 93.72 KB, response 4.16 KB, updates 14 KB. Published cluster result: 9433 messages/s for 32k active users,1.7 s latency. Those results do not establish an archival recovery bound.

### 5. Two-Face: small malicious-secure digest with explicit leakage

[Chu–Wang–Jia, ePrint2025/1056](https://eprint.iacr.org/2025/1056.pdf), Fig1,§§3–6,Table2,AppendixA. Two noncolluding servers, one actively malicious, return pertinent signal indices consistently with an assumed public ledger. Unknown senders need only the recipient public key. At`N=2^19`,50 matches: **33.57 KB digest,0.33 s client reconstruction,120.29 s server runtime with 16 threads on LAN**; server-to-server traffic 229.45 MB. Computation scans the board.

The compact functionality explicitly reveals **match count**. The volume-hiding variant returns `O(N)` data—48 MB at that board size. Network-level unlinkability is assumed separately. It uses ElGamal/secp256r1/DDH, not PQ. Client reconstruction reverses seeded permutations; the inspected paper does not establish a polylog-cost random-access permutation implementation, so **do not infer `O(k+polylog N)` total client work from digest size alone**. Returned indices still require private payload fetching. Ledger-to-note binding and durable nullifier/spent-state verification remain wallet responsibilities.

### 6. Bootstrap does not eliminate the recovery index

[Sabot, CCS2025/ePrint2025/971, revision2025-08-08](https://eprint.iacr.org/2025/971.pdf), §§4–8,Table2: clients privately discover prospective senders using a secret-shared relationship matrix and authenticated PIR. At`M=2^14`, malicious-setting variant reports 7.04 KiB per established channel **when amortized over 10 retrievals**; one retrieval costs 25.76 KiB. Client notification cost remains `O(M)`, global matrix work `O(M²)`. Its privacy analysis against active adversaries is informal; authenticated contact records do not prevent a malicious server suppressing notifications. The [evaluated prototype](https://github.com/laurahetz/sabot) explicitly remains experimental. It handles first-contact privacy, not permanent money history.

[Alpenhorn, OSDI2016](https://www.usenix.org/conference/osdi16/technical-sessions/presentation/lazar) bootstraps from a username using IBE, anytrust servers and evolving pairwise keywheels. Published scale: 10M users,150 s average dialing,3.7 KB/s client bandwidth. Its evolving address-book secrets and metadata forward secrecy are different goals from reconstructing all historic receiving state from a permanent seed. Backing that state up can change those compromise guarantees; it is not free seed recovery.

## Required Mudra contract — design inference, not a new proved construction

1. **Accepted notes create the index.** Couple each accepted note with a hidden, well-formed routing entry under the same finalized commitment. A sender may know the recipient public address but must not need its read key, prior senders or current counter. Admission must enforce one valid append, not permit arbitrary mailbox replacement. Encrypting an amount and recipient in the note does not prove routing correctness.
2. **Private append and recoverable head.** Store an authenticated permanent head/count plus encrypted pages or links. Resolve concurrent unknown senders atomically without disclosing the address/count. A random slot without a private directory merely recreates a scan. An ordinary public `H(address)` bucket makes recipient-selection privacy vulnerable to an address dictionary.
3. **Cold recovery is read-only and repeatable.** Derive an independent view/recovery key and stable lookup material from seed. Obtain head, pages and spent-state evidence privately at one canonical root. A consumed head or unrecoverable client-only counter is forbidden. Receipts/cursors must survive lost replies; wallet acceptance advances only after verification. Normal reads must not erase historical recoverability.
4. **Authentication includes completeness and freshness.** A Merkle proof authenticates returned entries; it does not prove that all accepted notes were routed or that an unknown tail was not omitted. Need a committed index-transition proof or equivalent trusted correctness mechanism, root-linked counters, finality/reorg rules and retained available data. A server-provided self-consistent old root is insufficient for cold recovery.
5. **State the leakage budget.** With padded page size`L`, returning`ceil(k/L)` observable linked pages leaks a size bucket. Strict volume hiding needs fixed quotas/cover or an explicit anonymous-unlinkable-query assumption. Recipient-selection and amount privacy can remain intact under some volume leakage, but this must be a deliberate statement, not an implicit weakening. Noncollusion requires distinct trust domains; several workers under one operator are not that assumption.
6. **Bound abuse without spend-key exposure.** Public receiving addresses imply anyone can generate traffic. Bind index growth to accepted economically bounded notes; validate encrypted writes before mutation. Keep view/recovery authorization separate from spending authorization and do not equate delegating a view key with private outsourcing. Recovery still needs note decryption and spent-status checks; index size should count historical notes, not only today's unspent balance.

A worthwhile bounded prototype is therefore **an authenticated, nonconsuming private inbox state machine first**, exercised with concurrent senders, server omission/replay, lost responses and seed-only reset. Benchmark write amplification, retained bytes per note, initial private-head access, padded-page recovery and all client preprocessing. Choose a specific proven retrieval/write primitive afterward; TEE and noncolluding-server implementations have materially different trust contracts. The reviewed evidence supports pursuing this architecture, not claiming a finished trustless PQ construction or inevitable impossibility.

Source file hashes: [mailboxes-sources-sha256.json](mailboxes-sources-sha256.json).
