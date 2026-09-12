# Quantus / OpenSSL ML-DSA cross-verification

Research executable, observed 2026-09-12. It is an independent Cargo workspace;
Mudra's production dependency graph is unchanged. Quantus is pinned to
`3b1464f751d536aba023a53df2a4f4533ab94f62` (`dilithium` 4.1.1).

Run with OpenSSL >= 3.5 on PATH:

```text
cargo run --locked --release --manifest-path audit/quantus-mldsa-crosscheck/Cargo.toml --target-dir /tmp/mudra-quantus-crosscheck-target
cargo clippy --locked --manifest-path audit/quantus-mldsa-crosscheck/Cargo.toml --all-targets --target-dir /tmp/mudra-quantus-crosscheck-target -- -D warnings
```

The executable uses public synthetic seeds only, creates a fresh scratch
directory and removes it on success. A failure leaves test artifacts for
inspection. No real wallet, credentials, funds or remote RPC are involved.

## Observed result

```text
OpenSSL 3.6.2 7 Apr 2026 (Library: OpenSSL 3.6.2 7 Apr 2026)
ML-DSA-65: 27 seed/message/context cases passed; pk=1952 signature=3309 bytes
ML-DSA-87: 27 seed/message/context cases passed; pk=2592 signature=4627 bytes
PASS: 54 cases; both signing directions, deterministic byte equality, hedged signing, negative mutations. This is interoperability evidence, not a security proof.
```

Clippy completed with no warnings. Rust: 1.95.0, macOS arm64.

For each parameter set: three seeds, three messages (1, 24, 4096 bytes) and
three contexts (empty, application context, 255 arbitrary bytes). Checks:

- Exact public-key equality from the same FIPS 204 seed.
- Exact deterministic signature equality across implementations.
- Quantus signatures verified by OpenSSL; OpenSSL signatures verified by Quantus.
- Quantus hedged and OpenSSL randomized signatures verified by the other backend.
- Changed context/message, signature bit flip, truncated and appended signatures
  rejected by both backends; wrong public key rejected by Quantus.
- 256-byte context rejected by Quantus; absent and empty contexts equivalent.
- Empty-message round trips exercised locally only: OpenSSL's `pkeyutl` 3.6.2
  rejects an empty input file before signing/verifying. The original attempt
  failed with `Could not allocate 0 bytes for oneshot sign/verify buffer`.

This tests the external **pure ML-DSA** interface, including its FIPS context
prefix. It does not test HashML-DSA, HD derivation, threshold signatures,
Substrate transaction encoding, hardware leakage, mobile performance or all
possible adversarial inputs. Mudra currently has no ML-DSA implementation;
this compares Quantus with OpenSSL, not an existing Mudra PQ signer.

Source references: [Quantus frontend](https://github.com/Quantus-Network/qp-rusty-crystals/blob/3b1464f751d536aba023a53df2a4f4533ab94f62/dilithium/src/frontend.rs),
[OpenSSL signature API](https://docs.openssl.org/3.6/man7/EVP_SIGNATURE-ML-DSA/),
[OpenSSL key API](https://docs.openssl.org/3.6/man7/EVP_PKEY-ML-DSA/).

The upstream repository root contains Apache-2.0 text, while the actual
`dilithium` manifest and its own LICENSE declare GPL-3.0. This research
dependency is isolated here; production adoption needs a resolved license
choice at the crate boundary.
