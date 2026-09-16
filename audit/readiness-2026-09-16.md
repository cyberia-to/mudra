---
tags: mudra, neuron, audit, readiness
date: 2026-09-16
---
# Neuron and Mudra: implementation readiness

**Neuron's local execution/authority contract is implementable and substantially
implemented. Mudra's existing identity adapter works; its intended private,
post-quantum system still needs concrete protocol profiles before complete
implementation.** Build defects and unfinished protocol design are separate
findings. No signature replacement or account/UTXO redesign is selected here.

This assesses the current **working trees**, including the uncommitted convergence
work: Neuron base `34bbff9dbf36421a45c5c283e6c1336dcafc997a`, Mudra base
`348c46195388ac009667144987511a0e33bdc859`. A checkout of those commits alone does
not contain the evaluated implementation. All 94 Neuron and 42 Mudra files listed
in the September 13 [source manifest](../../soft3/audit/neuron-cell/source-manifest.json)
still match its hashes. This checks recorded files, not every transitive build
input. [Fresh evidence](readiness-2026-09-16/README.md) distinguishes rerun tests
from the earlier [P13 report](../../soft3/audit/neuron-cell/release.md).

## Decision matrix

| Capability | Specification readiness | Current implementation | Next step |
|---|---|---|---|
| Neuron identity, progs, invocations, budgets, effects, migration | Concrete schemas, transitions and failure rules for the local profile | Working engine/node/CLI, shared BBG persistence, signed publication and guarded dispatch | Harden and package this profile; map current conformance evidence |
| Mudra native identity and NSIG1 | Concrete `H(compressed_pubkey)`, ADR-036 domain and exact 102-byte envelope | Implemented and used by Neuron | Fix parser/integration defects; preserve existing vectors |
| Private programmable authority | Required statement/privacy semantics defined; deployed profile not selected | Generic ZK execution exists; private authority relation and admission gate are absent | Fix relation, verifier/profile, policy state and replay transition |
| `seal` | KEM role defined; standard-vs-native profile and payload envelope require selection | No exported implementation | Pin complete KEM/KDF/AEAD/key-record profile, then implement |
| `stealth` | CSIDH family and delivery role defined; production parameters/variant/encodings unspecified | No exported implementation | Pin validated action, parameter set, key validation and KDF |
| `veil` | Candidate FHE interface; concrete parameters/cost/error budget not selected | No exported implementation | Select only when required by the chosen recovery/computation protocol |
| `quorum` | Basic Shamir sketch usable; VSS/DKG/threshold construction incomplete | No exported implementation | Define verifiable sharing and scheme-specific threshold relation |
| `delay`, `place`, `order` | Delay construction unselected; place contains unsound claims; local order contract missing | No exported Mudra implementation | Separate qualification tracks; keep off the first authority critical path |
| Private recovery and fast wallet opening | Strong requirements and acceptance cases; index proposal is still draft | No complete private delivery/index/checkpoint/spend-recovery path | Specify and prototype neuron-prepared writes plus private authenticated reads |

Sources: [Neuron runtime](../../neuron/specs/runtime-v1.md),
[authority](../../neuron/specs/local-authority.md),
[dispatch](../../neuron/specs/worker-dispatch.md), [Mudra exports](../src/lib.rs),
[module contracts](../specs/README.md) and [NSIG1](../specs/neuron-auth.md).
Underlying arithmetic/storage libraries do not by themselves implement these
seven complete Mudra primitives.

## What is ready in Neuron

The local implementation has multiple independent progs under one subject,
revision conflicts, durable resource reservations and parent/child accounting,
unknown-effect recovery, lifecycle operations, and fenced legacy migration.
Publication independently checks the signed proposed transition. Embedded worker
dispatch consumes a durable one-use claim while holding the current grant guard.

The scope is explicit: one root pins one network per database/profile; current
workers use `native-rust/1`, `neuron/host-act/1`, `none/1`, `embedded-sync/1`. Remote
takeover, distributed multi-writer authority and consensus finality need separate
profiles. The [Inf projection](../../neuron/specs/query-projection.md) is a local
read API with `provable() == false`. This does not make local persistence or
authority incomplete; it limits the advertised service.

The current release evidence uses G01–G14 convergence gates. A fresh C01–C58
disposition map required by [conformance](../../neuron/specs/conformance.md)
was not found; the existing C-series map is explicitly historical. Neuron's
worker tests cover sequential interleavings and restart; the specified true
two-worker race needs directly linked concurrent evidence. These are validation
gaps, not reproduced dispatch bugs. Current tests do not establish cryptographic
qualification of their Hemera dependency.

## Four decisions before private production implementation

1. **Authority and key lifecycle.** Specify exact public inputs/private witnesses,
   supported policies, member-hiding, nonce/nullifier scope, accepted root and
   atomic admission. Current NSIG1 keys retain their existing IDs; changing that
   key changes the subject. A stable subject with rotatable authority needs its
   own explicit state/creation/recovery profile. The general separation is already
   in [identity](../specs/identity.md); wire/state rules remain to be instantiated.
