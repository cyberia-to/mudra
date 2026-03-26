---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# quorum — split a secret, require cooperation

split a secret into k-of-n shares. no single party holds the full secret. k parties must cooperate to act. information-theoretic security (Shamir) plus hash-based verification (VSS via hemera). native Goldilocks field arithmetic.

## interface

```
// Shamir Secret Sharing
split(secret: F_p, k: usize, n: usize) → Vec<Share>
recover(shares: &[Share], k: usize) → Result<F_p, ThresholdError>

// Verifiable Secret Sharing
split_verifiable(secret: F_p, k: usize, n: usize) → (Vec<Share>, Commitment)
verify_share(share: &Share, commitment: &Commitment) → bool

// Distributed Key Generation
dkg_round1(party_id: usize, k: usize, n: usize) → (DKGState, DKGMessage1)
dkg_round2(state: &DKGState, messages: &[DKGMessage1]) → (SecretShare, PublicKey, DKGMessage2)
dkg_finalize(state: &DKGState, messages: &[DKGMessage2]) → Result<(), DKGError>

// Threshold Decryption
partial_decrypt(share: &SecretShare, ciphertext: &Ciphertext) → DecryptionShare
combine_decryptions(shares: &[DecryptionShare], k: usize) → Plaintext
```

## Shamir Secret Sharing

secret s is encoded as the free coefficient of a random polynomial f of degree k-1 over F_p:

```
f(x) = s + a_1·x + a_2·x^2 + ... + a_{k-1}·x^{k-1}

  coefficients a_1..a_{k-1} ← uniform(F_p)
  share_i = (i, f(i))       for i = 1..n

recovery:
  any k shares → Lagrange interpolation → f(0) = s
  fewer than k shares → information-theoretically zero knowledge about s
```

field: Goldilocks (p = 2^64 - 2^32 + 1). Lagrange interpolation is native field arithmetic.

## Verifiable Secret Sharing (VSS)

the dealer commits to the polynomial coefficients via hemera hashes:

```
commitment = [H(a_0), H(a_1), ..., H(a_{k-1})]

verification:
  given share (i, v), check that v is consistent with commitments
  uses Pedersen-like commitment scheme over Goldilocks
  hemera provides the binding hash
```

VSS prevents a malicious dealer from distributing inconsistent shares. each party verifies its own share against the public commitment before accepting.

## Distributed Key Generation (DKG)

n parties collectively generate a shared public key with no trusted dealer. each party contributes entropy. no single party knows the full secret key.

```
round 1:
  each party runs Shamir split on their own random secret
  broadcasts commitment to their polynomial

round 2:
  each party sends share_j to party j (encrypted via seal or stealth)
  each party verifies received shares against commitments (VSS)

finalization:
  each party's secret share = sum of received shares
  public key = sum of all commitments at x=0
  k-of-n parties can sign/decrypt, no single party has full key
```

## Threshold Decryption

given a ciphertext encrypted under the collective public key:

```
1. each party computes partial decryption using their secret share
2. k partial decryptions are combined via Lagrange interpolation
3. result: plaintext, recoverable only with k-of-n cooperation
```

## properties

| property | value |
|----------|-------|
| secrecy | information-theoretic (fewer than k shares reveal nothing) |
| verifiability | hash-based (hemera commitments for VSS) |
| setup | DKG requires no trusted dealer |
| threshold | configurable k-of-n (any k, any n, k ≤ n) |
| field | Goldilocks (native Lagrange interpolation) |

## usage in cyber

- multi-neuron governance: k-of-n neurons must cooperate for privileged operations
- distributed randomness beacon: DKG produces unpredictable, unbiasable randomness for PoUW challenges and Fiat-Shamir challenges
- key recovery: split neuron secret key across devices, recoverable with k-of-n
- threshold hash chains: Poseidon2 hash chains in MPC (Shamir-based), <0.5s at 1ms latency for 3-party threshold

## dependencies

- nebu: Goldilocks field arithmetic (Lagrange interpolation, polynomial evaluation)
- hemera: hash commitments for VSS (Pedersen-like binding)
- mudra::seal or mudra::stealth: encrypted share distribution in DKG round 2
