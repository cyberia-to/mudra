//! Bounded external-interface interoperability check against OpenSSL >= 3.5.
//! All seeds are public test data. Never use the generated keys for funds.
use qp_rusty_crystals_dilithium::{SensitiveBytes32, ml_dsa_65, ml_dsa_87};
use std::{fs, path::Path, process::Command};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn openssl(dir: &Path, args: &[&str], success: bool) -> Vec<u8> {
    let output = Command::new("openssl")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("OpenSSL >= 3.5 must be installed");
    assert_eq!(
        output.status.success(),
        success,
        "openssl {args:?}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !success {
        // A CLI/configuration error must not be counted as a rejected signature.
        assert!(String::from_utf8_lossy(&output.stdout).contains("Signature Verification Failure"));
    }
    output.stdout
}

fn verify(dir: &Path, context: &[u8], success: bool) {
    openssl(
        dir,
        &[
            "pkeyutl",
            "-verify",
            "-rawin",
            "-pubin",
            "-inkey",
            "public.der",
            "-keyform",
            "DER",
            "-in",
            "message.bin",
            "-sigfile",
            "quantus.sig",
            "-pkeyopt",
            &format!("hexcontext-string:{}", hex(context)),
        ],
        success,
    );
}

fn sign(dir: &Path, context: &[u8], deterministic: bool) -> Vec<u8> {
    openssl(
        dir,
        &[
            "pkeyutl",
            "-sign",
            "-rawin",
            "-inkey",
            "test-only.pem",
            "-in",
            "message.bin",
            "-out",
            "openssl.sig",
            "-pkeyopt",
            &format!("hexcontext-string:{}", hex(context)),
            "-pkeyopt",
            if deterministic {
                "deterministic:1"
            } else {
                "deterministic:0"
            },
        ],
        true,
    );
    fs::read(dir.join("openssl.sig")).unwrap()
}

macro_rules! crosscheck {
    ($module:ident, $name:literal, $dir:expr) => {{
        let dir = $dir;
        // OpenSSL pkeyutl 3.6.2 refuses empty input files before verification.
        // Keep empty-message coverage local; use 1/24/4096-byte inputs across backends.
        let messages = [
            vec![0],
            b"cyber identity action v1".to_vec(),
            (0..4096).map(|x| x as u8).collect(),
        ];
        let contexts = [
            Vec::new(),
            b"cyber/neuron-authority/v1".to_vec(),
            (0..255).map(|x| x as u8).collect(),
        ];
        let mut cases = 0;
        for seed_index in 0..3u8 {
            let seed: [u8; 32] = std::array::from_fn(|i| (i as u8).wrapping_add(seed_index));
            openssl(
                dir,
                &[
                    "genpkey",
                    "-algorithm",
                    $name,
                    "-pkeyopt",
                    &format!("hexseed:{}", hex(&seed)),
                    "-out",
                    "test-only.pem",
                ],
                true,
            );
            openssl(
                dir,
                &[
                    "pkey",
                    "-in",
                    "test-only.pem",
                    "-pubout",
                    "-outform",
                    "DER",
                    "-out",
                    "public.der",
                ],
                true,
            );
            let mut test_entropy = seed;
            let key = $module::Keypair::generate(&mut SensitiveBytes32::new(&mut test_entropy));
            let public = key.public().to_bytes();
            let empty_signature = key.sign(b"", None, None).unwrap();
            assert!(key.verify(b"", &empty_signature, None));
            let der = fs::read(dir.join("public.der")).unwrap();
            // OpenSSL's SPKI is a 22-byte header followed by the raw FIPS 204 key.
            assert_eq!(der.len(), public.len() + 22);
            assert_eq!(&der[22..], public.as_slice(), "seed -> public key differs");
            let mut other_seed = [0x77; 32];
            let other = $module::Keypair::generate(&mut SensitiveBytes32::new(&mut other_seed));
            for message in &messages {
                fs::write(dir.join("message.bin"), message).unwrap();
                for context in &contexts {
                    let signature = key.sign(message, Some(context), None).unwrap();
                    fs::write(dir.join("quantus.sig"), signature).unwrap();
                    verify(dir, context, true);
                    let independent = sign(dir, context, true);
                    assert_eq!(
                        independent.as_slice(),
                        signature.as_slice(),
                        "deterministic signature differs"
                    );
                    assert!(key.verify(message, &independent, Some(context)));
                    let randomized = sign(dir, context, false);
                    assert!(key.verify(message, &randomized, Some(context)));
                    let mut hedge_bytes = [0x42; 32];
                    let hedge = SensitiveBytes32::new(&mut hedge_bytes);
                    let hedged = key.sign(message, Some(context), Some(&hedge)).unwrap();
                    fs::write(dir.join("quantus.sig"), hedged).unwrap();
                    verify(dir, context, true);

                    // Public API rejects altered messages, contexts, keys and encodings.
                    let wrong_context = if context.is_empty() {
                        b"wrong".as_slice()
                    } else {
                        b"".as_slice()
                    };
                    assert!(!key.verify(message, &signature, Some(wrong_context)));
                    fs::write(dir.join("quantus.sig"), signature).unwrap();
                    verify(dir, wrong_context, false);
                    let mut wrong_message = message.clone();
                    wrong_message.push(0xff);
                    assert!(!key.verify(&wrong_message, &signature, Some(context)));
                    fs::write(dir.join("message.bin"), &wrong_message).unwrap();
                    verify(dir, context, false);
                    fs::write(dir.join("message.bin"), message).unwrap();
                    assert!(!other.verify(message, &signature, Some(context)));
                    let mut bitflip = signature.to_vec();
                    bitflip[0] ^= 1;
                    let mut appended = signature.to_vec();
                    appended.push(0);
                    for invalid in [
                        &signature[..signature.len() - 1],
                        bitflip.as_slice(),
                        appended.as_slice(),
                    ] {
                        assert!(!key.verify(message, invalid, Some(context)));
                        fs::write(dir.join("quantus.sig"), invalid).unwrap();
                        verify(dir, context, false);
                    }
                    assert!(key.sign(message, Some(&[0u8; 256]), None).is_err());
                    assert!(!key.verify(message, &signature, Some(&[0u8; 256])));
                    if context.is_empty() {
                        assert_eq!(signature, key.sign(message, None, None).unwrap());
                    }
                    cases += 1;
                }
            }
        }
        println!(
            "{}: {cases} seed/message/context cases passed; pk={} signature={} bytes",
            $name,
            $module::PUBLICKEYBYTES,
            $module::SIGNBYTES
        );
        cases
    }};
}

fn main() {
    let dir = std::env::temp_dir().join(format!("mudra-mldsa-crosscheck-{}", std::process::id()));
    fs::create_dir(&dir).expect("fresh scratch directory");
    let version = openssl(&dir, &["version"], true);
    print!("{}", String::from_utf8_lossy(&version));
    let cases =
        crosscheck!(ml_dsa_65, "ML-DSA-65", &dir) + crosscheck!(ml_dsa_87, "ML-DSA-87", &dir);
    fs::remove_dir_all(&dir).unwrap();
    println!(
        "PASS: {cases} cases; both signing directions, deterministic byte equality, hedged signing, negative mutations. This is interoperability evidence, not a security proof."
    );
}
