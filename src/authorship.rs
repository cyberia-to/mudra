// ---
// tags: mudra, rust, privacy, zheng
// crystal-type: source
// crystal-domain: comp
// ---
//! P1 private authorship: the edge is public, the author is private.
//!
//! Normative: `specs/authorship.md`. A signal's author proves, in zheng's
//! zero-knowledge profile (veil, envelope profile 2), that it holds the
//! secret behind one authority commitment in a committed authority set,
//! bound to the signal's content commitment, and publishes a scoped
//! nullifier — without revealing which commitment, the secret, or the
//! Merkle path.
//!
//! ```text
//! public  d, R (set root), C (content commitment), S (nullifier scope),
//!         N (nullifier)
//! secret  s (4 limbs), σ_0..σ_{d−1} (siblings), b_0..b_{d−1} (directions)
//! a       = H([D_AUTH s])
//! cur_0   = a,  cur_{i+1} = H(b_i = 0 ? [cur_i σ_i] : [σ_i cur_i])
//! output  [eq(cur_d, R) [N C]],  N = H([D_NULL [s S]])
//! accept  iff the output is [0 [N C]] and the veil proof verifies
//! ```
//!
//! `H` is nox's hash pattern (15) as zheng compiles it: the structural
//! digest of the noun (an atom is hemera's tree leaf of its 8 LE bytes, a
//! pair the tree node of its halves' digests), then one plain hemera
//! `StepSponge` permutation with the digest in the rate; the first four
//! lanes are the result. A digest travels as the noun `[[d0 d1] [d2 d3]]`.
//! [`noun_digest`] is the native twin the set holder uses to build the
//! tree.

use nebu::Goldilocks;
use nox::data::{hash_atom, hash_pair};
use zheng::execution::private::PrivateStatement;
use zheng::execution::veil::{self, VeilProof};
use zheng::execution::{ExecutionNoun as N, ExecutionStatement, VerifyingKey};

/// A digest as four canonical Goldilocks limbs (the 32-byte particle, LE).
pub type Digest = [u64; 4];

/// Domain atom of an authority commitment, ASCII "P1AUTH".
pub const D_AUTH: u64 = 0x5031_4155_5448;
/// Domain atom of a nullifier, ASCII "P1NULL".
pub const D_NULL: u64 = 0x5031_4e55_4c4c;
/// The empty leaf of an authority set.
pub const EMPTY: Digest = [0; 4];
/// Smallest and largest set depth the statement admits.
pub const MIN_DEPTH: usize = 1;
pub const MAX_DEPTH: usize = 32;
/// The execution budget every P1 statement carries.
pub const BUDGET: u64 = 1 << 24;

/// Errors of P1 proving and verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Depth outside `MIN_DEPTH..=MAX_DEPTH`.
    Depth,
    /// A limb at or above the Goldilocks modulus.
    NonCanonical,
    /// Index or leaf count outside the set.
    Index,
    /// The secret does not open the path to the root (prover side).
    NotAMember,
    /// zheng refused to prove or the proof did not verify.
    Proof(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Depth => write!(f, "authority set depth out of range"),
            Error::NonCanonical => write!(f, "limb not a canonical Goldilocks element"),
            Error::Index => write!(f, "leaf index or count outside the set"),
            Error::NotAMember => write!(f, "secret does not open a path to the root"),
            Error::Proof(m) => write!(f, "P1 proof: {m}"),
        }
    }
}

impl std::error::Error for Error {}

fn canonical(d: &Digest) -> Result<(), Error> {
    if d.iter().all(|&v| v < nebu::field::P) { Ok(()) } else { Err(Error::NonCanonical) }
}

// ── native twin of nox's hash pattern ───────────────────────────────────

fn atom_digest(v: u64) -> nox::Digest {
    hash_atom(Goldilocks::new(v))
}

/// `H` of the digest noun `[[d0 d1] [d2 d3]]`.
fn digest_noun(d: &Digest) -> nox::Digest {
    hash_pair(
        &hash_pair(&atom_digest(d[0]), &atom_digest(d[1])),
        &hash_pair(&atom_digest(d[2]), &atom_digest(d[3])),
    )
}

/// Pattern 15's last step: one StepSponge permutation of the structural
/// digest (zheng `execution/hash.rs`, `Builder::hash`).
fn limbs(d: &nox::Digest) -> Digest {
    let rate = d.map(|v| hemera::field::Goldilocks::new(v.as_u64()));
    // a four-element rate is one absorb step: `last` is always present
    let state = hemera::StepSponge::absorb(&rate).last().unwrap_or([hemera::field::Goldilocks::new(0); 16]);
    core::array::from_fn(|i| state[i].as_canonical_u64())
}

