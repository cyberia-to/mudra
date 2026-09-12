# Mudra comparison: mechanisms, costs and evidence

2026-09-12. Supporting evidence for the [Cyber choice and visual assessment](../signature-optimality.md).
The main report explains the architectural recommendation; this page preserves
the detailed comparison and measurements.

## Five-system comparison

Sources: [Quantus](../quantus-identity-privacy.md),
[Neptune Privacy / XNT](neptune-privacy.md),
[Neptune Cash / NPT](neptune-evidence.md),
[Mudra design and parameter evidence](mudra-design.md).
Snapshot: Quantus chain 1.0.1/circuits 4.3.0; XNT 0.2.7; Cash core 0.17.0,
desktop 4.2.1, which packages core 0.15.0. XNT SDK behavior is identified
separately from its mobile app.

| Decision axis | Quantus | Neptune Privacy (XNT) | Neptune Cash (NPT) | Current Cyber/Mudra design | Proposed Cyber/Mudra next |
|---|---|---|---|---|---|
| **Spend authority** | Transparent: ML-DSA-65/87. Wormhole: deposit-secret, inclusion and nullifier proof | Tip5 lock predicate inside Triton STARK | Tip5 lock predicate inside Triton STARK | Hemera ownership/policy predicate inside Zheng proof | Same authority; explicit action, network, policy and replay binding |
| **Receiving a secret** | Wormhole uses deposits to derived public accounts; no confidential-note KEM in this path | Default Generation: lattice KEM + AES-GCM. Optional CTIDH-512 short addresses + AES-GCM | Default Generation: same lattice KEM + AES-GCM; optional EC-hybrid | seal: lattice KEM; stealth: CSIDH NIKE; BBG uses ephemeral stealth discovery | Qualified KEM/NIKE profiles; separate viewing and spending keys |
| **Ledger confidentiality** | Deposit→exit link hidden; deposit/exit addresses and amounts public | Ordinary UTXO amounts and lock policies hidden | Ordinary UTXO amounts and lock policies hidden | BBG hides owners/values; nullifiers and aggregates public | Preserve confidential state while hiding retrieval interests |
| **Observable metadata** | Amounts, timing, endpoints, anchors, nullifiers | Fees, timing, structure, receiver tags; reused tags link receipts | Fees, timing, structure, receiver tags; reused tags link receipts | Public roots/nullifiers/aggregates; announcement leakage needs an explicit budget | Same ledger observables; padded private-query transcript; IP/timing remain separate |
| **Cold restore** | Mobile index lookup over derived addresses and their history; gap 20 | SDK indexed receiver lookup over height ranges; mobile recovery policy unverified | Desktop downloads stripped history and scans locally; 25-key look-ahead | BBG specifies recipient scanning of global announcements | OMR retrieves candidate notes; Inf verifies complete query execution |
| **Wallet interface** | Mobile app | iOS/Android apps; public SDK; app/source correspondence unverified | Desktop light wallet, separate core/CLI | Mudra library and node/client architecture | cyb client consumes standard node retrieval capability |
| **What the query server learns** | Full queried address groups and exact nullifier hashes | SDK sends recognizable hashes of public receiver tags, exact commitments and spent hashes | Ordinary history download hides which notes matched; witness requests expose candidate ranges | Local scanning needs no recipient-specific discovery query; remote disclosure contract unfinished | Cryptographic query privacy, subject to the composed OMR/PIR protocol |
| **Warm state / witnesses** | Cached history plus repeated address/history reconstruction; spend witnesses on demand | SDK can request new ranges; core revisits monitored witnesses each tip | Desktop processes new blocks/held coins; latest archival core derives witnesses on demand | BBG authenticates live commitments/nullifiers; wallet cursor and witness policy need definition | Verified epoch cursor, authenticated spend state and reorg rollback |
| **Light-client completeness** | Indexed balance trusts omissions/spent answers | Indexed collection has no complete-result proof | Common-history download alone does not establish canonical complete history | Inf supplies the intended provable-query contract | Root/range-bound detection **and** PIR execution proof |
| **Cryptographic verification surface** | ML-DSA plus a separate Poseidon2/Plonky2 Wormhole circuit/recursion stack | Tip5/Triton, custom lattice KEM; optional CTIDH integration | Tip5/Triton and custom lattice KEM | Hemera/Zheng, CSIDH and lattice commitment/encryption profiles | Existing stack plus OMR's RLWE/BFV/PIR composition |
| **Quantum qualification** | ML-DSA categories 3/5; Wormhole's configured 100-bit floor is a separate claim | Triton setting 160; 192-bit seed; CTIDH-512 needs its own attack model | Same seed family; EC-hybrid receipt privacy fails under quantum ECDH recovery | Hemera/proof and CSIDH/lattice parameters each require qualification | Explicit resource model and parameters for every layer; no inherited whole-wallet “PQ bits” |

**Recovery always needs historical data as well as keys.** Both Neptunes'
on-chain notifications support rediscovery; off-chain receipts require retained
payloads. A derivation gap can miss distant indices. Server-side indexing moves
global-history work off the client; it does not eliminate that work.

