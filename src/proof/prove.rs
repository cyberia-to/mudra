// ---
// tags: mudra, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! The nox → zheng proving pipeline (milestone 2e).
//!
//! Milestones 2a–2d express secp256k1 ECDSA verification in Goldilocks-limb
//! arithmetic: every operation is a nox pattern (`add`/`sub`/`mul`/`inv`,
//! `and`/`xor`/`not`/`shl`). This module closes the loop on the *mechanism* that
//! turns such a computation into a checked certificate:
//!
//! 1. build a nox program (a formula noun tree),
//! 2. run [`nox::reduce`] natively to get the result,
//! 3. [`certify_execution`]: zheng public profile v3 — statement (program,
//!    inputs, outputs, cost, budget) plus the free witness positions,
//! 4. [`verify_certificate`]: the verifier recompiles the relation from the
//!    program and checks every row exactly.
//!
//! Profile v3 discloses the witness and is linear in size; it is a soundness
//! floor, not a succinct or zero-knowledge proof (zheng specs/execution.md).
//! The 0.3.x trace fold this module used before (`zheng::commit`/`verify`)
//! bound no statement and checked no fold, so it proved nothing.
//!
//! It is demonstrated on the Goldilocks field operations that every limb of the
//! verifier decomposes into (`a·b + c`). Assembling the *full* `verify_claim`
//! into one nox program is the remaining arithmetization work. Gated behind
//! the `prove` feature because it pulls the whole proof system.

use nebu::Goldilocks;
use nox::{reduce, NullCalls, Order, Outcome, Reduction, VecTrace};
use zheng::execution::{
    certify_execution, verify_certificate, Certificate, ExecutionNoun, ExecutionStatement,
};

/// nox opcode tags (see nox reduction patterns).
const QUOTE: u64 = 1;
const ADD: u64 = 5;
const MUL: u64 = 7;

/// Reduction budget for the demonstration program.
const BUDGET: u64 = 1000;

fn atom(v: u64) -> ExecutionNoun {
    ExecutionNoun::Atom(v)
}

fn pair(a: ExecutionNoun, b: ExecutionNoun) -> ExecutionNoun {
    ExecutionNoun::Pair(Box::new(a), Box::new(b))
}

/// `[5 [[7 [[1 a] [1 b]]] [1 c]]]` — `a·b + c` as a nox formula noun.
fn madd_program(a: u64, b: u64, c: u64) -> ExecutionNoun {
    let quote = |v| pair(atom(QUOTE), atom(v));
    let mul = pair(atom(MUL), pair(quote(a), quote(b)));
    pair(atom(ADD), pair(mul, quote(c)))
}

/// Run `a·b + c` on nox natively, returning its execution trace and result —
/// two arithmetic patterns (`mul`, `add`) plus the quoting/compose steps
/// around them, the atoms every field-limb operation is built from.
fn run_madd(a: u64, b: u64, c: u64) -> (VecTrace, u64) {
    let mut r = Reduction::<4096>::new();
    let subject = r.atom(Goldilocks::new(0)).expect("subject");

    let quote = |r: &mut Reduction<4096>, v: u64| -> Order {
        let tag = r.atom(Goldilocks::new(QUOTE)).unwrap();
        let val = r.atom(Goldilocks::new(v)).unwrap();
        r.pair(tag, val).unwrap() // [1 v]
    };

    let qa = quote(&mut r, a);
    let qb = quote(&mut r, b);
    let qc = quote(&mut r, c);

    // [7 [ [1 a] [1 b] ]]  — a·b
    let mul_tag = r.atom(Goldilocks::new(MUL)).unwrap();
    let mul_body = r.pair(qa, qb).unwrap();
    let mul = r.pair(mul_tag, mul_body).unwrap();

    // [5 [ (a·b) [1 c] ]]  — add the product to c
    let add_tag = r.atom(Goldilocks::new(ADD)).unwrap();
    let add_body = r.pair(mul, qc).unwrap();
    let formula = r.pair(add_tag, add_body).unwrap();

    let mut trace = VecTrace::default();
    let result = match reduce(&mut r, subject, formula, BUDGET, &NullCalls, &mut trace) {
        Outcome::Ok(res, _) => r.atom_value(res).expect("atom result").as_u64(),
        other => panic!("nox reduce failed: {other:?}"),
    };
    (trace, result)
}

/// Certify `a·b + c` end to end: run it on nox, certify the same program with
/// zheng public profile v3. Returns the statement, its certificate and the
/// natively computed value (the statement's output must equal it).
pub fn prove_madd(a: u64, b: u64, c: u64) -> (ExecutionStatement, Certificate, u64) {
    let (_, result) = run_madd(a, b, c);
    let (statement, certificate) =
        certify_execution(&madd_program(a, b, c), &[], BUDGET).expect("zheng certificate v3");
    (statement, certificate, result)
}

/// Verify a certificate against its statement.
pub fn verify_proof(statement: &ExecutionStatement, certificate: &Certificate) -> bool {
    verify_certificate(statement, certificate).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn madd_proof_verifies_end_to_end() {
        let (stmt, cert, result) = prove_madd(3, 5, 7);
        assert_eq!(result, 22, "3·5 + 7 computed correctly by nox");
        assert_eq!(stmt.public_output, vec![22], "the statement carries the result");
        assert!(verify_proof(&stmt, &cert), "the zheng certificate verifies");
    }

    #[test]
    fn trace_has_multiple_pattern_rows() {
        let (trace, _) = run_madd(6, 7, 1);
        assert!(trace.0.len() >= 2, "trace spans several reduction steps");
    }

    #[test]
    fn wrong_output_is_rejected() {
        let (mut stmt, cert, _) = prove_madd(3, 5, 7);
        stmt.public_output[0] = 23;
        assert!(!verify_proof(&stmt, &cert), "a false result does not verify");
    }

    #[test]
    fn budget_binds_the_step_count() {
        // A statement that claims a budget below the cost is rejected — the
        // certificate cannot understate the work done.
        let (stmt, cert, _) = prove_madd(3, 5, 7);
        assert!(stmt.cycles > 1);
        let mut tight = stmt.clone();
        tight.budget = stmt.cycles - 1;
        assert!(!verify_proof(&tight, &cert), "understated budget rejected");
        assert!(certify_execution(&madd_program(3, 5, 7), &[], stmt.cycles - 1).is_err());
        let mut cycles = stmt;
        cycles.cycles -= 1;
        assert!(!verify_proof(&cycles, &cert), "understated cost rejected");
    }
}
