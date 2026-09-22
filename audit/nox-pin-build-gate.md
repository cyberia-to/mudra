# cyber-nox / zheng pin: clean checkout build gate

date: 2026-09-22 · revision: e2d1a63 (origin/master) + this branch

## finding

a clean checkout of mudra fails `cargo check --tests` before this branch:
the local `nox` and `zheng` path dependencies resolve at 0.3.0 and 0.4.0
respectively, but `Cargo.toml` pinned the version requirement to `0.1.2`
for both. cargo requires the path package's version to satisfy the
version requirement even when the dependency is optional and not
activated by default features:

```
$ cargo check --tests
error: failed to select a version for the requirement `cyber-nox = "^0.1.2"`
candidate versions found which didn't match: 0.3.0
location searched: nox/rs
required by package `cyber-mudra v0.1.0`
```

this blocked every mudra-owned launch row for the workers, since none of
them could get past dependency resolution.

## fix

bump both pins to match the local crate versions: `nox = "0.3"`,
`zheng = "0.4"`. verified on this branch:

```
$ cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.27s
$ cargo test
test result: ok. 18 passed; 0 failed
test result: ok. 2 passed; 0 failed   (tests/vectors.rs)
```

## remains

the `prove` feature (opt-in, not in default features) is still broken
after this fix — a real API break, not a pin issue:

```
$ cargo check --tests --features prove
error[E0063]: missing field `bbg_root` in initializer of `Statement`
  --> src/proof/prove.rs:36:5
```

nox 0.3's `Statement` gained a `bbg_root` field between 0.1.2 and 0.3;
`src/proof/prove.rs::open_statement` was not updated. this is a
separate, larger slice (understand what nox now binds `bbg_root` to,
then decide what mudra's phase-2e proving pipeline should commit there)
and is out of scope for this build-gate fix.
