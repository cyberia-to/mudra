# genies now exists; it is not yet safe as stealth's dependency

date: 2026-09-24 · mudra revision e2d1a63 (origin/master) · strata revision 56aedb2 (origin/main)

## finding

`specs/stealth.md` names `genies` (field arithmetic, validated commutative
group action) as a dependency and requires, in its own profile-qualification
list, "constant-time algorithm, randomness requirements and physical-attack
scope" before a profile can be called qualified. An earlier read of this row
found `genies` absent from the checkout entirely. That is out of date: the
crate now exists, built for `lens`'s Porphyry commitment backend
(`strata/genies/rs/`, workspace member, `strata-genies` on crates.io), and its
own module doc names the exact use this row needs: "Post-quantum privacy
primitives: stealth addresses, VRF, threshold protocols, blind signatures,
anonymous credentials" (`strata/genies/rs/src/lib.rs:9`).

It implements CSIDH-512 for real: `action::Ideal` (a 74-prime exponent
vector, `strata/genies/rs/src/action.rs:33`), `action::action(ideal, curve)`
computing the class-group action by chained ℓ-isogenies
(`action.rs:114`), and `action::dh` exposing it as Diffie-Hellman
(`action.rs:178`) — exactly `stealth.md`'s mechanism section (`[a] E0`,
commutative action, shared result via `dh(a, dh(b, E0)) = dh(b, dh(a, E0))`).

It is not the qualified profile `stealth.md` requires. `action()`'s isogeny
count per prime is bounded by `|ideal.exponents[i]|` — secret-dependent
(`action.rs:122`, `while e != 0`) — and `XSampler::next` walks
`x = 0, 1, 2, ...` branching on `Fq::legendre(&rhs)` until it finds a point
of the sign the current secret exponent needs (`action.rs:101-140`): both the
number of loop iterations and which branch is taken depend on the secret
ideal. This is the textbook (non-constant-time) CSIDH shape the field is
named for, not the "dCTIDH" (deterministic, dummy-op-hardened) variant
`stealth.md` itself distinguishes and requires ("dCTIDH means deterministic
CTIDH... dummy-free variants require their own algorithm and security
analysis"). It is fine as `lens::Porphyry`'s use of the same field
(Fiat-Shamir challenges over `Fq`, `lens/porphyry/src/lib.rs`), which is
public-transcript arithmetic with nothing secret to leak. It is unsafe to
wrap directly as `stealth::keygen`/`agree`: the secret ideal is exactly what
a timing side-channel over `action()` would recover.

## verified

```
$ git -C strata rev-parse origin/main
56aedb2d12b3126c601eb333419136d403614dbb
$ ls strata/genies/rs/src
action.rs  algebra_impl.rs  curve.rs  encoding.rs  fq.rs  isogeny.rs  lib.rs  ops.rs  vectors.rs
$ grep -n "while e != 0\|fn action\|fn dh" strata/genies/rs/src/action.rs
122:        while e != 0 {
114:pub fn action(ideal: &Ideal, curve: &MontCurve) -> MontCurve {
174:pub fn dh(secret: &Ideal, peer_curve: &MontCurve) -> MontCurve {
```

mudra's own gate, for completeness: `cargo check --tests` on this checkout
fails before reaching any stealth-related code —
`failed to select a version for the requirement `cyber-nox = "^0.1.2"``
against the sibling checkout's `0.3.0` — the pre-existing, unrelated row-38
pin gap (open, mudra#6). This PR touches only this new audit file; no
`Cargo.toml`, `Cargo.lock` or `src/` change.

## remains

Row 11 needs, in order: (1) a constant-time profile of `genies::action` —
dummy isogeny steps so iteration count and branch taken no longer depend on
the secret ideal, a strata-side cryptographic engineering task with its own
security review, out of scope for mudra; (2) once qualified, `stealth.md`'s
`keygen`/`agree` wrapped in `mudra` over that profile; (3) veil's parameter
qualification (`veil.md`'s own open list); (4) the ciphertext/stealth-address
column in `tok`'s ledger; (5) transfer vectors. None of these are attempted
here — this note only corrects which of them is actually blocking, so the
next slice is not spent re-implementing a field that already exists.

## risk

docs-only: one new file, no source, manifest or spec change. Revert is a
plain file removal.
