use super::*;

fn seed(i: u8) -> [u8; 32] {
    [i; 32]
}

fn digest(v: u64) -> Digest {
    [v, v + 1, v + 2, v + 3]
}

const CONTENT: Digest = [11, 12, 13, 14];
const SCOPE: Digest = [21, 22, 23, 24];

/// A depth-3 set: authors 0..5 at indices 0..5.
fn set() -> (AuthoritySet, Vec<Secret>) {
    let secrets: Vec<Secret> = (0..5).map(|i| Secret::from_seed(&seed(i))).collect();
    let leaves: Vec<Digest> = secrets.iter().map(Secret::commitment).collect();
    (AuthoritySet::new(3, &leaves).unwrap(), secrets)
}

#[test]
fn native_tree_and_paths_agree() {
    let (set, secrets) = set();
    for (i, s) in secrets.iter().enumerate() {
        assert_eq!(root_of(&s.commitment(), &set.path(i).unwrap()), set.root());
    }
    // absent leaves are EMPTY: the same set padded explicitly has the same root
    let mut padded: Vec<Digest> = secrets.iter().map(Secret::commitment).collect();
    padded.resize(8, EMPTY);
    assert_eq!(AuthoritySet::new(3, &padded).unwrap().root(), set.root());
    assert_eq!(set.path(5).unwrap_err(), Error::Index);
    assert_eq!(AuthoritySet::new(0, &[]).unwrap_err(), Error::Depth);
    assert_eq!(AuthoritySet::new(1, &[EMPTY; 3]).unwrap_err(), Error::Index);
}

#[test]
fn every_member_proves_and_the_cost_is_the_same_at_every_position() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    for (i, s) in secrets.iter().enumerate() {
        let path = set.path(i).unwrap();
        // prove() checks the execution's statement — outputs AND cycles —
        // equals the one the verifier derives with the key's constant cost
        let (statement, proof) = prove(&key, s, &path, &set.root(), &CONTENT, &SCOPE).unwrap();
        assert_eq!(statement.nullifier, s.nullifier(&SCOPE));
        verify(&key, &statement, &proof).unwrap();
    }
}

#[test]
fn a_secret_outside_the_set_is_refused_before_proving() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    let outsider = Secret::from_seed(&seed(99));
    let path = set.path(0).unwrap();
    assert_eq!(prove(&key, &outsider, &path, &set.root(), &CONTENT, &SCOPE).unwrap_err(), Error::NotAMember);
    // a member with another member's path, a moved sibling, a flipped direction
    assert_eq!(prove(&key, &secrets[1], &path, &set.root(), &CONTENT, &SCOPE).unwrap_err(), Error::NotAMember);
    let mut bad = set.path(1).unwrap();
    bad.siblings[2][0] ^= 1;
    assert_eq!(prove(&key, &secrets[1], &bad, &set.root(), &CONTENT, &SCOPE).unwrap_err(), Error::NotAMember);
    let mut bad = set.path(1).unwrap();
    bad.directions[0] ^= 1;
    assert_eq!(prove(&key, &secrets[1], &bad, &set.root(), &CONTENT, &SCOPE).unwrap_err(), Error::NotAMember);
    let mut bad = set.path(1).unwrap();
    bad.directions[0] = 2;
    assert_eq!(prove(&key, &secrets[1], &bad, &set.root(), &CONTENT, &SCOPE).unwrap_err(), Error::NonCanonical);
}

/// A prover that skips the native membership check: zheng proves the
/// execution, whose first output is then nonzero, so no statement the
/// verifier derives (it always expects 0) accepts the proof.
#[test]
fn a_non_member_proof_made_around_the_native_check_does_not_verify() {
    let (set, _) = set();
    let key = Key::new(3).unwrap();
    let outsider = Secret::from_seed(&seed(99));
    let path = set.path(2).unwrap();
    let statement = Statement {
        depth: 3,
        root: set.root(),
        content: CONTENT,
        scope: SCOPE,
        nullifier: outsider.nullifier(&SCOPE),
    };
    let (st, proof) =
        veil::prove(&key.program, &statement.inputs(), &secret_stream(&outsider, &path), BUDGET).unwrap();
    assert_ne!(st.execution.public_output[0], 0, "the root check fails inside the program");
    assert!(verify(&key, &statement, &AuthorshipProof(proof)).is_err());
}

