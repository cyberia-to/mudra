# Mudra and neuron identity readiness — local evidence

Observed 2026-09-12 in `/Users/master/cyber`. Research only: no workspace
source, specification, manifest or lockfile was edited. Build output and the
adversarial probe are under `/tmp`. The neuron harness is being changed in
another user thread; these findings describe the observed working tree, not a
finished release or a clean checkout. Mudra HEAD was
`d6eab0d251d7521c65940e1b3c5aab86abcb6af2`; neuron HEAD was
`34bbff9dbf36421a45c5c283e6c1336dcafc997a`, both with pre-existing changes.

## Readiness conclusion

Mudra has usable classical key derivation and ADR-036 signing/verification.
It does not currently supply production-ready post-quantum identity, stable-ID
key rotation/recovery, or a working proof-backed claim verifier. The current
neuron harness has useful separation between subject, network, authority and
request identity, with a default-deny authority port. It has not yet supplied
a Mudra-backed authority implementation in the inspected neuron sources.

| Identity concept | Actual implementation and boundary |
|---|---|
| Neuron subject | `neuron/id/src/lib.rs:9`: `[u8;32]` alias. The alias supplies representation, not authentication. |
| Current cryptographic ID | `mudra/src/claim.rs:92`: `Hemera(compressed_pubkey)`. The helper accepts any byte slice; the caller must validate a compressed SEC1 point. |
| Address | `mudra/src/cosmos.rs:33` and `:42`: RIPEMD160(SHA256(pubkey)), encoded with caller HRP. HRP changes presentation, not the underlying 20-byte account ID. |
| Domain key | `mudra/src/domain.rs:44`: scalar from Hemera(entropy32 || 0 || domain), then secp256k1. Domain normalization is delegated to the caller. |
| Authorization | `neuron/engine/src/neuron.rs:21`: host Authority port. `:24` defaults to denial; `:171` commits subject, network, policy, epoch, prog/invocation, act, kind and statement. Evidence verification is the host's responsibility. |
| Retry identity | `neuron/engine/src/neuron.rs:213`: distinct `neuron/request-id/1` record binding subject and nonce. This is neither an account key nor an authentication proof. |

The accepted current profile is `cyber-secp256k1-hemera-v1`
(`neuron/specs/identity.md:21`). That spec explicitly says changing the public
key changes the subject and retaining the old identity requires the original
key or a separately specified recovery profile (`:37`). Program/policy epoch
changes do not implement recovery of a lost cryptographic key.

## Concrete gaps

1. **The `prove` feature does not compile against the current dependencies.**
   `mudra/src/proof/prove.rs:36` initializes `zheng::Statement` without its
   required `bbg_root` field. The locked test command below fails with E0063.
   Even after repairing that interface, the implemented pipeline proves only
   `a*b+c` (`:82`); full claim arithmetization remains explicitly unimplemented
   (`:18`). Its demo statement sets program/input/output hashes and focus to
   zero (`:35`). This is not an operation-bound authentication proof.

2. **The arithmetic ECDSA reference does not match the native verifier on
   adversarial encodings.** `proof/ecdsa.rs:30` reduces supplied r/s modulo n,
   then only checks for zero. `proof/scalar.rs:55` documents that reduction.
   A synthetic valid `(r,s=1,z=0,Q)` still verifies with the invalid encoding
   `s=n+1`; k256 rejects it. The reference also accepts high-S signatures that
   native k256 verification rejects. `proof/curve.rs:60` reduces an encoded
   SEC1 x coordinate through `Fe::from_bytes`; the invalid encoding `x=p+1`
   is accepted as x=1, while k256 rejects it. These are demonstrated native /
   emulator mismatches. They do not demonstrate forgery of a completed claim
   proof: no such proof implementation was run or found.

3. **Malformed claim text can panic.** `mudra/src/claim.rs:79` slices arbitrary
   UTF-8 input at two-byte offsets before checking that it is ASCII hex.
   `Claim::decode("address 😀 00 00")` panics, contrary to its malformed-input
   `None` contract at `:60`. Lengths are also decoded into unbounded temporary
   vectors before conversion to fixed-size fields.

4. **Stable derivation needs an explicit frozen profile.** Domain spelling
   changes keys: `Example.com` and `example.com` differ; an empty domain is
   accepted. This is consistent with the documented caller normalization
   responsibility (`domain.rs:38`) but requires a single shared normalization
   contract at every adapter. Both native IDs and domain keys call the current
   `hemera::hash` directly. Existing Mudra tests fix Cosmos derivation bytes but
   do not freeze expected native-ID/domain-key bytes. Hemera itself labels the
   active inverse-16 profile experimental and full-round security unestablished
   (`hemera/rs/src/params.rs:32`). This is an unresolved release dependency,
   not evidence that the hash has been broken.

5. **Migration claims are not a universal action envelope.**
   `claim.rs:97` binds a versioned migration tag and target ID; the ADR-036
   document binds the signer address but has empty chain ID (`:111`).
   `claim::verify` accepts only an HRP as external context (`:155`), with no
   genesis, action, sequence, expiry, grant or policy epoch. An application
   needing those properties must sign/verify a canonical action payload and
   enforce current authorization. `verify_arbitrary` explicitly does not bind
   the supplied signer address to the key (`:133`). `claim::create` deliberately
   permits a target ID different from H(pubkey), as a migration binding.