/// `H([l r])` for two digest nouns — one tree node.
pub fn noun_digest(l: &Digest, r: &Digest) -> Digest {
    limbs(&hash_pair(&digest_noun(l), &digest_noun(r)))
}

/// An author's P1 secret: four Goldilocks limbs, never on the wire.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret([u64; 4]);

impl core::fmt::Debug for Secret {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Secret(…)")
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        for limb in self.0.iter_mut() {
            // SAFETY: a valid, aligned, exclusively borrowed u64.
            unsafe { core::ptr::write_volatile(limb, 0) };
        }
    }
}

impl Secret {
    /// Derive from 32 bytes of key material (hemera, domain-separated);
    /// each limb is the LE u64 of its 8 bytes reduced mod p.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let mut h = hemera::Hasher::new();
        h.update(b"mudra-p1-secret-v1");
        h.update(seed);
        let out = h.finalize();
        let b = out.as_bytes();
        Self(core::array::from_fn(|i| {
            let mut w = [0u8; 8];
            w.copy_from_slice(&b[8 * i..8 * i + 8]);
            Goldilocks::new(u64::from_le_bytes(w)).canonicalize().as_u64()
        }))
    }

    /// The limbs, for a caller that stores the secret itself.
    pub fn limbs(&self) -> &[u64; 4] {
        &self.0
    }

    /// From stored limbs; refuses non-canonical ones.
    pub fn from_limbs(limbs: [u64; 4]) -> Result<Self, Error> {
        canonical(&limbs)?;
        Ok(Self(limbs))
    }

    /// `a = H([D_AUTH s])`: what the authority set holds for this author.
    pub fn commitment(&self) -> Digest {
        limbs(&hash_pair(&atom_digest(D_AUTH), &digest_noun(&self.0)))
    }

    /// `N = H([D_NULL [s S]])`.
    pub fn nullifier(&self, scope: &Digest) -> Digest {
        limbs(&hash_pair(&atom_digest(D_NULL), &hash_pair(&digest_noun(&self.0), &digest_noun(scope))))
    }
}

// ── the authority set ───────────────────────────────────────────────────

/// A Merkle tree of `2^d` authority commitments (absent leaves = [`EMPTY`]).
#[derive(Clone, Debug)]
pub struct AuthoritySet {
    /// `levels[0]` = leaves, `levels[d]` = `[root]`.
    levels: Vec<Vec<Digest>>,
}

/// A leaf's authentication path, leaf level first.
#[derive(Clone, PartialEq, Eq)]
pub struct Path {
    pub siblings: Vec<Digest>,
    /// `b_i`: 0 when the running node is the left child at level `i`.
    pub directions: Vec<u64>,
}

impl core::fmt::Debug for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Path(depth {})", self.siblings.len())
    }
}

impl AuthoritySet {
    pub fn new(depth: usize, commitments: &[Digest]) -> Result<Self, Error> {
        if !(MIN_DEPTH..=MAX_DEPTH).contains(&depth) {
            return Err(Error::Depth);
        }
        if depth < usize::BITS as usize && commitments.len() > 1usize << depth {
            return Err(Error::Index);
        }
        commitments.iter().try_for_each(canonical)?;
        // levels hold only the populated prefix; an absent node of level i
        // is the root of an empty subtree of height i
        let mut empty = EMPTY;
        let mut level: Vec<Digest> = commitments.to_vec();
        let mut levels = Vec::with_capacity(depth + 1);
        for _ in 0..depth {
            let next = level
                .chunks(2)
                .map(|c| noun_digest(&c[0], c.get(1).unwrap_or(&empty)))
                .collect::<Vec<_>>();
            levels.push(core::mem::take(&mut level));
            level = if next.is_empty() { vec![] } else { next };
            empty = noun_digest(&empty, &empty);
        }
        if level.is_empty() {
            level.push(empty);
        }
        levels.push(level);
        Ok(Self { levels })
    }

