---
tags: mudra, audit, privacy
date: 2026-09-21
---
# P2 transfer privacy: stealth and veil implementation gap

Registry property 11 (`cyber/launch.md`): "P2 stealth and veil | mudra, tok
| open | stealth.md, veil.md, transfer vectors." P2 from the launch cores:
"balances and transfers are private by mudra stealth addresses and veil."
This audit traces what's specified against what tok's ledger and mudra's
crypto modules actually hold today, mirroring the [P1 content-commitment
audit](p1-signal-content-commitment-gap.md) (launch #10) for the balance
side of privacy.

## what P2 requires

A transfer's recipient is addressed by a stealth key so an outside observer
cannot link the payment to the recipient's public identity, and the balance
itself is held under veil (FHE) so on-chain state does not disclose amounts
in the clear.

## what tok holds today

[tok's ledger](../../tok/rs/src/ledger.rs) (`e27da0dff72b948418a5e40a5ed6290c0fd167f3`,
2026-09-16) keys balances directly by `(NeuronId, TokenId) → u64`:
`balances: BTreeMap<(NeuronId, TokenId), u64>`, read back in plain by
`balance()`. `NeuronId` is `[u8; 32]` ([neuron/id/src/lib.rs:9](../../neuron/id/src/lib.rs)),
the same public identifier used everywhere else a neuron is referenced —
not a per-payment stealth-derived address. Every balance and every mutation
(`transfer`, mint, burn — landed in launch #13, #26, #30, #32, #33) is a
plain `u64` against a plain public key. There is no ciphertext type, no
stealth-address column, nothing between the public `NeuronId` and the
integer amount.

## what mudra specifies and holds

[identity.md](../specs/identity.md) names both modules as the delivery
layer: "stealth derives pairwise secrets by a validated commutative action,
including per-payment ephemeral delivery" and "veil supplies profile-
specific private computation," with private-recovery.md handling discovery
and key lifecycle on top. The specs ([stealth.md](../specs/stealth.md),
[veil.md](../specs/veil.md)) are complete interface and mechanism documents:
stealth defines CSIDH-based non-interactive key agreement
(`keygen`/`agree`) over a commutative group action with per-payment
ephemeral delivery (`R = [r] E0`); veil defines a TFHE-style scheme over
Goldilocks NTT rings (`keygen`/`encrypt`/`add`/`mul`/`bootstrap`).

Neither has an implementation in `mudra/src/`: the module list there is
`claim, cosmos, domain, neuron, proof, spell` — no `stealth` or `veil`
module. Stealth's dependency, the isogeny field crate `genies` (F_q,
commutative group action — named in `mudra/CLAUDE.md`'s companion-repo
table at `~/git/genies/`), does not exist in this checkout. Veil's
dependency, `nebu` (Goldilocks NTT), exists and is used elsewhere in the
stack, but veil itself needs a complete bootstrapping/noise-management
scheme on top of NTT multiplication, which `veil.md` explicitly flags as
unqualified: "this specification assigns no unmeasured Hemera/FHE
speedup," and parameter qualification (security, noise, key-switching,
bootstrapping) is listed as still open in the spec itself.

## conclusion

P2 is blocked on the same shape of gap as P1, on the balance side instead
of the signal side: tok's ledger is fully public today, and both privacy
primitives it would need are complete specs with zero implementation,
one of them (stealth) missing its underlying field crate entirely. Closing
this row needs, in order: (1) `genies` as a buildable crate implementing
the CSIDH group action stealth.md specifies, (2) a qualified veil profile
(parameters + bootstrap correctness, per veil.md's own open list), (3) a
stealth-address column and ciphertext balance type in tok's ledger, (4)
transfer vectors exercising the new type. None of these four is a
2026-09-21 code change; this audit is the evidence entry for property 11,
and item 1 is the smallest of the four — the next tractable slice once
someone starts the `genies` crate.
