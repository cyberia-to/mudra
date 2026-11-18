//! The P1 nox program `P_d` and its public-input layout.
//!
//! Normative: `specs/authorship.md` § program. The builders below are the
//! bare noun constructors zheng's execution noun needs; [`program`] puts
//! them together into the relation the prover and the verifier share.

use super::{D_AUTH, D_NULL, Error, MAX_DEPTH, MIN_DEPTH};
use zheng::execution::ExecutionNoun as N;

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
pub const INPUTS: usize = 12;

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
