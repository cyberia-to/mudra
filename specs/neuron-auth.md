# Native secp256k1 statement authentication (NSIG1)

This is the implemented H(compressed SEC1 public key) neuron profile used by the
local neuron authority adapter. It uses the existing Cosmos/ADR-036 primitives.
Proof-native/hash-preimage profiles retain their separate specification and
identities; this profile asserts neither post-quantum security nor network finality.

A statement is a 32-byte content commitment supplied by the owning data codec.
The message is the exact ASCII bytes `cyber:neuron:authority:v1:` followed by the
32 statement bytes. Its ADR-036 signer is the Cosmos address of the compressed
public key under the literal `neuron` HRP. The envelope is exactly 102 bytes:

| Offset | Length | Meaning |
|---|---|---|
| 0 | 5 | ASCII `NSIG1` |
| 5 | 33 | Compressed SEC1 secp256k1 public key |
| 38 | 64 | Existing ADR-036 compact ECDSA signature |

Verification requires exact length/prefix, valid curve key/signature, unchanged
`claim::neuron_of(public)` equal to the expected subject, and the ADR-036 signature
over that exact domain-separated message. It consumes no spell, storage, scheduler,
VM or inference dependency. `neuron::sign` rejects a subject different from the key's
native ID. Verification alone grants no permission; the host/network checks its
current scope and binds the canonical statement to the actual operation.

The signature format moves unchanged from neuron-node into mudra; node's existing
`verify_evidence` API delegates to it. Existing NSIG1 bytes, native IDs, ADR-036
signer strings and messages remain valid. This adds no new signing identity.

The canonical signing function is `neuron::sign`; `sign_statement` remains a
source-compatible alias for existing callers. Custody, permission and durable
release belong to Vault and its host, not this primitive.
