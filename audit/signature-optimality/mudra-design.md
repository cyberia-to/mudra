# Mudra's strategic cryptographic design

2026-09-12. Research only: no source/spec edits, no implementation-readiness or compile-bug criterion, and no replacement signature recommendation.

## Preserve the architecture that was chosen

Mudra's intended split is **proof-based authority + recipient encryption + non-interactive shared secrets**, not a competition to choose one primitive for everything. `mudra/README.md:7–20` assigns authentication, integrity, randomness and metering to Zheng; `mudra/specs/identity.md:9–16` describes Hemera preimage ownership. `mudra/specs/README.md:12–20` lists seven roles: seal, stealth, veil, quorum, delay, order and place.

| Role | Current chosen design | Distinct purpose | Relevant cost | Decision still needed |
|---|---|---|---|---|
| Native authority | Hemera(secret) + Zheng lock-condition proof | Publicly checkable authorization and policy predicates | Proof generation/verification; separate from key agreement | Exact action-bound, versioned identity/proof profile |
| seal | Lattice KEM, described both as ML-KEM and custom Goldilocks Module-RLWE | Fresh secret delivery to a recipient with a published key | KEM ciphertext plus encapsulation/decapsulation | Standard ML-KEM or separately specified native profile |
| stealth | CSIDH/CTIDH family NIKE | Derive a shared secret from one's secret and another public key | Group action and peer-key validation; scan cost per candidate | Exact prime, keyspace, attack model, variant and encoding |
| Private balances | BBG jali RLWE commitments; Ikat for A_live, Brakedown for N_live | Hide values and separate unspent commitments from nullifiers | Spec claims 4KB/entry, plus state proofs | Full ring/encryption/commitment encoding and security parameters |
| Private discovery | BBG one-time genies address and recipient chain scan | Locate incoming outputs without publishing recipient identity | Scan, metadata download and retained key epochs | Ephemeral announcement, view/spend separation and scan budget |

The last two rows are current design, not an alternative imported from another chain: `bbg/specs/privacy.md:105–117`, `:151–184`, `:210–224`.

## seal: a consequential parameter ambiguity

`mudra/specs/seal.md:20–25` selects degree **64**, rank **4**, modulus **2^64−2^32+1**. Lines 55–59 then give standard ML-KEM sizes. Those describe different schemes. FIPS 203 fixes degree **256**, modulus **3329**, ranks **2/3/4**, prescribed noise/compression/hash algorithms and the CCA-secure KEM construction. Its approved sizes are:

| Standard profile | Public key | Decapsulation key | Ciphertext | Shared secret |
|---|---:|---:|---:|---:|
| ML-KEM-512 | 800B | 1632B | 768B | 32B |
| ML-KEM-768 | 1184B | 2400B | 1088B | 32B |
| ML-KEM-1024 | 1568B | 3168B | 1568B | 32B |

These are standard profile values, not measured or justified values for Mudra's Goldilocks construction. See [FIPS 203, §8 Tables 2–3 and §6](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf).

The strategic choice is explicit: preserve standard interoperability, or define a native-field construction with its own distributions, encoding, failure probability, attack estimates and chosen-ciphertext transform. Sharing field arithmetic is a useful design goal; it does not transfer another parameter set's security estimate or wire size.

