# XNT short-address predicate and serialization check

2026-09-12. Companion to the [source assessment](../neptune-privacy.md).
XNT revision: `cc8e8704026c916d545f02aa4d03b5693b6a58dd` (v0.2.7).

The executable copies the released byte-to-field encoding and hash-lock
instruction sequence, using the selected Triton 7.0.0 and twenty-first 1.1.0
dependencies. It supplies three synthetic public byte strings, derives the
same preimages/postimages as the inspected short-address constructor, and
checks VM acceptance against three incorrect witnesses. The strings need not
be valid CTIDH curves: the tested predicate only consumes their derived hashes.
The source assessment establishes how the released address code feeds that
predicate. This isolates the spending relation; it is not a full transaction,
valid-curve generation, STARK proof, network exploit or CTIDH cryptanalysis.

The same executable measures the serialized Generation address field layout,
KEM public key and ciphertext sizes, and the short-address bech32m encodings.
It does not build the complete XNT node or exercise the mobile app.

From this directory:

```sh
cargo run --locked
```

[Recorded output](output.log): three public-derived witnesses accepted, three
wrong witnesses rejected; Generation payload 2,168 B / 3,482 characters;
short payload 64 B / 116 characters; short subaddress 72 B / 130 characters.
All data are synthetic. The committed lockfile fixes the dependency resolution.

Source attribution: the byte encoder and lock instruction sequence are adapted
from XNT's [common.rs](https://github.com/neptuneprivacy/xnt-core/blob/cc8e8704026c916d545f02aa4d03b5693b6a58dd/xnt-core/src/state/wallet/address/common.rs#L163-L182)
and [lock_script.rs](https://github.com/neptuneprivacy/xnt-core/blob/cc8e8704026c916d545f02aa4d03b5693b6a58dd/xnt-core/src/protocol/consensus/transaction/lock_script.rs#L64-L87).
The surrounding synthetic executable was added for this audit. The upstream
[Apache-2.0 license](LICENSE) is included.
