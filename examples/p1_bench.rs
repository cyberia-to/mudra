//! P1 private authorship (`specs/authorship.md`): prove / verify / size per
//! authority-set depth, on the prover's default (CPU) backend.
//!
//! ```text
//! cargo run --release --features authorship --example p1_bench -- 12 16 20
//! ```
//!
//! A device backend is installed by the stand, not by this crate: mudra
//! carries no dependency on lens's `gpu` crate, because a dev-dependency
//! is not feature-gated and would make every default build need a lens
//! checkout. To time Metal, add `cyber-lens-gpu` as a dev-dependency on a
//! stand that has lens and call `cyber_lens_gpu::install` before the loop
//! (how the Metal column of `audit/p1-authorship-2026-10.md` was taken).
//!
//! `key ms` = program + relation compile + proving and verifying keys,
//! once per depth; `prove ms` = zheng `veil::prove_with_key` with that
//! held key; `verify ms` = with the held verifying key; `set ms` = the
//! native tree over 2^min(d, 16) members. `P1_REPS` (default 3), medians.

use std::time::Instant;

use mudra::authorship::{AuthoritySet, Key, Secret, prove, verify};

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let reps: usize = std::env::var("P1_REPS").ok().and_then(|s| s.parse().ok()).unwrap_or(3);
    let depths: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let depths = if depths.is_empty() { vec![12, 16, 20] } else { depths };
    println!("backend: the prover's default, reps {reps}");
    println!("| depth | members | set ms | key ms | cycles | prove ms | verify ms | proof bytes |");
    println!("|---|---|---|---|---|---|---|---|");
    for depth in depths {
        let members = 1usize << depth.min(16);
        let secrets: Vec<Secret> = (0..members as u64)
            .map(|i| {
                let mut s = [0u8; 32];
                s[..8].copy_from_slice(&i.to_le_bytes());
                Secret::from_seed(&s)
            })
            .collect();
        let t = Instant::now();
        let leaves: Vec<_> = secrets.iter().map(Secret::commitment).collect();
        let set = AuthoritySet::new(depth, &leaves).unwrap();
        let set_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let key = Key::new(depth).unwrap();
        let key_ms = t.elapsed().as_secs_f64() * 1e3;
        let index = members / 3;
        let path = set.path(index).unwrap();
        let (content, scope) = ([1, 2, 3, 4], [5, 6, 7, 8]);
        let mut prove_ms = vec![];
        let mut last = None;
        for _ in 0..reps {
            let t = Instant::now();
            last = Some(prove(&key, &secrets[index], &path, &set.root(), &content, &scope).unwrap());
            prove_ms.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let (statement, proof) = last.unwrap();
        let verify_ms: Vec<f64> = (0..reps)
            .map(|_| {
                let t = Instant::now();
                verify(&key, &statement, &proof).unwrap();
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        println!(
            "| {depth} | {members} | {set_ms:.0} | {key_ms:.0} | {} | {:.0} | {:.1} | {} |",
            key.cycles(),
            median(prove_ms),
            median(verify_ms),
            proof.as_bytes().len()
        );
    }
}
