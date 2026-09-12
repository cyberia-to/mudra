# Mudra: the Cyber choice

**2026-09-12 · Architecture recommendation.** The current design and proposed
extension are distinguished below; concrete cryptographic profiles still need
qualification.

**Cyber keeps proof-based authority, confidential state and CSIDH stealth.
The next step is private, verifiable wallet recovery through the node.**
This combines Neptune's useful idea of proving private ownership with Cyber's
existing programmable execution and authenticated graph.

## How the chosen scheme works

1. **Authorize with Hemera + Zheng.** Hemera commits to the owner's secret;
   a Zheng proof shows that the owner satisfies the spending/policy rules for
   this exact action. The verifier checks the proof without learning the secret.
   The same proof can cover authorization and the state transition.
2. **Receive privately with Mudra.** CSIDH/stealth lets sender and recipient
   derive a shared secret for an encrypted payment, even while the recipient is
   offline. Per-payment stealth uses a public ephemeral announcement. The spend
   secret stays independent: the sender also knows the shared secret.
   **seal** supplies a separate lattice KEM for recipient encryption where that
   interface is needed.
3. **Keep confidential state in BBG.** Commitments hide owners and values;
   public nullifiers prevent reuse of spent notes. Authenticated state gives
   the node a root against which to prove balances and transitions.
4. **Recover through private, verifiable queries.** Today the BBG design asks
   recipients to scan announcements. The proposed extension lets a node find
   a wallet's incoming notes privately, then prove that it processed the full
   requested history. The client verifies the result and current spend state.

**Why this fits Cyber:** programmable authorization, private balances and
provable queries share the existing execution/state stack. This architecture
choice does not require adopting a new detached signature algorithm.

## Why the other designs are not the whole answer

- **Quantus:** attractive mobile recovery and proof aggregation, but public
  amounts and endpoint-visible wallet queries fall short of Cyber's privacy
  goal. Keep its interoperability and product lessons.
- **Neptune Cash:** the closest model for private proof-based spending.
  Its ordinary wallet still downloads global stripped history; Cyber already
  has BBG/Inf for the state and query layer.
- **Neptune Privacy:** compact CTIDH receiving addresses are directly relevant.
  Its indexed SDK recovery still exposes recipient interests, and the
  CSIDH-512 parameter choice needs a defensible quantum attack model.

## Visual assessment

**My engineering assessment for Cyber's requirements.** These grades compare
mechanisms and tradeoffs; they are not benchmark scores or security certificates.

**🟢 3 — strong fit · 🟡 2 — material tradeoff · 🔴 1 — weak fit · ? — unestablished**

**XNT** = Neptune Privacy; **NPT** = Neptune Cash.
**Cyber¹** = current specification; **Cyber+²** = proposed architecture target.
The two Cyber columns assess designs, not delivered guarantees.

| Criterion | Quantus | XNT | NPT | Cyber¹ | Cyber+² |
|---|:---:|:---:|:---:|:---:|:---:|
| Hidden amounts | 🔴 1 | 🟢 3 | 🟢 3 | 🟢 3 | 🟢 3 |
| Recipient-selection privacy | 🔴 1 | 🔴 1 | 🟡 2 | 🟡 2 | 🟢 3 |
| Low client recovery work | 🟢 3 | 🟢 3 | 🔴 1 | 🔴 1 | 🟡 2 |
| Verified history coverage | 🔴 1 | 🔴 1 | 🔴 1 | 🟡 2 | 🟢 3 |
| Programmable authority | 🟡 2 | 🟢 3 | 🟢 3 | 🟢 3 | 🟢 3 |
| Low crypto review burden | 🟡 2 | 🟡 2 | 🟡 2 | 🔴 1 | 🔴 1 |
| Measured private-send speed | ? | ? | ? | ? | ? |

The grades have specific reasons:

- **Recovery:** Quantus and XNT index a wallet's history on the server.
  NPT and current BBG scan global history on the client. OMR reduces downloaded
  payloads and client decryption, but retains a compact history-sized digest
  and substantial server work; hence Cyber+ gets **2**, not a free **3**.
- **Query trust:** NPT's common-history scan hides discovery interests, but
  witness requests leak information. Current Cyber has the Inf proof contract;
  the private-recovery composition is the extension. Coverage proves that the
  declared history was processed; detection errors and overflow need their own
  bounds. IP addresses, session timing and queried ranges remain observable.
- **Complexity:** both Neptunes combine proofs with a custom KEM; XNT also
  offers CTIDH. Cyber adds its own Hemera and RLWE/polynomial-commitment state
  composition; the extension adds OMR's homomorphic encryption/PIR. The lower
  grade reflects that additional qualification burden, not predicted bug rates.
- **Speed:** no matched full private-transfer benchmark supports a ranking.

[Detailed mechanisms, versions and evidence](signature-optimality/comparison-details.md)
remain available separately. No total score is used: strong recovery cannot
compensate for disclosing amounts when amount privacy is a requirement.

## UnifOMR: what the new paper changes

**OMR means Oblivious Message Retrieval:** retrieving your messages without the
server learning which messages belong to you. For a wallet, the messages are
encrypted payment notifications.

With **UnifOMR**, the sender adds an encrypted detection clue. A server processes
those clues using recipient-supplied encrypted detection material. The recipient
decodes a compact response and privately retrieves candidate payloads using
**PIR — Private Information Retrieval**. This moves expensive discovery work
off the wallet while protecting its selection. It retains server-side history
processing and adds clue/key storage. [Paper](https://eprint.iacr.org/2026/910).

### Cyber integration follow-up, 2026-09-12

**OMR hides the selection; Inf/Zheng authenticate the computation.** BBG commits
the complete notification board. Inf defines the root/range and complete-query
semantics. Zheng proves encrypted detection and the PIR answer against that
board, without the server needing the recipient's decryption key. The wallet
then verifies note inclusion and spend state against canonical roots.

This is the selected direction for a normal node capability or adjacent worker.
Detection error, overflow, transcript privacy and data availability remain part
of the contract. [Cybergraph design](../../cybergraph/docs/private-retrieval.md),
[Inf coverage](../../inf/specs/proof.md#complete-input-coverage),
[node roadmap C2.1](../../cyber/roadmap/c-network.md#c21-verifiable-private-retrieval).

## What still needs a decision

**Choose the exact CSIDH profile and scan budget; resolve standard ML-KEM versus
a separately specified Goldilocks seal; finalize independent view/spend keys
and action/policy binding.** These qualify the chosen architecture.
[Parameter evidence](signature-optimality/mudra-design.md).

## Concrete costs

The retained [size and cost table](signature-optimality/comparison-details.md#concrete-costs)
separates signatures, receiving material, encrypted notifications and proofs.
The [UnifOMR measurements](signature-optimality/unifomr-evidence.md#costs-parameters-and-what-was-actually-measured)
also account for server work, keys and per-message clues.