Calling seal "interactive because the receiver publishes a key first" (`seal.md:8`, `:65`) obscures the real distinction. With a prepublished recipient key, a sender can encapsulate and send ciphertext while the recipient is offline. NIKE also needs access to the peer's public key; its extra property is deriving the same pairwise secret from the two existing keypairs **without an additional encapsulation ciphertext**. This is a protocol-flow inference from [FIPS 203's KEM flow](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.203.pdf) and the [CSIDH construction](https://eprint.iacr.org/2018/383). It does not make KEM and NIKE interchangeable in every authenticated handshake.

## stealth: distinguish variants, parameters and measurements

`mudra/specs/stealth.md:43` expands the prefix incorrectly. The authors' **dCTIDH means deterministic CTIDH**, combines deterministic evaluation with CTIDH batching, and explicitly still uses dummy operations. Dummy-free dCSIDH is a different implementation approach; later [Hardened CTIDH research, September 2025](https://arxiv.org/abs/2509.12877) removes dummy operations and addresses fault concerns. Constant-time alone is not a blanket physical-attack guarantee. See [dCTIDH, Remark 1](https://eprint.iacr.org/2025/107).

The local 512→64, 1024→128, 2048→256 "classical security" table (`stealth.md:47–51`) cannot stand as a profile definition. Prime size, keyspace and quantum attack model are separate inputs. In the primary high-security study, **2048/4096-bit primes target NIST level 1** under respectively aggressive/conservative assumptions: attacker depth bounded by 2^60/2^80. The estimates deliberately omit quantum group-action oracle cost, potentially understating security; they are qualified resource models, not a proof of exact strength. The 2048-bit example uses about 2^221 secret keys; 4096 uses 2^256. [Campos et al. 2024, §2.4–3.1, Table 1](https://thomwiggers.nl/publications/secsidh/secsidh.pdf).

Published baseline values, with units kept intact:

| Primitive/profile | Operation | Published cost | Qualification |
|---|---|---:|---|
| CTIDH-512, 2^256 keyspace | Group action / public-key computation | 125.53 million cycles ≈41.84ms | 3GHz Xeon E3-1220v5, Skylake, TurboBoost disabled |
| Same CTIDH-512 | Peer-key validation + group action | 129.64 million cycles ≈43.21ms | Same machine; distinct from action-only |
| dCTIDH-2048-194 | Group-action evaluation | 1,409.47 million cycles | Median of 25,000 runs, Intel i7-6700 Skylake |
| dCTIDH-2048-205 | Group-action evaluation | 1,430.31 million cycles | Same experiment; not a whole wallet/keygen benchmark |
| ML-KEM-768, mlkem-native | Keygen / encaps / decaps | 21,169 / 23,498 / 30,037 cycles | Mac Mini M1 (2020), upstream 2026-09-10 dataset |

CTIDH values: [authors' CHES 2021 slides, slide 39](https://iacr.org/submit/files/slides/2021/ches/ches2021/31320/slides.pdf), [paper Table 2](https://eprint.iacr.org/2021/633). Milliseconds are arithmetic conversion at the stated 3GHz. dCTIDH values: [2025 paper Table 1](https://eprint.iacr.org/2025/107.pdf). The latter paper studies 2048-bit parameters for its level-1 model; these are not measurements of Mudra or of the later XNT 512-bit implementation.

ML-KEM values: [official benchmark data](https://pq-code-package.github.io/mlkem-native/dev/bench/data.js), revision [`0924122d0e92b2d682fab5d5e5e2593ec84c8a1f`](https://github.com/pq-code-package/mlkem-native/commit/0924122d0e92b2d682fab5d5e5e2593ec84c8a1f). The [benchmark documentation](https://pq-code-package.github.io/mlkem-native/dev/bench/) specifies the FIPS-203 API, packed keys and validation; x86 measurements have SMT/TurboBoost enabled. These cross-machine data do not justify a precise speed ratio. They provide no support for the local "stealth ~5x slower than ML-KEM" claim (`seal.md:68`, `identity.md:244`).

The public-key encoding also matters: original CSIDH's single 512-bit curve coefficient is **64 bytes**. The actual dCTIDH authors' 2047-bit profiles use **264-byte public keys**, comprising a 256-byte coefficient plus an 8-byte seed for full-torsion point generation. This is explicit in [`src/secsidh.h:10,21,32`](https://github.com/PaZeZeVaAt/dCTIDH/blob/ec5df98c28248b6a5d47faac07737a37d17328db/src/secsidh.h) and [`src/CTIDH/ctidh.h:37–40`](https://github.com/PaZeZeVaAt/dCTIDH/blob/ec5df98c28248b6a5d47faac07737a37d17328db/src/CTIDH/ctidh.h). "dCTIDH-2048 = 256-byte wire key" omits required variant-specific data.

## What NIKE supplies to the wallet, and what the design must add

The following are design inferences from the commutative action in `mudra/specs/stealth.md:23–38`, applied to BBG's current scan sketch:

- With recipient public B=[b]E0 and sender ephemeral secret r, publish **R=[r]E0**, not r. Sender computes [r]B; recipient computes [b]R. BBG's `privacy.md:220` instead refers to r without saying how the recipient obtains the necessary announcement. Making r public permits observers to run the same derivation against candidate recipients; keeping r secret leaves the written scan formula unspecified.
- A shared secret can derive encryption/MAC keys with explicit purpose, network, participants, nonce and epoch binding. A channel MAC proves possession to the other channel participant; because that participant also holds the MAC key, it is not publicly transferable evidence of who authorized an action. Keep that role separate from the public Zheng authority proof.
- **The sender also knows the NIKE secret.** A hash of that secret alone cannot be the recipient's exclusive spending credential. Specify distinct discovery and spending authority, and a verifier-checkable ownership relation. "Applying some secret to this valid curve gives a valid curve" (`privacy.md:223`) does not identify the intended owner.
- Per-output ephemeral NIKE gives the scanner one action/validation workload per candidate announcement unless an additional filtering protocol changes that workload. Tags derived from the same shared secret can save later checks but still require deriving that secret first. Pairwise cached secrets amortize actions but change correlation, compromise and rotation properties.
- Static-static NIKE derives the same pairwise secret until keys/context change. It does not itself rotate keys, preserve a root identity, create forward secrecy or hide traffic metadata. Separate root authority, view/scan key and spending key; define authenticated key epochs and retention. Retaining old view keys supports historical discovery but retains exposure if those keys are compromised; deleting them changes recovery and late-payment behavior.

This is why **BBG's explicit CSIDH-512 selection** (`privacy.md:212`) must be reconciled with the selected threat model and scan budget. It is not enough to preserve a generic CSIDH name while changing modulus size. The requested genies design material is present under `strata/genies/specs/`; its 512-bit profile and dCTIDH descriptions repeat the same unresolved model/naming choices.

The jali balance profile is separate from seal: different polynomial degrees can be intentional. However, BBG calls it degree512, approximately60-bit q and 4KB/commitment (`privacy.md:107–111`), while naming Goldilocks and showing a pair of ring elements. Two full 512×u64 polynomials require8KB absent a specified compression/derivation rule. Pin modulus, plaintext/error encoding and stored components before using the 4KB number in capacity planning; do not borrow ML-KEM security/size labels for this different protocol.

## Decision sequence

1. Preserve proof authority, seal and NIKE as distinct roles; this review supplies no reason to replace them with an unsolicited conventional signature.
2. Resolve **seal standard-vs-Goldilocks profile**, with distinct names and parameter/security evidence if both are retained.
3. Resolve **stealth's exact prime/keyspace/attack-resource model and implementation family**; benchmark action, validation, complete ephemeral keygen and candidate scanning separately on target hardware.
4. Finalize the announcement/view/spend/epoch contract and the separate jali commitment encoding. Then the project's intended design can be compared meaningfully on bandwidth, scan load, proving cost and recoverability.

No general "CSIDH is broken" inference is made from SIDH's different failure. The primary sources above instead establish parameter-model debate, concrete cost differences and continued physical-security work. This is a bounded review, not an exhaustive certificate that no later attack exists.

The [retained benchmark extract](mlkem-bench-selected.json) contains the dated M1 upstream measurements. Immutable source URLs above pin the dCTIDH encoding. Additional download metadata was kept in `/tmp/mudra-revision-20260912`. No local CTIDH or ML-KEM benchmark was run for this revision.