#[test]
fn a_proof_answers_only_its_own_statement() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    let (statement, proof) = prove(&key, &secrets[3], &set.path(3).unwrap(), &set.root(), &CONTENT, &SCOPE).unwrap();
    let other = |f: &dyn Fn(&mut Statement)| {
        let mut s = statement.clone();
        f(&mut s);
        verify(&key, &s, &proof).is_err()
    };
    assert!(other(&|s| s.root = digest(1)), "other set root");
    assert!(other(&|s| s.content = digest(2)), "other content: the proof is bound to the signal");
    assert!(other(&|s| s.scope = digest(3)), "other scope");
    assert!(other(&|s| s.nullifier = digest(4)), "other nullifier");
    assert!(other(&|s| s.nullifier = secrets[4].nullifier(&SCOPE)), "another member's nullifier");
    assert_eq!(verify(&key, &Statement { depth: 4, ..statement.clone() }, &proof), Err(Error::Depth));
    assert!(verify(&Key::new(4).unwrap(), &Statement { depth: 4, ..statement.clone() }, &proof).is_err());
    let mut bad = statement.clone();
    bad.content[0] = nebu::field::P;
    assert_eq!(verify(&key, &bad, &proof), Err(Error::NonCanonical));
}

#[test]
fn tampered_proof_bytes_are_rejected() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    let (statement, proof) = prove(&key, &secrets[0], &set.path(0).unwrap(), &set.root(), &CONTENT, &SCOPE).unwrap();
    let bytes = proof.as_bytes().to_vec();
    let n = bytes.len();
    // every byte of the header, then a spread of positions across the body
    let positions: Vec<usize> = (0..64).chain((64..n).step_by(n / 97 + 1)).chain([n - 1]).collect();
    for i in positions {
        let mut b = bytes.clone();
        b[i] ^= 0x01;
        let ok = AuthorshipProof::from_bytes(&b).is_ok_and(|p| verify(&key, &statement, &p).is_ok());
        assert!(!ok, "flip at byte {i} of {n} accepted");
    }
    assert!(AuthorshipProof::from_bytes(&bytes[..n - 1]).is_ok_and(|p| verify(&key, &statement, &p).is_err()));
}

// ── leakage ─────────────────────────────────────────────────────────────

/// The verifier's whole view is the P1 statement: the zheng statement it
/// derives holds the program (fixed by the depth), R‖C‖S as inputs,
/// 0‖N‖C as outputs, the depth's constant cost and the budget — no limb
/// of the secret, the commitment or the path.
#[test]
fn the_statement_carries_only_public_values() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    // index 2: every sibling is a populated node (index 4's level-0 sibling
    // is the empty leaf [0; 4], equal to the public flag by value)
    let path = set.path(2).unwrap();
    let (statement, _) = prove(&key, &secrets[2], &path, &set.root(), &CONTENT, &SCOPE).unwrap();
    let z = key.zheng_statement(&statement).execution;
    assert_eq!(z.public_input, [set.root(), CONTENT, SCOPE].concat());
    assert_eq!(z.public_output, [vec![0], statement.nullifier.to_vec(), CONTENT.to_vec()].concat());
    assert_eq!((z.cycles, z.budget), (key.cycles(), BUDGET));
    let hidden: Vec<u64> = secrets[2]
        .limbs()
        .iter()
        .chain(&secrets[2].commitment())
        .chain(path.siblings.iter().flatten())
        .copied()
        .collect();
    let public: Vec<u64> = z.public_input.iter().chain(&z.public_output).copied().collect();
    assert!(hidden.iter().all(|h| !public.contains(h)));
}

/// No 8-byte window of the proof, at any alignment, equals a limb of the
/// secret, of the author's commitment or of a path sibling (little- or
/// big-endian).
#[test]
fn no_secret_commitment_or_path_limb_appears_in_the_proof() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    for i in [0usize, 3] {
        let path = set.path(i).unwrap();
        let (_, proof) = prove(&key, &secrets[i], &path, &set.root(), &CONTENT, &SCOPE).unwrap();
        let hidden: Vec<u64> = secrets[i]
            .limbs()
            .iter()
            .chain(&secrets[i].commitment())
            .chain(path.siblings.iter().flatten())
            .copied()
            .collect();
        for w in proof.as_bytes().windows(8) {
            let w: [u8; 8] = w.try_into().unwrap();
            let (le, be) = (u64::from_le_bytes(w), u64::from_be_bytes(w));
            assert!(!hidden.contains(&le) && !hidden.contains(&be), "hidden limb on the wire (author {i})");
        }
    }
}

