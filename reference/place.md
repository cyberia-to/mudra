---
tags: mudra, crypto
crystal-type: entity
crystal-domain: crypto
---
# place — prove where you are

proves geographic position through round-trip time measurements and geometric consistency. four axioms, zero trusted institutions.

## axioms

1. **existence**: at least one observer asserts a position (bootstrap anchor)
2. **bounded signal speed**: signals travel at most at the speed of light in the declared medium
3. **spherical Earth**: the embedding is constrained to Earth's surface (known radius)
4. **one honest observer**: at least one node in the RTT mesh reports truthfully

## mechanism

### RTT measurement

```
interface:
  rtt_measure(peer: NeuronId) → Duration
  rtt_to_distance(rtt: Duration, medium: Medium) → Distance

medium types:
  Fiber:    c_eff ≈ 2/3 × c  (refractive index of glass)
  Radio:    c_eff ≈ c         (free space)
  Copper:   c_eff ≈ 2/3 × c  (signal propagation in wire)

distance upper bound:
  dist(A, B) ≤ RTT(A, B) × c_medium / 2
```

RTT provides a distance upper bound. lying by reporting shorter RTT claims closer proximity than reality — but cannot claim to be farther (signals cannot exceed light speed).

### distance matrix

```
interface:
  build_distance_matrix(measurements: [(NeuronId, NeuronId, Duration)]) → Matrix<Distance>
```

N neurons produce N×(N-1)/2 pairwise RTT measurements. each measurement committed simultaneously via Merkle tree (prevents selective presentation of favorable measurements).

### classical MDS (multidimensional scaling)

```
interface:
  mds_embed(distances: Matrix<Distance>, dimensions: u8) → Vec<(f64, f64, f64)>
```

given pairwise distance matrix, classical MDS recovers 3D coordinate embedding:

1. double-center the squared distance matrix
2. eigendecompose → top 3 eigenvalues/eigenvectors
3. coordinates = eigenvectors × sqrt(eigenvalues)

the embedding is unique up to rotation/reflection (resolved by the bootstrap anchor from axiom 1).

### self-calibration

Earth's circumference provides the scale factor. nodes on opposite sides of the planet have RTT bounded by:

```
max_rtt ≈ 2 × (circumference / 2) / c_fiber ≈ 200 ms
```

the MDS embedding self-calibrates to Earth's scale from canonical propagation speeds. no GPS, no external reference frame needed.

## verification

```
interface:
  location_prove(neuron: NeuronId, mesh: RTTMesh) → LocationProof
  location_verify(proof: LocationProof, mesh: RTTMesh) → Result<Position, LocationError>
```

a location proof consists of:
- N RTT measurements to distinct peers (committed via Merkle tree)
- VDF proof on each challenge-response (prevents pre-computation)
- medium declaration for each link
- MDS embedding coordinates
- geometric consistency score

verification checks:
1. RTT measurements are internally consistent (triangle inequality)
2. VDF proofs valid (timing cannot be gamed)
3. MDS embedding stress is below threshold (measurements consistent with 3D geometry)
4. declared medium matches observed RTT bounds
5. Merkle commitment covers all measurements (no selective presentation)

## sybil resistance

physical: faking RTT consistency across a dense global mesh requires physical presence at the claimed locations. an attacker in one location cannot simultaneously produce consistent RTT measurements to peers distributed worldwide.

economic: relay fees proportional to 1/latency make geographic honesty a dominant strategy equilibrium. lying about location means suboptimal routing fees.

## properties

| property | value |
|----------|-------|
| precision | city-level (~50-100 km) from RTT alone |
| trust | one honest observer (axiom 4) |
| setup | zero trusted institutions |
| proof size | O(N) RTT measurements + VDF proofs |
| verification | O(N²) for MDS, O(N) for consistency checks |

## dependencies

- hemera: Merkle commitment of RTT measurements
- mudra::delay: VDF on challenge-response (prevents pre-computation)
- nebu: field arithmetic for MDS computation (eigendecomposition over Goldilocks)

## usage in cyber

**relay pricing**: relay fees proportional to inverse latency. geographic honesty = maximum relay revenue.

**consensus geography**: nodes declare location. BBG_poly(locations) records proven positions. sharding by geographic proximity becomes verifiable.

**signal ordering**: VDF + location provides physical spacetime ordering — not just time (VDF) but also space (location).

## open questions

- minimum mesh density for reliable MDS embedding
- precision vs number of RTT measurements tradeoff
- indoor/outdoor medium detection
- satellite link handling (high latency, variable path)
- integration with BBG_poly(locations) evaluation dimension
