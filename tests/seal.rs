//! seal: FIPS 203 ML-KEM-768 profile behaves as the spec requires.

use mudra::seal::{Ciphertext, Profile, PublicKey, SealError, SecretKey, decap, encap, keygen};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn rng() -> StdRng {
    StdRng::seed_from_u64(1)
}

#[test]
fn encap_then_decap_agree() {
    let (sk, pk) = keygen(&mut rng());
    let (ct, ss_sender) = encap(&pk, &mut rng());
    let ss_recipient = decap(&sk, &ct).unwrap();
    assert_eq!(ss_sender, ss_recipient);
    assert_eq!(ss_sender.derive(b"payload"), ss_recipient.derive(b"payload"));
    assert_ne!(ss_sender.derive(b"payload"), ss_sender.derive(b"mac"));
}

#[test]
fn tampered_ciphertext_rejects_implicitly() {
    let (sk, pk) = keygen(&mut rng());
    let (ct, ss) = encap(&pk, &mut rng());
    let mut bytes = ct.to_bytes();
    bytes[100] ^= 0x01;
    let bad = Ciphertext::from_bytes(&bytes).unwrap();
    // no error, no oracle — just a different secret
    let ss_bad = decap(&sk, &bad).unwrap();
    assert_ne!(ss_bad, ss);
}

#[test]
fn envelopes_round_trip_and_carry_the_profile() {
    let (sk, pk) = keygen(&mut rng());
    let pk2 = PublicKey::from_bytes(&pk.to_bytes()).unwrap();
    let sk2 = SecretKey::from_bytes(&sk.to_bytes()).unwrap();
    assert_eq!(pk2.profile(), Profile::MlKem768);
    assert_eq!(pk.to_bytes(), pk2.to_bytes());
    assert_eq!(sk2.public_key().to_bytes(), pk.to_bytes());
    let (ct, ss) = encap(&pk2, &mut rng());
    assert_eq!(decap(&sk2, &ct).unwrap(), ss);
    assert_eq!(pk.to_bytes().len(), 1 + 1184);
    assert_eq!(ct.to_bytes().len(), 1 + 1088);
}

#[test]
fn malformed_envelopes_are_hard_errors() {
    assert_eq!(PublicKey::from_bytes(&[]).err(), Some(SealError::UnknownProfile));
    assert_eq!(PublicKey::from_bytes(&[0x09, 1, 2]).err(), Some(SealError::UnknownProfile));
    assert_eq!(
        Ciphertext::from_bytes(&[0x03, 1, 2, 3]).err(),
        Some(SealError::BadLength { expected: 1088, got: 3 })
    );
}
