---
tags: mudra, audit, privacy
date: 2026-09-21
---
# P1 signal privacy: content-commitment gap

Registry property 10 (`cyber/launch.md`): "a signal's content is visible to
its participants; the network sees a commitment and a proof of validity."
This audit traces what the current spec and code actually deliver against
that claim, and what closing it needs.

## what P1 requires

The network must see two things for a signal: a hiding commitment to its
content, and a Zheng proof that the committed content is valid. Any node
that is not a participant in the signal learns nothing about the linked
particles, the impulse shape, or the conviction box movements beyond what
the commitment and proof are designed to leak.

## what the spec defines today

[cybergraph's signal spec](../../cybergraph/specs/signal.md)
(`ea1bb1f6e825824d051dc0f5ffa8108e52b0f268`, 2026-08-11) defines the signal
as $(\nu, \mathit{net}, \vec\ell, \Delta\phi^*, \sigma, h_0, h_1)$. $\vec\ell$
— the actual cyberlinks — is a plain field of the broadcast tuple, not a
commitment to it. $\sigma$ is described as "a zheng proof covering the
impulse, all conviction box movements, and cyberlink validity against the
current BBG root" — a validity proof over disclosed content, not a proof
of a hidden commitment's validity. The spec's own wording — "one proof for
everything... any verifier runs `decide(σ)` ... without recomputing
anything" — depends on the verifier holding $\vec\ell$ and $\Delta\phi^*$ in
the clear to check them against $\sigma$.

So the signal as specified broadcasts content in plaintext to every node,
non-participants included, with a proof layered on top of the disclosed
data. This is the inverse of P1: today the network sees the content and a
proof of validity, not a commitment and a proof of validity.

No mudra spec defines a signal-content commitment interface. `identity.md`
covers action-authentication and authority privacy, `seal.md` covers
lattice-KEM confidentiality for a named recipient, `private-recovery.md`
covers payment-note recovery. None of the three propose or bind a hiding
commitment for the $\vec\ell / \Delta\phi^*$ pair that non-participants
would see in place of plaintext.

## what the proving backend can do today

Closing the gap needs a zheng proof that verifies against a commitment
without requiring the verifier to hold the opening. [zheng's private
execution audit](../../zheng/audit/private-execution-release.md)
(`8b19098fd394c459ba01dd98cb6081d469dc27c7`, 2026-09-16) states this
directly: "Zheng's current direct dependencies provide no reviewed private
execution backend or complete blinding protocol." The existing Spartan
implementation sends witness-dependent sumcheck polynomials and evaluations
in the clear; swapping in a hiding PCS alone does not blind those messages.
Four candidate routes are surveyed there (RISC Zero guest, Triton-native
interpreter, native hiding STARK stack, upstream Spartan/Nova port), none
complete, none reviewed.

`lens::Commitment` (Brakedown, used in `zheng/rs/src/types.rs` and
`folding/decide.rs`) exists as an internal witness-binding primitive inside
zheng's own polynomial commitment scheme, but nothing wraps signal or
cyberlink content with it, and using it that way would still need the
hiding-protocol work above to keep the proof itself from leaking the
witness.

## conclusion

P1 for signal content is blocked on two undone pieces, in order:

1. a mudra or cybergraph spec clause defining the signal content-commitment
   interface — what gets committed, what binding/hiding assumption it
   carries, and how a non-participant verifier's view changes;
2. zheng's private execution / blinding protocol, already tracked as an
   open release requirement independent of P1.

Neither is a 2026-09-21 code change. This audit is the evidence entry for
property 10; the two follow-ups above are its next slices, and the second
one is shared with zheng's own private-execution release gate rather than
being new scope.
