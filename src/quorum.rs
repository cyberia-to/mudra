//! quorum — split a secret into k-of-n shares (`specs/quorum.md`).
//!
//! Shamir secret sharing over the Goldilocks field: the secret is the constant
//! term of a random polynomial of degree k−1, share i is its evaluation at the
//! non-zero point i, and any k shares recover the constant by Lagrange
//! interpolation at zero. Fewer than k shares carry no information about the
//! secret (information-theoretic).
//!
//! Built: `split`, `recover`. Not built: verifiable sharing, DKG and threshold
//! decryption — the spec's VSS hashes polynomial coefficients and then asks for
//! homomorphic share checks, which a hash cannot provide (audit 2026-09-16).
//! Those need a commitment with the right algebra before any code is honest.

use nebu::Goldilocks;
use rand_core::Rng;

/// One share: the evaluation point (never zero) and the polynomial's value there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Share {
    pub index: u64,
    pub value: Goldilocks,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ThresholdError {
    /// k must be at least 1 and at most n.
    BadThreshold { k: usize, n: usize },
    /// Fewer shares than the threshold.
    NotEnoughShares { have: usize, need: usize },
    /// Two shares claim the same evaluation point.
    DuplicateIndex(u64),
    /// A share at index zero would leak the secret directly.
    ZeroIndex,
}

/// A uniformly random canonical field element.
fn random_element(rng: &mut impl Rng) -> Goldilocks {
    // rejection-sample below p so the coefficient distribution is uniform
    const P: u64 = 0xFFFF_FFFF_0000_0001;
    loop {
        let v = rng.next_u64();
        if v < P {
            return Goldilocks::new(v);
        }
    }
}

/// Split `secret` into `n` shares of which any `k` recover it.
pub fn split(
    secret: Goldilocks,
    k: usize,
    n: usize,
    rng: &mut impl Rng,
) -> Result<Vec<Share>, ThresholdError> {
    if k == 0 || n == 0 || k > n {
        return Err(ThresholdError::BadThreshold { k, n });
    }
    // coefficients a_0 = secret, a_1..a_{k-1} random
    let mut coeffs = Vec::with_capacity(k);
    coeffs.push(secret.canonicalize());
    for _ in 1..k {
        coeffs.push(random_element(rng));
    }
    Ok((1..=n as u64)
        .map(|i| Share {
            index: i,
            value: eval(&coeffs, Goldilocks::new(i)),
        })
        .collect())
}

/// Horner evaluation of the polynomial with the given coefficients at `x`.
fn eval(coeffs: &[Goldilocks], x: Goldilocks) -> Goldilocks {
    let mut acc = Goldilocks::ZERO;
    for &c in coeffs.iter().rev() {
        acc = acc * x + c;
    }
    acc.canonicalize()
}

/// Recover the secret from at least `k` distinct shares.
///
/// Uses the first `k` shares after validation; extra shares are ignored, so
/// consistency between shares is not checked here (that is what verifiable
/// sharing would add).
pub fn recover(shares: &[Share], k: usize) -> Result<Goldilocks, ThresholdError> {
    if k == 0 {
        return Err(ThresholdError::BadThreshold { k, n: shares.len() });
    }
    if shares.len() < k {
        return Err(ThresholdError::NotEnoughShares {
            have: shares.len(),
            need: k,
        });
    }
    let used = &shares[..k];
    for (a, s) in used.iter().enumerate() {
        if s.index == 0 {
            return Err(ThresholdError::ZeroIndex);
        }
        if used[..a].iter().any(|t| t.index == s.index) {
            return Err(ThresholdError::DuplicateIndex(s.index));
        }
    }
    // Lagrange interpolation at x = 0:
    //   secret = Σ_j y_j · Π_{m≠j} x_m / (x_m − x_j)
    let mut secret = Goldilocks::ZERO;
    for (j, sj) in used.iter().enumerate() {
        let xj = Goldilocks::new(sj.index);
        let mut num = Goldilocks::ONE;
        let mut den = Goldilocks::ONE;
        for (m, sm) in used.iter().enumerate() {
            if m == j {
                continue;
            }
            let xm = Goldilocks::new(sm.index);
            num = num * xm;
            den = den * (xm - xj);
        }
        secret = secret + sj.value * num * den.inv();
    }
    Ok(secret.canonicalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(7)
    }

    #[test]
    fn any_k_of_n_recover_the_secret() {
        let secret = Goldilocks::new(0xDEAD_BEEF_CAFE);
        let shares = split(secret, 3, 5, &mut rng()).unwrap();
        assert_eq!(shares.len(), 5);
        for pick in [[0usize, 1, 2], [0, 2, 4], [1, 3, 4], [2, 3, 4]] {
            let subset: Vec<Share> = pick.iter().map(|&i| shares[i]).collect();
            assert_eq!(recover(&subset, 3).unwrap(), secret);
        }
    }

    #[test]
    fn k_minus_one_shares_are_refused() {
        let shares = split(Goldilocks::new(42), 3, 5, &mut rng()).unwrap();
        assert_eq!(
            recover(&shares[..2], 3),
            Err(ThresholdError::NotEnoughShares { have: 2, need: 3 })
        );
    }

    #[test]
    fn k_minus_one_shares_recover_a_different_value_for_a_different_secret() {
        // information-theoretic: with 2 of 3 shares the "recovered" constant of a
        // degree-1 fit is not the secret
        let secret = Goldilocks::new(1234);
        let shares = split(secret, 3, 3, &mut rng()).unwrap();
        let wrong = recover(&shares[..2], 2).unwrap();
        assert_ne!(wrong, secret);
    }

    #[test]
    fn duplicate_and_zero_indices_are_refused() {
        let shares = split(Goldilocks::new(1), 2, 3, &mut rng()).unwrap();
        let dup = [shares[0], shares[0]];
        assert_eq!(recover(&dup, 2), Err(ThresholdError::DuplicateIndex(1)));
        let zero = [
            Share {
                index: 0,
                value: Goldilocks::ZERO,
            },
            shares[1],
        ];
        assert_eq!(recover(&zero, 2), Err(ThresholdError::ZeroIndex));
    }

    #[test]
    fn thresholds_are_validated() {
        assert!(matches!(
            split(Goldilocks::ONE, 4, 3, &mut rng()),
            Err(ThresholdError::BadThreshold { k: 4, n: 3 })
        ));
        assert!(matches!(
            split(Goldilocks::ONE, 0, 3, &mut rng()),
            Err(ThresholdError::BadThreshold { .. })
        ));
    }

    #[test]
    fn k_equals_one_is_the_secret_itself() {
        let secret = Goldilocks::new(99);
        let shares = split(secret, 1, 4, &mut rng()).unwrap();
        for s in &shares {
            assert_eq!(s.value, secret);
        }
    }
}
