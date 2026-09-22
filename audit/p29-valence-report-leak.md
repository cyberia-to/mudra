# property 29: valence privacy leaks by construction, and closing it collides with truth-scoring's own spec

Status: gap analysis, 2026-09-21, corrected 2026-09-22. Documentation only, no source changed.
States precisely why the current cyberlink/signal path does not satisfy
launch property 29 (valence privacy), and surfaces an architectural
tension the registry entry does not mention: [[tru]]'s truth-scoring spec
assumes the same field property 29 asks to hide is public.

## the requirement

`cyber/launch.md` places valence privacy in core 4, the truth market:
"[[valence]] privacy the third leg that removes the coordination channel"
(core 6's P2 covers balances and transfers, not valence). Property 29 in
the registry reads "valence privacy hides individual positions and
reports; only aggregates public," owned by mudra and zheng, evidence "same
proof profile as P1." [[tru]]'s strong-truthfulness spec states the same
target in its mechanism table: "only aggregates public; individual
positions and reports hidden" (`tru/specs/strong-truthfulness.md:23`,
`tru@39e0cafc`). The 2026-09-22 decision on `launch.md` ("the wire and
ticket leaks (rows 10, 12, 29) are about the private side, not the edges")
puts a link's valence on the private side: the edge is public, the report
on it is not.

## what exists instead

[valence](../../cybergraph/specs/valence.md) (`cybergraph@0eae7ae9`) fixes
$v \in \{-1, 0, +1\}$ as "the fifth field of a cyberlink" — one of the five
plaintext fields $(p, q, \tau, a, v)$ that make up a link
(`specs/valence.md:1,3`). It is not a separate submission channel with its
own encryption; it rides on the cyberlink itself.

[the durable signal codec](../../foculus/specs/signal-codec.md)
(`foculus@9e32aa68`) confirms this at the wire level, more directly than
for property 10: `specs/signal-codec.md:12` names the field explicitly —
"Links contain neuron/from/to/token[32], amount(u64), `valence(i8)`,
height(u64)." Valence is not incidentally exposed alongside other link
data; it is a named, typed, plaintext byte on the wire, visible to every
relaying node, not only to the link's participants.

This is the same wire property 10's audits examine
(`audit/p1-signal-content-leak.md` on zheng's unmerged `launch/10-p1-signal-content-leak`
branch, zheng#21; `audit/p1-signal-content-commitment-gap.md` here,
mudra#3). Under the P1 wording corrected on 2026-09-22 (edge public,
author private) the plaintext edge itself is by design; valence is the
report riding on the edge, on the private side, so it needs the same
hiding construction P1 needs for the author and position fields — a
proof of validity over a hidden witness, zheng private execution. "Same
proof profile as P1" in the registry evidence is correct; valence needs no
independent primitive.

## a second problem the registry entry does not name

[truth-scoring](../../tru/specs/truth-scoring.md) (`tru@39e0cafc`) is the
spec for how valence is consumed, and it is written against valence being
public. Its mapping table states the meta-prediction $m_i$ IS the valence
field (`specs/truth-scoring.md` "mapping to cyberlinks" section), and then:
"Every cyberlink is simultaneously a structural assertion and a BTS
prediction, in one atomic act. The scoring engine computes $s_i$ for every
neuron from the public graph without any additional input"
(`specs/truth-scoring.md:70`).

That sentence is a design commitment to reading $v$ directly off the
public graph. Hiding $v$ per property 29 does not just need a commitment
and a participant-only opening (the P1 pattern) — it needs the BTS scoring
engine itself to compute $s_i$ over a value it can no longer read in the
clear, which `truth-scoring.md` does not describe. "Only aggregates
public" (the registry's own phrasing for property 29) is in direct tension
with "the scoring engine computes $s_i$ ... from the public graph" as the
spec reads today. One of the two has to give: either truth-scoring gains a
private-input variant (a harder version of the same hiding-witness problem
property 10 already found zheng has no committed construction for), or
property 29's target changes from hiding raw valence to hiding something
coarser that the current public-scoring design can still compute over
(e.g. publishing $v$ but not the identity/stake binding it carries).

## why this is not a signal-codec bug, and not mudra-local either

The valence field's leak is downstream of the same wire format audited for
property 10 — fixing `signal_codec::encode_signal` for $\vec\ell$ in
general fixes valence for free. The scoring-engine tension is new to this
property: it is a `tru` design decision, not a codec or `mudra` gap, and it
means property 29 cannot close by cryptography alone. The spec that
consumes the hidden field has to be rewritten to match, or the property's
target has to be redefined to what public scoring can still support.

## what closing property 29 needs, concretely

1. the general link-content hiding construction from property 10 (commit
   $\vec\ell$, participant-only opening, proof of validity without
   disclosure) — valence needs no separate cryptographic primitive.
2. a decision, recorded on `cyber/launch.md`, on which of the two
   `truth-scoring.md:70` conflicts with: either BTS scoring moves to a
   private-input relation (proving $s_i$'s correctness without revealing
   $v$ to the verifier — itself downstream of the same missing hiding
   construction), or property 29's target is redefined to something
   compatible with public per-link scoring (for example: valence stays
   readable by the scoring engine but unlinkable to the reporting neuron's
   other activity, which is a different, weaker privacy statement than
   "only aggregates public").
3. once 2 is decided, `truth-scoring.md`'s mapping section and its "public
   graph, no additional input" sentence need to be rewritten to match
   whichever profile is chosen — the spec cannot say both today.

## verified

Line citations checked directly against the committed HEAD of each sibling
repo at the revisions named above, read-only, no working-tree changes made
outside this repo. No source changed in this repo; `cargo check --tests`
and `cargo test` don't apply to a documentation-only change and were not
run.

## remains

- decide item 2 above and record it in `cyber/launch.md`'s decisions log;
- once property 10 lands a hiding construction, confirm it covers the
  valence coordinate without a separate protocol;
- rewrite `truth-scoring.md`'s public-graph scoring claim to match the
  chosen profile.

## risk

None — adds one markdown file, no behavior change. Revert is a clean
single commit revert.