    pub fn depth(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn root(&self) -> Digest {
        self.levels[self.depth()][0]
    }

    /// The path of leaf `index`.
    pub fn path(&self, index: usize) -> Result<Path, Error> {
        if index >= self.levels[0].len() {
            return Err(Error::Index);
        }
        let mut empty = EMPTY;
        let mut siblings = Vec::with_capacity(self.depth());
        let mut directions = Vec::with_capacity(self.depth());
        let mut i = index;
        for level in &self.levels[..self.depth()] {
            siblings.push(*level.get(i ^ 1).unwrap_or(&empty));
            directions.push((i & 1) as u64);
            i >>= 1;
            empty = noun_digest(&empty, &empty);
        }
        Ok(Path { siblings, directions })
    }
}

/// Recompute the root from a leaf and its path (native).
pub fn root_of(leaf: &Digest, path: &Path) -> Digest {
    path.siblings.iter().zip(&path.directions).fold(*leaf, |cur, (sib, &b)| {
        if b == 0 { noun_digest(&cur, sib) } else { noun_digest(sib, &cur) }
    })
}

// ── the program ─────────────────────────────────────────────────────────

fn at(v: u64) -> N {
    N::Atom(v)
}
fn pr(a: N, b: N) -> N {
    N::Pair(Box::new(a), Box::new(b))
}
fn op(tag: u64, arg: N) -> N {
    pr(at(tag), arg)
}
fn axis(n: u64) -> N {
    op(0, at(n))
}
fn quote(v: N) -> N {
    op(1, v)
}
fn cons(x: N, y: N) -> N {
    op(3, pr(x, y))
}
fn hash(x: N) -> N {
    op(15, x)
}
/// Run `x` on the subject, then `y` on its result.
fn compose(x: N, y: N) -> N {
    op(2, pr(x, quote(y)))
}
/// One secret field element (call pattern 16, tag 1 = secret witness,
/// check `0`: accepted as given — the relation checks it).
fn secret() -> N {
    op(16, pr(quote(at(1)), quote(at(0))))
}
fn digest_of(f: impl Fn(usize) -> N) -> N {
    cons(cons(f(0), f(1)), cons(f(2), f(3)))
}
/// Public inputs: R‖C‖S, four limbs each.
const INPUTS: usize = 12;

/// Axis of public input `k`. zheng builds the subject from the statement's
/// inputs last first, `[i11 [i10 … [i0 0]]]`, so input `k` is list
/// position `11 − k`.
fn input(k: usize) -> N {
    axis((1u64 << (INPUTS - 1 - k + 2)) - 2)
}

/// One Merkle level on `[cur [[σ b] rest]]` → `[H(b = 0 ? [cur σ] : [σ cur]) rest]`.
fn level() -> N {
    let left = cons(axis(2), axis(12));
    let right = cons(axis(12), axis(2));
    cons(hash(op(4, pr(axis(13), pr(left, right)))), axis(7))
}

/// The P1 program for depth `d` (`specs/authorship.md` § program).
pub fn program(depth: usize) -> Result<N, Error> {
    if !(MIN_DEPTH..=MAX_DEPTH).contains(&depth) {
        return Err(Error::Depth);
    }
    // A: inputs → [R [C [S s]]]; s is the first four secrets
    let a = cons(
        digest_of(input),
        cons(digest_of(|j| input(4 + j)), cons(digest_of(|j| input(8 + j)), digest_of(|_| secret()))),
    );
    // B: [R [C [S s]]] → [[a path] [R [C N]]]; R=2, C=6, S=14, s=15
    let leaf = hash(cons(quote(at(D_AUTH)), axis(15)));
    let path = (0..depth).fold(quote(at(0)), |rest, _| cons(cons(digest_of(|_| secret()), secret()), rest));
    let nullifier = hash(cons(quote(at(D_NULL)), cons(axis(15), axis(14))));
    let b = cons(cons(leaf, path), cons(axis(2), cons(axis(6), nullifier)));
    // C: fold the levels over [a path] → [[root 0] [R [C N]]]
    let levels = (1..depth).fold(level(), |acc, _| compose(acc, level()));
    let c = cons(compose(axis(2), levels), axis(3));
    // D: [[root 0] [R [C N]]] → [eq(root, R) [N C]]; root=4, R=6, C=14, N=15
    let d = cons(op(9, pr(axis(4), axis(6))), cons(axis(15), axis(14)));
    Ok(compose(a, compose(b, compose(c, d))))
}

/// The secret stream in the order the program consumes it (left before
/// right in every cons): `s`, then per level from the leaf up the
/// sibling's four limbs and the direction — level 0 is the path list's
/// head, the outermost cons.
fn secret_stream(secret: &Secret, path: &Path) -> Vec<u64> {
    let mut out = secret.0.to_vec();
    for (sib, &b) in path.siblings.iter().zip(&path.directions) {
        out.extend_from_slice(sib);
        out.push(b);
    }
    out
}

// ── statement, key, prove, verify ───────────────────────────────────────

/// The public statement of one P1 authorship proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub depth: usize,
    /// Authority set root.
    pub root: Digest,
    /// Content commitment of the signal (author fields excluded).
    pub content: Digest,
    /// Nullifier scope.
    pub scope: Digest,
    pub nullifier: Digest,
}