The proof settings above are not quantified quantum-security levels. Both
Neptunes' approximately 192-bit seeds permit generic quantum search around
2^96 oracle queries given a checkable derived address; this is a theoretical
query count, not a practical attack or timing measurement.

## Concrete costs

Objects below have different jobs. A receiving key, signature field and complete
transaction proof must be budgeted separately.

| Quantity | Quantus | Neptune Privacy | Neptune Cash | Current Mudra | Proposed Mudra |
|---|---|---|---|---|---|
| Transparent signature field | **5,262 / 7,220 B**, including public key and enum | Native proof authorization | Native proof authorization | Native proof authorization | Preserve native proof authorization |
| Receiving material | Public account destination | Generation **2,168 B**; optional short payload **64 B**, subaddress **72 B** | Generation **2,168 B** | CSIDH-512 curve **64 B**; full address encoding separate | Parameter-dependent; authors' dCTIDH-2048 key is **264 B** |
| Secret-delivery overhead | No encrypted-note capsule in reviewed Wormhole path | Generation KEM **2,560 B**; short-mode ephemeral key **64 B**, before field encoding | Generation KEM **2,560 B** | BBG's per-output stealth scheme needs an ephemeral public announcement; seal size depends on chosen profile | Stealth announcement plus OMR clue if enabled |

XNT layouts were checked with pinned dependency serialization. Its short mode
saves normally **2,480 B per equal-payload announcement** after field encoding,
including the length/header differences. Generation text addresses are **3,482
characters**, XNT short addresses **116**. This is a real bandwidth benefit;
stronger CSIDH parameters change that tradeoff.

Published CTIDH-512 action+validation costs **129.64 million cycles** on a
3 GHz Xeon E3-1220v5 (~43 ms). dCTIDH-2048 action costs **1,409 million cycles**
on an i7-6700. These are distinct primitive/profile measurements, not mobile
send times. Per-output NIKE scanning pays an action per candidate unless a
separate discovery/filtering mechanism reduces candidates.
[Sources and operation boundaries](mudra-design.md#stealth-distinguish-variants-parameters-and-measurements).

**No comparable full private-transfer latency winner is established.** Quantus's
reported ~266 KB is a **371-capacity public aggregate**, not one wallet proof.
XNT's advertised one-second L2 is not a measured released-L1 transfer.
The decisive benchmark should report one equal-input/output private transfer,
proof bytes/time/RAM, and seed restore plus incremental update at fixed history
and wallet sizes on the same target hardware.

## Decisions to finalize

1. **Preserve proof authorization.** Bind ownership to the complete action and
   stable subject's versioned policy. A standalone signature becomes a separate
   choice only if we require independently transferable signed messages outside
   that proof contract. Quantus remains useful as a cross-verification source.
2. **Choose an exact stealth profile and scan budget.** CSIDH-512's 64-byte key
   does not establish today's target quantum margin. The high-security study
   uses **2048/4096-bit primes for level 1 under different attack-resource
   models**. A seed length or field width is not a security estimate.
   [Parameter analysis](https://thomwiggers.nl/publications/secsidh/secsidh.pdf).
3. **Resolve seal's standard-versus-native construction.** The spec combines
   Goldilocks/degree 64/rank 4 with standard ML-KEM sizes. FIPS 203 uses
   modulus 3329/degree 256. Choose standard interoperability or qualify a
   separately named native construction; standard sizes/security do not transfer.
   [Exact discrepancy](mudra-design.md#seal-a-consequential-parameter-ambiguity).
4. **Separate discovery from exclusive spend authority.** A NIKE shared secret
   is also known to the sender. Specify ephemeral public announcements,
   authenticated key epochs and independent viewing/spending relations. XNT's
   short-address mapping provides a concrete integration counterexample:
   its lock witness is derived from public address bytes. An offline VM check
   confirms that predicate issue; it does not break CSIDH.
   [Source and bounded reproduction](neptune-privacy.md#ctidh-genuine-bandwidth-benefit-separate-parameter-and-integration-questions).

## UnifOMR: what the new paper changes

The fifth column combines private discovery with our existing proof stack.
[UnifOMR Param2](https://eprint.iacr.org/archive/2026/910/1778291181.pdf#page=36)
reports **41 s server / 54 ms client**, **5,955 KB digest**, **48 MB keys** and
**2,565 B clue per message** for one recipient, 524,288 messages, 50 matches and
612-byte payloads. These are author measurements before Cyber proof overhead.
Server detection and the compact client digest still scale with global message
count. See [measurement and error qualifications](unifomr-evidence.md).

### Cyber integration follow-up, 2026-09-12

The selected home is an ordinary node capability or adjacent worker:
**BBG commits the full board; Inf defines complete query semantics; Mudra
supplies private operations; Zheng proves encrypted detection and PIR response
computation.** This preserves recipient secrecy while authenticating execution
over the requested root/range. Detection errors, overflow and transcript privacy
remain explicit protocol requirements. The design trace is
[Cybergraph private retrieval](../../../cybergraph/docs/private-retrieval.md), with
[Inf's coverage contract](../../../inf/specs/proof.md#complete-input-coverage) and
[node roadmap C2.1](../../../cyber/roadmap/c-network.md#c21-verifiable-private-retrieval).
