---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# seal — encrypt for a recipient

seal a message so only the intended recipient can open it. lattice-based key encapsulation (Module-RLWE). interactive — the receiver publishes a public key, the sender seals a shared secret under it. NIST standardized as ML-KEM (FIPS 203, 2024). post-quantum secure.

## interface

```
keygen() → (SecretKey, PublicKey)
encap(pk: &PublicKey) → (Ciphertext, SharedSecret)
decap(sk: &SecretKey, ct: &Ciphertext) → Result<SharedSecret, DecapError>
```

## algebra

Module-RLWE over Goldilocks field. same field as hemera, nox, zheng — native arithmetic, no conversion.

```
Ring R = Z_p[x] / (x^64 + 1)       cyclotomic polynomial, degree 64
Module dimension: 4×4 over R
Field: p = 2^64 - 2^32 + 1         Goldilocks
```

## protocol

```
keygen():
  s ← small_distribution(R^4)       secret key: short vector
  A ← uniform(R^{4×4})              public matrix
  e ← error_distribution(R^4)       noise
  b = A·s + e                       public key: noisy product
  return (sk=s, pk=(A, b))

encap(pk=(A, b)):
  r ← small_distribution(R^4)       ephemeral randomness
  e' ← error_distribution(R^4)
  e'' ← error_distribution(R)
  c1 = A^T · r + e'
  c2 = b^T · r + e'' + encode(shared_secret)
  return ((c1, c2), shared_secret)

decap(sk=s, ct=(c1, c2)):
  shared_secret = decode(c2 - s^T · c1)
  return shared_secret
```

security assumption: given A and b = A·s + e, recovering s is computationally hard. the error term masks the secret.

## parameter sets

| set | classical security | public key | ciphertext | shared secret |
|-----|-------------------|-----------|-----------|--------------|
| ML-KEM-512 | 128 bit | 800 B | 768 B | 32 B |
| ML-KEM-768 | 192 bit | 1184 B | 1088 B | 32 B |
| ML-KEM-1024 | 256 bit | 1568 B | 1568 B | 32 B |

## comparison with stealth

| property | seal | stealth |
|----------|------|---------|
| interaction | interactive (receiver key first) | non-interactive (NIKE) |
| quantum security | post-quantum (Module-LWE, proven reduction) | conjectured PQ (isogeny) |
| public key size | 800-1568 B | 64-256 B |
| performance | fast | ~5x slower |
| standardization | NIST FIPS 203 (2024) | research |

seal handles interactive key exchange. stealth handles non-interactive scenarios (stealth addresses, anonymous channels).

## usage in cyber

- private neuron-to-neuron data: receiver publishes lattice public key as a particle, sender encrypts cyberlink metadata
- encrypted spell parameters
- private particle delivery

## dependencies

- nebu: Goldilocks field arithmetic, NTT for polynomial multiplication in R
- hemera: hash for key derivation (shared secret → symmetric key)
