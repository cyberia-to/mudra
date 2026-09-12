// SPDX-License-Identifier: Apache-2.0
// Adapted from neptuneprivacy/xnt-core cc8e8704026c916d545f02aa4d03b5693b6a58dd.
// Modified into a synthetic predicate/serialization check; see README.md.

use bech32::ToBase32;
use triton_vm::prelude::*;
use twenty_first::math::lattice::kem;

// Exact encoding copied from pinned XNT common.rs165–182.
fn bytes_to_bfes(bytes: &[u8]) -> Vec<BFieldElement> {
    let mut padded_bytes = bytes.to_vec();
    while !padded_bytes.len().is_multiple_of(8) {
        padded_bytes.push(0);
    }
    let mut bfes = vec![BFieldElement::new(bytes.len() as u64)];
    for chunk in padded_bytes.chunks(8) {
        let int = u64::from_be_bytes(chunk.try_into().unwrap());
        if int < BFieldElement::P - 1 {
            bfes.push(BFieldElement::new(int));
        } else {
            bfes.push(BFieldElement::new(BFieldElement::P - 1));
            bfes.push(BFieldElement::new(int & 0xffffffff));
        }
    }
    bfes
}
#[derive(serde::Serialize)]
struct GenerationAddress {
    receiver_identifier: BFieldElement,
    encryption_key: kem::PublicKey,
    receiver_postimage: Digest,
    lock_postimage: Digest,
}
fn main() {
    // Synthetic public byte strings only; no live address, account or network.
    for case in 0..3 {
        let pk = [case as u8; 64];
        let public_preimage = Tip5::hash_varlen(&bytes_to_bfes(&pk));
        let public_postimage = public_preimage.hash();
        let push = public_postimage
            .values()
            .iter()
            .rev()
            .map(|x| triton_instr!(push x.value()))
            .collect::<Vec<_>>();
        let program = triton_program!(divine 5 hash {&push} assert_vector read_io 5 halt);
        let input = PublicInput::new(vec![BFieldElement::new(91); 5]);
        let witness = NonDeterminism::new(public_preimage.reversed().values());
        assert!(VM::run(program.clone(), input.clone(), witness).is_ok());
        let wrong = NonDeterminism::new([BFieldElement::new(1); 5]);
        assert!(VM::run(program, input, wrong).is_err());
    }
    let (_, pk) = kem::keygen([7u8; 32]);
    let address = GenerationAddress {
        receiver_identifier: BFieldElement::new(11),
        encryption_key: pk,
        receiver_postimage: Digest::default(),
        lock_postimage: Digest::default(),
    };
    let raw = bincode::serialize(&address).unwrap();
    let encoded = bech32::encode("xntnwm", raw.to_base32(), bech32::Variant::Bech32m).unwrap();
    let short = bech32::encode("xntctm", [0u8; 64].to_base32(), bech32::Variant::Bech32m).unwrap();
    let sub = bech32::encode("xntctam", [0u8; 72].to_base32(), bech32::Variant::Bech32m).unwrap();
    println!("synthetic_public_witness_vm_accept=3 wrong_witness_reject=3");
    println!(
        "generation_kem_pk_bytes={} generation_address_payload_bytes={} generation_address_chars={} dctidh_address_payload_bytes=64 dctidh_address_chars={} dctidh_subaddress_payload_bytes=72 dctidh_subaddress_chars={} generation_kem_ciphertext_bfes={}",
        bincode::serialize(&pk).unwrap().len(),
        raw.len(),
        encoded.len(),
        short.len(),
        sub.len(),
        kem::CIPHERTEXT_SIZE_IN_BFES
    );
}