/// Two witnesses of ONE statement — the same author's commitment at two
/// positions of the set, so different siblings and directions — give
/// proofs that a test cannot tell apart:
///
/// - lengths: a veil proof's length varies with the proving randomness
///   (163–166 KB here at depth 3, within one position too); the two
///   positions' lengths have the same mean (Welch |t| < 4);
/// - content: the top four bits of every aligned u64 word, pooled over all
///   proofs of a position, are identically distributed between the two
///   positions (chi-square two-sample, 15 degrees of freedom, reject above
///   44.3 ≈ p 1e-4).
///
/// zheng's own test does this per proof component on fixed seeds
/// (`execution/veil/zk_tests.rs`); its parser and seeded coins are
/// crate-private, so this reads the proof as words, with OS randomness.
#[test]
fn proofs_from_two_positions_of_one_statement_are_indistinguishable() {
    const PROOFS: usize = 24;
    const CRITICAL: f64 = 44.3;
    let s = Secret::from_seed(&seed(7));
    let mut leaves: Vec<Digest> = (0..8).map(|i| Secret::from_seed(&seed(40 + i)).commitment()).collect();
    leaves[1] = s.commitment();
    leaves[6] = s.commitment();
    let set = AuthoritySet::new(3, &leaves).unwrap();
    let key = Key::new(3).unwrap();
    let proofs = |index: usize| -> Vec<Vec<u8>> {
        (0..PROOFS)
            .map(|_| {
                let (st, p) = prove(&key, &s, &set.path(index).unwrap(), &set.root(), &CONTENT, &SCOPE).unwrap();
                verify(&key, &st, &p).unwrap();
                p.as_bytes().to_vec()
            })
            .collect()
    };
    let (a, b) = (proofs(1), proofs(6));
    let stats = |ps: &[Vec<u8>]| {
        let n = ps.len() as f64;
        let mean = ps.iter().map(|p| p.len() as f64).sum::<f64>() / n;
        let var = ps.iter().map(|p| (p.len() as f64 - mean).powi(2)).sum::<f64>() / (n - 1.0);
        (mean, var, n)
    };
    let ((ma, va, na), (mb, vb, nb)) = (stats(&a), stats(&b));
    let t = (ma - mb) / (va / na + vb / nb).sqrt().max(1e-9);
    let hist = |ps: &[Vec<u8>]| {
        let mut h = [0f64; 16];
        for p in ps {
            for w in p.chunks_exact(8) {
                h[(u64::from_le_bytes(w.try_into().unwrap()) >> 60) as usize] += 1.0;
            }
        }
        h
    };
    let (ha, hb) = (hist(&a), hist(&b));
    let (sa, sb): (f64, f64) = (ha.iter().sum(), hb.iter().sum());
    let chi: f64 = (0..16)
        .filter(|&i| ha[i] + hb[i] > 0.0)
        .map(|i| {
            let tot = ha[i] + hb[i];
            let (ea, eb) = (tot * sa / (sa + sb), tot * sb / (sa + sb));
            (ha[i] - ea).powi(2) / ea + (hb[i] - eb).powi(2) / eb
        })
        .sum();
    println!("length mean {ma:.0} vs {mb:.0} (Welch t {t:.2}); {sa} vs {sb} words, chi² two-sample {chi:.1}");
    assert!(t.abs() < 4.0, "proof length depends on the position: t = {t:.2}");
    assert!(chi < CRITICAL, "proof words depend on the position: chi² {chi:.1}");
}

#[test]
fn proofs_are_randomized() {
    let (set, secrets) = set();
    let key = Key::new(3).unwrap();
    let path = set.path(2).unwrap();
    let (_, p1) = prove(&key, &secrets[2], &path, &set.root(), &CONTENT, &SCOPE).unwrap();
    let (_, p2) = prove(&key, &secrets[2], &path, &set.root(), &CONTENT, &SCOPE).unwrap();
    assert_ne!(p1, p2, "a fixed proof would be a linkable tag");
}

#[test]
fn secrets_are_redacted_in_debug_output() {
    let s = Secret::from_seed(&seed(5));
    let shown = format!("{s:?} {:?}", AuthoritySet::new(1, &[s.commitment()]).unwrap().path(0).unwrap());
    assert!(!shown.contains(&s.limbs()[0].to_string()));
    assert_eq!(Secret::from_limbs([nebu::field::P, 0, 0, 0]).unwrap_err(), Error::NonCanonical);
}

