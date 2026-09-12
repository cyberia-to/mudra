use k256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

mod proof {
    #[path = "/Users/master/cyber/mudra/src/proof/field.rs"]
    pub mod field;
    #[path = "/Users/master/cyber/mudra/src/proof/scalar.rs"]
    pub mod scalar;
    #[path = "/Users/master/cyber/mudra/src/proof/curve.rs"]
    pub mod curve;
    #[path = "/Users/master/cyber/mudra/src/proof/ecdsa.rs"]
    pub mod ecdsa;
}

fn bytes(hex: &str) -> [u8; 32] {
    std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let malformed = std::panic::catch_unwind(|| mudra::Claim::decode("address 😀 00 00"));
    println!("claim_decode_unicode_panics={}", malformed.is_err());

    let upper = mudra::domain::DomainKey::derive(&[3; 32], "Example.com", "pussy").unwrap();
    let lower = mudra::domain::DomainKey::derive(&[3; 32], "example.com", "pussy").unwrap();
    println!("domain_case_changes_native_id={}", upper.native != lower.native);
    println!("empty_domain_and_zero_entropy_accepted={}", mudra::domain::DomainKey::derive(&[0; 32], "", "pussy").is_ok());

    // All keys/data below are public synthetic test values.
    let (gx, _) = proof::curve::Point::generator().to_affine().unwrap();
    let r = gx.to_bytes();
    let secret = proof::scalar::Scalar::from_bytes(&r).inv().to_bytes();
    let key = mudra::SigningKey::from_slice(&secret).unwrap();
    let public = mudra::cosmos::compressed(key.verifying_key());
    let mut one = [0; 32]; one[31] = 1;
    let z = [0; 32];
    let valid = Signature::from_scalars(r, one).unwrap();
    assert!(key.verifying_key().verify_prehash(&z, &valid).is_ok());
    assert!(proof::ecdsa::verify(&public, &r, &one, &z));
    let noncanonical_s = bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364142");
    println!("out_of_range_s_native_parse_accepts={}", Signature::from_scalars(r, noncanonical_s).is_ok());
    println!("out_of_range_s_emulator_verifies={}", proof::ecdsa::verify(&public, &r, &noncanonical_s, &z));
    let high_s = bytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140");
    let high_sig = Signature::from_scalars(r, high_s).unwrap();
    println!("high_s_native_verifies={}", key.verifying_key().verify_prehash(&z, &high_sig).is_ok());
    println!("high_s_emulator_verifies={}", proof::ecdsa::verify(&public, &r, &high_s, &z));

    let mut noncanonical_pk = [0; 33]; noncanonical_pk[0] = 2;
    noncanonical_pk[1..].copy_from_slice(&bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30"));
    println!("noncanonical_pubkey_native_accepts={}", VerifyingKey::from_sec1_bytes(&noncanonical_pk).is_ok());
    println!("noncanonical_pubkey_emulator_accepts={}", proof::curve::Point::from_sec1(&noncanonical_pk).is_some());
}