2. **Concrete crypto profiles.** Pin hash/proof parameters, encodings, key epochs,
   KDF/context, authenticated encryption and the selected CSIDH implementation.
   `seal` may use a fully specified standard profile or a separately qualified
   native construction. Names such as CSIDH or Goldilocks do not select these
   details. Hemera's current inverse-16 qualification is still
   [open](../../hemera/.claude/plans/production-finalization.md).
3. **Receiving while offline.** Define recoverable note bytes and openings,
   hidden index routing, atomic concurrent appends, retained pending receipts,
   checkpoint coverage/freshness, seed discovery and private spent-state reads.
   The [neuron-prepared proposal](../specs/props/private-recovery-index.md)
   identifies this work correctly. An owner checkpoint covers processed history;
   it cannot account for later receipts until the tail is processed.
4. **Public disclosure.** Define which values/aggregates, query sizes, timing and
   metadata may be exposed. Exact public aggregate deltas can reveal isolated
   contributions. Current [BBG privacy](../../bbg/specs/privacy.md) and Mudra's
   intended hidden-value contract require a common disclosure policy.

There is a real ZK building block: [Zheng's private relation preparation](../../zheng/rs/src/execution/private.rs),
[Trisha's verifier-regenerated Triton checker](../../trisha/rs/ccs.rs) and
[Joy integration](../../joy/rs/zk_execution.rs). Its private-state mode operates
over authenticated **public** tables, currently bounded to 2048 fields. It does
not implement an encrypted wallet database. Public DirectProof is a separate
format that discloses witness-dependent data. The integration must check the
expected authority program/claim, not merely accept a valid generic proof.

Neuron currently verifies NSIG1. Cybergraph's separate
[native storage profile](../../cybergraph/specs/native-storage.md) explicitly
does not supply the private authorization/proof gate. Inf's coverage contract is
specified, while its current BBG source is non-provable; neither storage nor the
query API should be counted as complete private recovery.

The standalone CLI uses an explicitly created local key file and a fresh owner
grant per command. Durable delegated-grant administration, hardware custody and
remote revocation coordination belong to the host composition; this CLI does not
establish those production custody properties.

## Concrete specification and engineering defects

- [Quorum](../specs/quorum.md) hashes polynomial coefficients, then invokes
  "Pedersen-like" share verification and adds commitments to derive a public key.
  No algebra or proof relation makes those operations executable as written.
  Ordinary Shamir is separable; VSS/DKG/threshold decryption need an actual scheme.
- [Place](../specs/place.md) incorrectly excludes claiming a larger RTT: an
  endpoint can delay its response. One anchor also does not uniquely orient an
  embedding. These guarantees need a revised adversarial model and construction.
- [Delay](../specs/delay.md) assumes equivocation requires recomputing the same
  VDF twice; a proof can be reused for the same input and difficulty. A branch-
  bound challenge and explicit selection rule would need their own argument.
- `specs/order.md` is absent although the module index advertises it.
  [neuron-measures](../specs/neuron-measures.md) still equates identity with
  H(secret), unlike the current subject/authority distinction. The crypto table
  in `CLAUDE.md` retains obsolete NIST/Goldilocks and field-VDF descriptions.
- Mudra `prove` currently fails to compile because its `Statement` initializer
  omits `bbg_root`. Repairing that restores an arithmetic `a*b+c` demonstration;
  [prove.rs](../src/proof/prove.rs) explicitly leaves full claim arithmetization
  unfinished. It will not create private authority by itself.
- The legacy CLI claim parser panics on `mudra verify 'addr aéa aa aa'` because
  [claim.rs](../src/claim.rs) slices UTF-8 at byte offsets. Return a malformed-input
  error. This reproduced parser defect is distinct from NSIG1 signature checking.
- Static review of the optional [ECDSA reference](../src/proof/ecdsa.rs) found
  that `r`/`s` are reduced modulo the order before only checking for zero; this
  does not enforce the documented canonical input range. Add pre-reduction range
  rejection and independent negative vectors before using it as a proof reference.
  This finding is outside the default k256-backed NSIG1 path; no NSIG1 forgery
  or exploit of this optional verifier was executed.

## Work sequence

Continue local Neuron integration now, close the two reproduced Mudra engineering
defects, and publish a pinned source closure for the already tested local profile.
In parallel, instantiate one private authority profile and connect its verifier
to atomic admission. Then deliver one complete encrypted-payment/recovery slice:
offline receipt → checkpoint/tail retrieval → verified current state → spend.
Compare it against a complete scan and measure the entire client/server path.
Broader FHE, threshold governance, VDF and location modules can follow their own
requirements. Completing all seven is unnecessary for the first authority slice.