impl Statement {
    fn inputs(&self) -> Vec<u64> {
        [self.root, self.content, self.scope].concat()
    }
    fn outputs(&self) -> Vec<u64> {
        let mut out = vec![0];
        out.extend_from_slice(&self.nullifier);
        out.extend_from_slice(&self.content);
        out
    }
    fn check(&self) -> Result<(), Error> {
        if !(MIN_DEPTH..=MAX_DEPTH).contains(&self.depth) {
            return Err(Error::Depth);
        }
        [self.root, self.content, self.scope, self.nullifier].iter().try_for_each(canonical)
    }
}

/// What prover and verifier derive once per depth: the program, its
/// encoding, the verifying key (compiled relation) and the cost every P1
/// execution of that depth has (data-independent; a test checks it).
pub struct Key {
    depth: usize,
    program: N,
    encoded: Vec<zheng::execution::NounToken>,
    vk: VerifyingKey,
    cycles: u64,
}

impl Key {
    pub fn new(depth: usize) -> Result<Self, Error> {
        let program = program(depth)?;
        let encoded = ExecutionStatement::encode_program(&program).map_err(Error::Proof)?;
        // the cost from a dummy witness: the program's control flow does not
        // depend on secrets (branches pick between two conses of equal cost)
        let dummy = Path { siblings: vec![EMPTY; depth], directions: vec![0; depth] };
        let (st, _, _) =
            zheng::execution::private::prepare_execution(&program, &[0; 12], &secret_stream(&Secret([0; 4]), &dummy), BUDGET)
                .map_err(Error::Proof)?;
        let vk = VerifyingKey::for_execution(&st.execution).map_err(Error::Proof)?;
        Ok(Self { depth, program, encoded, vk, cycles: st.execution.cycles })
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    /// The cost every proof of this depth states.
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// The zheng statement a verifier derives from the P1 statement alone.
    pub fn zheng_statement(&self, statement: &Statement) -> PrivateStatement {
        PrivateStatement {
            execution: ExecutionStatement {
                program: self.encoded.clone(),
                public_input: statement.inputs(),
                public_output: statement.outputs(),
                cycles: self.cycles,
                budget: BUDGET,
            },
        }
    }
}

/// A P1 authorship proof: opaque veil bytes (no statement inside; the
/// verifier derives it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorshipProof(pub VeilProof);

impl AuthorshipProof {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        VeilProof::from_bytes(bytes).map(Self).map_err(Error::Proof)
    }
}

/// Prove authorship of the content `content` under the set root `root`,
/// with nullifier scope `scope`. Refuses a secret/path that does not open
/// to `root` before any proving work.
pub fn prove(
    key: &Key,
    secret: &Secret,
    path: &Path,
    root: &Digest,
    content: &Digest,
    scope: &Digest,
) -> Result<(Statement, AuthorshipProof), Error> {
    if path.siblings.len() != key.depth || path.directions.len() != key.depth {
        return Err(Error::Depth);
    }
    if path.directions.iter().any(|&b| b > 1) {
        return Err(Error::NonCanonical);
    }
    path.siblings.iter().try_for_each(canonical)?;
    if root_of(&secret.commitment(), path) != *root {
        return Err(Error::NotAMember);
    }
    let statement = Statement {
        depth: key.depth,
        root: *root,
        content: *content,
        scope: *scope,
        nullifier: secret.nullifier(scope),
    };
    statement.check()?;
    let (st, proof) = veil::prove(&key.program, &statement.inputs(), &secret_stream(secret, path), BUDGET)
        .map_err(Error::Proof)?;
    if st != key.zheng_statement(&statement) {
        return Err(Error::Proof("execution disagrees with the native statement".into()));
    }
    Ok((statement, AuthorshipProof(proof)))
}

/// Verify a P1 authorship proof against a statement whose root the caller
/// trusts (a committed authority set) and whose content commitment it
/// computed from the signal itself.
pub fn verify(key: &Key, statement: &Statement, proof: &AuthorshipProof) -> Result<(), Error> {
    if statement.depth != key.depth {
        return Err(Error::Depth);
    }
    statement.check()?;
    veil::verify_with(&key.zheng_statement(statement), &proof.0, Some(&key.vk)).map_err(Error::Proof)
}

#[cfg(test)]
#[path = "authorship_tests.rs"]
mod tests;
