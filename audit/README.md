# Audit and research

Observed results and implementation assessments belong here. They do not change
the accepted protocol specifications.

- [2026-09-12 specification/research release scope](releases/specs-2026-09-12.md)
- [2026-09-16 Neuron/Mudra implementation readiness and design gaps](readiness-2026-09-16.md)
- [Cyber crypto choice and visual assessment](signature-optimality.md)
- [Private recovery: compact discovery, prepared state and sender-assisted indexing](private-recovery/README.md)
- [Five-system comparison: detailed mechanisms, costs and evidence](signature-optimality/comparison-details.md)
- [Mudra design: CSIDH, seal parameters and viewing/spending separation](signature-optimality/mudra-design.md)
- [Neptune Privacy: addresses, recovery and reproducible predicate evidence](signature-optimality/neptune-privacy.md)
- [Quantus signatures, privacy, wallet recovery and Mudra identity readiness](quantus-identity-privacy.md)
- [Earlier local Mudra readiness evidence](mudra-identity-readiness.md)
- [Quantus circuit and chain evidence](quantus-wormhole-evidence.md)
- [Quantus wallet recovery and query evidence](quantus-wallet-evidence.md)
- [Quantus / OpenSSL executable cross-verification](quantus-mldsa-crosscheck/README.md)
- [2026-09-21/22 P1 signal privacy: the author is on the wire (launch row 10)](p1-signal-content-commitment-gap.md)
- [2026-09-21/22 P2 stealth and veil: implementation gap against the tok ledger (launch row 11)](p2-stealth-veil-transfer-gap.md)
- [2026-09-24 genies exists, not yet constant-time for stealth (launch row 11)](p2-stealth-genies-availability-2026-09-24.md)
- [2026-09-21/22 Property 29: valence privacy leaks by construction, and collides with truth-scoring's public-graph assumption](p29-valence-report-leak.md)

The four privacy gap traces above are dated observations. The privacy design they
measured against was replaced on 2026-10-16 by one pool, one proof, one material
(`soft3/proposals/privacy-frozen.md`), which drops the isogeny `stealth` path and
retargets P1 authorship onto `SPEND`.
