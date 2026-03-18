//! Sign registry entries with ed25519. Use keygen to create a keypair, then sign to produce a signed registry.
//!
//! Usage:
//!   cargo run --bin registry_sign -- keygen
//!   cargo run --bin registry_sign -- sign <registry.json> <privkey_b64>

use base64::{engine::general_purpose::STANDARD, Engine};
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("keygen") => {
            let rng = SystemRandom::new();
            let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("keygen failed");
            let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("from_pkcs8 failed");
            println!("private_key: {}", STANDARD.encode(pkcs8.as_ref()));
            println!("public_key:  {}", STANDARD.encode(pair.public_key().as_ref()));
        }
        Some("sign") => {
            if args.len() < 4 {
                eprintln!("Usage: registry_sign sign <registry.json> <privkey_b64>");
                std::process::exit(1);
            }
            let registry_path = &args[2];
            let privkey_b64 = &args[3];
            let privkey_bytes = STANDARD
                .decode(privkey_b64.as_bytes())
                .expect("invalid base64 private key");
            let pair = Ed25519KeyPair::from_pkcs8(&privkey_bytes).expect("invalid PKCS#8 private key");

            let content = std::fs::read_to_string(registry_path).expect("failed to read registry");
            let mut registry: serde_json::Value = serde_json::from_str(&content).expect("invalid JSON");

            let entries = registry["entries"]
                .as_array_mut()
                .expect("registry must have entries array");

            for entry in entries {
                let platform_id_hex = entry["platform_id_hex"]
                    .as_str()
                    .expect("entry must have platform_id_hex");
                let provider = entry["provider"].as_str().expect("entry must have provider");
                let region = entry["region"].as_str().expect("entry must have region");
                let verification_level = entry["verification_level"]
                    .as_u64()
                    .unwrap_or(0);
                let msg = format!(
                    "{}|{}|{}|{}",
                    platform_id_hex.trim().to_lowercase(),
                    provider,
                    region,
                    verification_level
                );
                let sig = pair.sign(msg.as_bytes());
                entry["signature"] = serde_json::json!(STANDARD.encode(sig.as_ref()));
                entry["signer_pubkey"] =
                    serde_json::json!(STANDARD.encode(pair.public_key().as_ref()));
            }
            println!("{}", serde_json::to_string_pretty(&registry).expect("serialize"));
        }
        _ => {
            eprintln!("Usage: registry_sign keygen | sign <registry.json> <privkey_b64>");
            std::process::exit(1);
        }
    }
}
