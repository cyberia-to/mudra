---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# stealth — agree on a secret without contact

stealth provides isogeny-based non-interactive key agreement. two parties with
authentic public keys derive a shared secret without a message round-trip.
CSIDH supplies the commutative class-group action; a concrete profile selects
its parameters, implementation family, encoding and key derivation.

## interface

```text
keygen(profile, randomness) → (SecretKey, PublicKey)
agree(profile, sk, peer_pk, context) → Result<SharedSecret, AgreementError>
```

keys carry a profile/version. agreement validates the peer key and rejects
incompatible profiles and invalid encodings before using the secret.
context binds the protocol purpose, network, participants and key epochs.
its encoding and the shared-secret KDF are fixed by the selected profile.

## mechanism

```text
base curve: E0
Alice: secret a, public A = [a] E0
Bob:   secret b, public B = [b] E0
shared action result: [a] B = [b] A
key: profile_KDF(canonical_shared_result, context)
```

the profile specifies whether the shared result is a curve coefficient,
j-invariant or another canonical representative. callers cannot interchange
these encodings. agreement outputs purpose-separated encryption/MAC keys;
the raw action result is not used directly as a key.

## static agreement and private delivery

static-static agreement uses the two existing public keys. it repeats the
pairwise secret until keys or context change. it supplies neither forward
secrecy nor traffic anonymity by itself.

per-payment delivery uses sender ephemeral secret r:

```text
recipient publishes B = [b] E0
sender publishes R = [r] E0
sender derives [r] B; recipient derives [b] R
```

the sender publishes R, keeps r secret, and authenticates the encrypted note
and its association with the output commitment. packet formats and admission
rules follow [private recovery](private-recovery.md).

the sender also knows this shared secret. exclusive spending authority uses
an independent secret/policy under [identity](identity.md). a shared-secret
hash alone cannot establish the recipient's exclusive right to spend.

## profile qualification

a profile fixes all of:

- base curve, prime, action primes, secret distribution and effective keyspace;
- exact public/secret/shared-value encoding and validation algorithm;
- constant-time algorithm, randomness requirements and physical-attack scope;
- KDF, context encoding, key derivation/rotation and error handling;
- classical and quantum attack-resource model and parameter estimates;
- measured keygen, validation, action and per-candidate scan costs.

prime width is not a security-bit count. a 64-byte CSIDH-512 curve encoding
does not by itself establish a production quantum-security level. larger
profiles may require additional encoded data beyond the curve coefficient.
wire sizes come from the exact profile.

dCTIDH means deterministic CTIDH. that construction uses dummy operations;
dummy-free variants require their own algorithm and security analysis.
constant-time execution addresses timing leakage within its stated model.

parameter selection remains explicit. the module does not designate a
production profile from a family name or an unsupported size/security table.
[Research and parameter evidence](../audit/signature-optimality/mudra-design.md#stealth-distinguish-variants-parameters-and-measurements).

## privacy and lifecycle

- authenticate recipient public-key bindings and their epochs before delivery.
- derive independent discovery, payload, channel and spending material.
- retain the view-key epochs required by the advertised recovery interval.
  erasing a key changes historical recovery and late-payment behavior.
- shared-secret tags require agreement first unless another qualified protocol
  filters candidates. they do not automatically remove per-output scan cost.
- receiver tags, public directory lookups, packet sizes, timing and retries
  obey the enclosing protocol's explicit leakage policy.

## dependencies

- genies: field arithmetic, validated commutative group action;
- hemera: profile-selected key derivation and domain separation;
- identity and private recovery: authenticated key bindings, ownership and
  end-to-end delivery requirements.
