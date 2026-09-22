---
tags: mudra, audit, privacy
date: 2026-09-21
updated: 2026-09-22
---
# P1 signal privacy: the author is on the wire (content-commitment gap, corrected)

Registry property 10 (`cyber/launch.md`, row 10): "P1 signal privacy —
commitment + validity proof, leakage bounded." This audit traces what the
current spec and code actually deliver against that claim, and what
closing it needs.

Corrected 2026-09-22. The first version of this audit (2026-09-21) measured
against the earlier P1 wording, "a signal's content is visible to its
participants; the network sees a commitment and a proof of validity", and
called the plaintext link vector $\vec\ell$ the inverse of P1. `launch.md`
core 6 corrected P1 the next day:

> P1 · the edge is public, the author is private. the network sees that
> particle p links particle q, the axon weight, and the aggregates φ* is
> computed from; it does not see which neuron signed, who holds the
> position, or who owns the energy. corrected 2026-09-22: the earlier
> wording, content visible only to participants, would have hidden the
> edges themselves and left nothing for the collective φ* to run on.

and the 2026-09-22 consolidated-state decision on the same page: "the wire
and ticket leaks (rows 10, 12, 29) are about the private side, not the
edges." Under the corrected P1 the plaintext edges are by design; the gap
is the author.

## what P1 requires

For every signal the network must see the edges (which particle links
which, the axon weight) and a proof that they are valid, and must not
learn which neuron signed them, who holds the position, or who owns the
energy spent. Non-participants see edges and aggregates; authorship,
positions and energy stay with the personal book (`launch.md`, decision
2026-09-22).

## what the spec defines today

[cybergraph's signal spec](../../cybergraph/specs/signal.md)
(`cybergraph@0eae7ae9`, file last changed in `ea1bb1f6`, 2026-08-11)
defines the signal as $(\nu, \mathit{net}, \vec\ell, \Delta\phi^*, \sigma, h_0, h_1)$
(`specs/signal.md:7`). The first field $\nu$ is "signing neuron"
(`specs/signal.md:11`): the author is a plaintext field of the broadcast
tuple. $\vec\ell$ (`specs/signal.md:13`) carries the edges in the clear,
which the corrected P1 permits. $\sigma$ is "a zheng proof covering the
impulse, all conviction box movements, and cyberlink validity against the
current BBG root" (`specs/signal.md:15`) — a validity proof over the
disclosed tuple, including the disclosed $\nu$; the spec's "any verifier
runs `decide(σ)` ... without recomputing anything" (`specs/signal.md:41`)
assumes the verifier holds $\nu$ and $\vec\ell$ in the clear.

[foculus's durable signal codec](../../foculus/specs/signal-codec.md)
(`foculus@9e32aa68`) fixes the same leak at the byte level: "Links contain
neuron/from/to/token[32], amount(u64), valence(i8), height(u64)"
(`specs/signal-codec.md:12`). Every link on the wire names its neuron.

So the signal as specified broadcasts the author with every edge, to every
relaying node. Under the corrected P1 this, not the plaintext $\vec\ell$,
is the gap: the edge is public as required, and so is the author, which
is forbidden.

mudra already specifies the primitive the fix needs. `identity.md`
§"private authority" (`specs/identity.md:88-100`): "a public permanent
subject identifier links its actions. the private profile proves
membership and policy satisfaction against a committed authority set
while keeping the member, ownership opening and private values hidden."
That is authorship hiding for an action, with nullifiers for replay and an
explicit leakage contract for aggregates — P1's requirement, as a spec
with no implementation in `mudra/src` (modules: `claim, cosmos, domain,
neuron, proof, spell`). What is missing is the binding: neither
`signal.md` nor `signal-codec.md` uses the private profile in place of
$\nu$ / `neuron[32]`, and no spec says which committed authority set a
signal proves membership against. `seal.md` (lattice KEM for a named
recipient) and `private-recovery.md` (payment-note discovery) do not
address it.

## what the proving backend can do today

Closing the gap needs a zheng proof that verifies validity and authorship
(membership in the committed authority set, signature under a hidden key)
without the verifier holding the author — private execution over a hidden
witness. zheng's private-execution audit, `zheng/audit/private-execution-release.md`,
states this directly: "Zheng's current direct dependencies provide no
reviewed private execution backend or complete blinding protocol"
(line 29 of the file as on disk 2026-09-22). The existing Spartan
implementation sends witness-dependent sumcheck polynomials and
evaluations in the clear; swapping in a hiding PCS alone does not blind
those messages. Four candidate routes are surveyed there (RISC Zero guest,
Triton-native interpreter, native hiding STARK stack, upstream
Spartan/Nova port), none complete, none reviewed.

Citation status, corrected 2026-09-22: the first version of this audit
cited that file at commit `8b19098f`. No such commit exists in the zheng
repository or on its remote; the file is untracked in the zheng working
tree (`git -C zheng status` lists `audit/private-execution-release.md` as
`??`). The quote above is from the file on disk and is not yet committed;
it becomes a verifiable citation when zheng commits it.

`lens::Commitment` (Brakedown, re-exported in `zheng/rs/src/types.rs:10`,
used in `folding/decide.rs`) exists as an internal witness-binding
primitive inside zheng's own polynomial commitment scheme, but nothing
wraps a signal's authorship with it, and using it that way would still
need the hiding-protocol work above to keep the proof itself from leaking
the witness.

## conclusion

P1 is blocked on two undone pieces, in order:

1. a cybergraph/foculus spec clause binding `identity.md`'s private
   authority profile into the signal: the `neuron[32]` per link and the
   top-level $\nu$ replaced by a membership proof against a named committed
   authority set, the nullifier scope, and the changed verifier view; and
   the same for the position and energy fields (`amount`, the ICBS
   position) that the corrected P1 also hides;
2. zheng's private execution / blinding protocol, already tracked as an
   open release requirement independent of P1 — the proof has to certify
   validity and authorship over a hidden witness.

Neither is a 2026-09-21 code change. This audit is the evidence entry for
property 10; the two follow-ups above are its next slices, and the second
one is shared with zheng's own private-execution release gate rather than
being new scope. The plaintext $\vec\ell$ this audit first flagged is not a
gap under the corrected P1 and should not be worked on.
