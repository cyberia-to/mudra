---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# stealth — agree on a secret without contact

two neurons derive a shared secret from public graph data — no message round-trip needed. isogeny-based non-interactive key exchange (dCTIDH/CSIDH). conjectured post-quantum security. enables stealth addresses and anonymous channels.

## interface

```
keygen() → (SecretKey, PublicKey)
agree(sk: &SecretKey, pk: &PublicKey) → SharedSecret
```

two calls, no interaction. Alice computes `agree(sk_alice, pk_bob)`. Bob computes `agree(sk_bob, pk_alice)`. both arrive at the same shared secret via commutativity of the class group action.

## mechanism

a class group acts on supersingular elliptic curves over F_p. each secret key is a vector of integer exponents. each public key is the resulting curve.

```
Setup:
  E₀: supersingular elliptic curve over F_p
  Class group action: [a] · E₀ = E_a    (secret isogeny walk)

keygen():
  a ← secret_distribution(Z^ℓ)     secret: vector of small integers
  E_a = [a] · E₀                   public: resulting curve
  return (sk=a, pk=E_a)

agree(sk=a, pk=E_b):
  shared = [a] · E_b = [a] · [b] · E₀
  return H(shared)                  hash the j-invariant

Commutativity:
  [a]·[b]·E₀ = [b]·[a]·E₀         both parties get the same curve
```

## why dCTIDH over CSIDH

original CSIDH leaks timing information through variable-time isogeny computation. dCTIDH uses dummy isogenies and constant-time arithmetic to resist side-channel attacks. "d" = dummy-free (division-based approach). "CT" = constant-time.

## parameters

| variant | classical security | public key | status |
|---------|-------------------|-----------|--------|
| dCTIDH-512 | ~64 bit | 64 B | research |
| dCTIDH-1024 | ~128 bit | 128 B | research |
| dCTIDH-2048 | ~256 bit | 256 B | research |

public keys are remarkably compact: 64-256 bytes vs 800-1568 bytes for lattice KEM.

## security

the isogeny assumption is less studied than lattice assumptions. SIDH was broken in 2022, though CSIDH survived those attacks (different algebraic structure — commutative group action vs non-commutative). active research area, not yet standardized.

## usage in cyber

- stealth addresses: sender creates a cyberlink detectable only by intended recipient, no prior communication
- non-interactive key exchange: two neurons derive shared secret from public graph data
- anonymous channels: shared secret reveals nothing about which neurons communicate

## dependencies

- nebu: Goldilocks field arithmetic (curve arithmetic over F_p)
- hemera: hash for shared secret derivation (j-invariant → symmetric key)