6. **Documentation mixes implemented and proposed identity profiles.**
   `mudra/specs/identity.md:12` defines H(secret), while the current accepted
   neuron profile uses H(pubkey). The Mudra spec still describes a 64-byte
   address (`:43`); current Hemera outputs 32 bytes and NeuronId is 32 bytes.
   Its displayed authentication statement and lock script (`:12`, `:49`)
   contain no action/message binding. Recovery and anonymous proof circuits
   are specification text, not implemented APIs. These distinctions must be
   retained when comparing a new signature scheme with the present runtime.

7. **Custody is outside the present primitive library.** BIP-39 recovery and
   HD derivation exist (`seed.rs:24`, `:30`, `:40`), but no rotation/recovery
   service or vault-backed secret handle is implemented here. The CLI accepts
   mnemonic words through process arguments (`src/bin/mudra.rs:164`, `:175`)
   and prints generated phrases (`:157`). It is not a production custody flow.

Neuron's bounded attachment projection explicitly relies on the host to verify
evidence (`neuron/model/src/identity.rs:160`); its current three tests check
structural identity separation, revision/revocation and rejection of missing
evidence. Passing those tests does not prove signature authorization works.

## Commands and results

Working directory `/Users/master/cyber/mudra`:

```text
cargo test --locked --offline --target-dir /tmp/mudra-id-research-target --lib --test vectors
PASS: 18 library tests, 2 bridge vectors.

cargo test --locked --offline --target-dir /tmp/mudra-id-research-target --no-default-features --lib
PASS: 15 library tests.

cargo test --locked --offline --target-dir /tmp/mudra-id-research-target --features prove --lib --test vectors
FAIL: E0063, missing bbg_root in Statement at src/proof/prove.rs:36.
```

Working directory `/Users/master/cyber/neuron`:

```text
cargo test --locked --offline --target-dir /tmp/neuron-id-research-target -p neuron-id -p neuron-model --test identity
PASS: 3 neuron-model identity integration tests.
```

## External adversarial probe

Exact source: `/tmp/mudra-id-research-probe.rs`.
Preserved copy: [mudra-identity-probe.rs](mudra-identity-probe.rs).
SHA-256: `3ebb1edf3d34bfe23597e2e7a9f0b06016b3eabeeecaf88c624ad2479fb7419b`.
The probe links the successfully built default Mudra library and directly
includes the existing field/scalar/curve/ecdsa source modules through Rust
`#[path]` attributes. It bypasses the unrelated broken `prove.rs` assembly
module only to inspect the native arithmetic reference. It constructs no nox
execution trace and produces/verifies no zheng proof. All keys and data are
public synthetic test values.

Exact invocation:

```sh
rustc --edition=2024 /tmp/mudra-id-research-probe.rs -L dependency=/tmp/mudra-id-research-target/debug/deps --extern mudra=/tmp/mudra-id-research-target/debug/deps/libmudra-d87ce5fbdae2e07f.rlib --extern k256=/tmp/mudra-id-research-target/debug/deps/libk256-e967ef986d6792a6.rlib --extern nebu=/tmp/mudra-id-research-target/debug/deps/libnebu-36464cb45c82fc43.rlib -o /tmp/mudra-id-research-probe
/tmp/mudra-id-research-probe
```

Observed output:

```text
claim_decode_unicode_panics=true
domain_case_changes_native_id=true
empty_domain_and_zero_entropy_accepted=true
out_of_range_s_native_parse_accepts=false
out_of_range_s_emulator_verifies=true
high_s_native_verifies=false
high_s_emulator_verifies=true
noncanonical_pubkey_native_accepts=false
noncanonical_pubkey_emulator_accepts=true
```

The all-zero entropy observation is an API-boundary observation, not a claim
that the API can detect whether arbitrary 32 bytes came from a secure RNG.

Inspected source SHA-256 values:

```text
c2ed006d0d27550fa83c152a47cd9e307e192d68668812baafde984eef19c3db  mudra/src/claim.rs
b42b912e46be3eb6765b982794cee59783ca46dd73cc1ca6f0076aaffd343931  mudra/src/domain.rs
12bb50e1c49c91b1b9ce575e1b143ba3dd6d4e1130af7a0416419ed13561f472  mudra/src/proof/ecdsa.rs
5938992f6019874264da8163f040b6d7280f7b028e4bb5172714ef49c097b56e  mudra/src/proof/curve.rs
11d38ce146e9114a761f297a944a1f54df78a75408f097ecb9d66bb5a4d6ff11  mudra/src/proof/prove.rs
```

## Suggested priorities before implementation

1. Decide whether the durable subject is permanently H(current public key), or
   a stable root whose separately authorized key bindings can rotate. Current
   accepted code/spec implements the former; replacing a key with a different
   signature scheme changes that ID unless an explicit migration/recovery
   contract is added. Keep this decision separate from signature selection.
2. Freeze identity derivation/hash/encoding profiles and vectors; distinguish
   native subject, foreign address, signature key and idempotency key in APIs.
3. Complete the host authority/vault adapter with exact action/network/epoch
   binding and current revocation checks. Preserve the harness's default-deny
   behavior and its ongoing ownership in the other thread.
4. Make public claim decoding bounded and non-panicking; define canonical
   domain input and safe custody entry points.
5. Before calling the proof bridge ready: restore compilation, reject
   noncanonical scalar/point encodings, match low-S policy, then implement
   the actual claim computation with program/input/output/action binding and
   malicious-proof tests. The current multiply-add demonstration is insufficient.
