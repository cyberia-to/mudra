# Private recovery: design findings

2026-09-12. Research and design revision; published measurements were not
reproduced locally. Canonical requirements are in
[private recovery](../../specs/private-recovery.md); the candidate architecture
is [private recovery index](../../specs/props/private-recovery-index.md).

## The useful change

UnifOMR is one point in the client/server tradeoff. Compact OMR constructions
already remove its global-length client detection stream. The promising Cyber
composition adds a continuously maintained, proved encrypted wallet state:
summary first, privately selected spend data on demand, full history separately.

| route | client advantage | cost or prerequisite |
|---|---|---|
| SophOMR compact discovery | result depends on configured match capacity, not global N | expensive per-recipient FHE scan; large detection keys |
| prepared encrypted state | skip lifetime replay when opening a wallet | someone maintained complete state; private lookup, freshness and availability |
| private balance + lazy spend data | avoid exporting all U notes to display a balance | complete summary/selection proofs and accessible note data |
| sender-assisted private index | potentially avoid scanning for each recipient | private concurrent append, routing and authenticated recovery remain a concrete construction problem |

SophOMR's author measurements at N=524,288, k=50, P=612 are **263 KB digest,
12 ms client decode, 1,098 s server and 114 MB detection key**. This is evidence
of compact client work, not an end-to-end Cyber recovery benchmark. Its passive
threat model and roughly 2^-30 per-message miss target require further work
for the selected malicious-service/lifetime-error contract.
[Final paper](https://www.usenix.org/system/files/usenixsecurity26-lee-keewoo.pdf).

The infrastructure cost matters. A deliberately simple linear extrapolation
of that single-core server result is 1,098 / 524,288 ≈ 0.002094 CPU seconds per
global message per recipient. At 1,000 global messages/s it implies about
2.1 core-equivalents per recipient, or 20,943 for 10,000 recipients, before
Cyber proving and stricter parameters. This is planning arithmetic, not a
throughput measurement. Shared preprocessing does not eliminate every private
predicate. Private routing/index maintenance is the longer-term scaling track.

## Evidence

- [Retrieval constructions and exact published costs](retrieval.md): SophOMR,
  InstantOMR, OSig, IBLT, PIR and proof-carrying state; pinned code and PDFs.
- [Sender-assisted inbox/index alternatives](mailboxes.md): ORAM, Express,
  Talek, signaling and the missing permissionless/seed-recovery pieces.
- [Contract review before this revision](contracts-before-revision.md):
  action binding, NIKE versus spending, retained note data and BBG conflicts.

The pre-revision Mudra findings are addressed in identity, seal, stealth,
veil and the new recovery contract. The following BBG issues remain explicit
integration work and are not used as foundations of the proposal:

- `specs/neuron-state.md` delegates a plaintext viewing-key witness to a
  prover; this discloses viewing ability to that prover.
- its common-divisor check permits G=1, so it does not prove the complete
  intersection; a polynomial commitment alone cannot compute hidden GCDs.
- summing a general polynomial evaluated at additive query shares does not
  produce its evaluation at the secret query point.
- epoch roots do not retain missing ciphertexts/openings or automatically
  prove current spent status for old notes.
- exact public aggregate changes can reveal an isolated private contribution.

[Exact references and scope](contracts-before-revision.md#fundamental-boundaries-and-conflicting-bbg-sketch).
These are design limitations, independent of transient implementation bugs.
