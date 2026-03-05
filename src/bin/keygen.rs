use mineclaw::encryption::EncryptionManager;
use std::env;
use std::io::{self, Read, Write};

/// Keygen CLI Tool
/// 
/// Modes:
/// 1. Generate Key (default): Prints new key to stdout. Logs to stderr.
/// 2. Encrypt: Reads plaintext from stdin, key from arg/env, prints ciphertext to stdout.
/// 3. Decrypt: Reads ciphertext from stdin, key from arg/env, prints plaintext to stdout.
fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("generate");

    match mode {
        "generate" => generate_key(),
        "encrypt" => encrypt_data(&args),
        "decrypt" => decrypt_data(&args),
        "--help" | "-h" => print_help(),
        _ => {
            eprintln!("Unknown mode: {}", mode);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!("MineClaw Encryption Tool");
    eprintln!("Usage:");
    eprintln!("  keygen generate                 Generate a new random key (stdout)");
    eprintln!("  keygen encrypt <key>            Encrypt stdin to stdout");
    eprintln!("  keygen decrypt <key>            Decrypt stdin to stdout");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -q --bin keygen generate > my.key");
    eprintln!("  echo 'secret' | cargo run -q --bin keygen encrypt $(cat my.key)");
    eprintln!("  echo 'encrypted:...' | cargo run -q --bin keygen decrypt $(cat my.key)");
}

fn generate_key() {
    eprintln!("Generating new encryption key...");
    let key = EncryptionManager::generate_key();
    // Only print the key to stdout
    print!("{}", key);
    io::stdout().flush().unwrap();
    eprintln!("\nDone. Key written to stdout.");
}

fn encrypt_data(args: &[String]) {
    let key = match args.get(2) {
        Some(k) => k.trim().to_string(),
        None => {
            eprintln!("Error: Key required. Usage: keygen encrypt <key>");
            std::process::exit(1);
        }
    };

    let manager = EncryptionManager::new(&key).unwrap_or_else(|e| {
        eprintln!("Error: Invalid key: {}", e);
        std::process::exit(1);
    });

    let mut plaintext = String::new();
    io::stdin().read_to_string(&mut plaintext).unwrap();
    
    // Trim newline from input if piped from echo
    let plaintext = plaintext.trim();

    if plaintext.is_empty() {
        eprintln!("Warning: Encrypting empty string");
    }

    match manager.encrypt(plaintext) {
        Ok(ciphertext) => {
            print!("encrypted:{}", ciphertext);
            io::stdout().flush().unwrap();
        },
        Err(e) => {
            eprintln!("Error: Encryption failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn decrypt_data(args: &[String]) {
    let key = match args.get(2) {
        Some(k) => k.trim().to_string(),
        None => {
            eprintln!("Error: Key required. Usage: keygen decrypt <key>");
            std::process::exit(1);
        }
    };

    let manager = EncryptionManager::new(&key).unwrap_or_else(|e| {
        eprintln!("Error: Invalid key: {}", e);
        std::process::exit(1);
    });

    let mut ciphertext_input = String::new();
    io::stdin().read_to_string(&mut ciphertext_input).unwrap();
    let ciphertext_input = ciphertext_input.trim();

    // Strip "encrypted:" prefix if present
    let ciphertext_base64 = ciphertext_input.trim_start_matches("encrypted:");

    match manager.decrypt(ciphertext_base64) {
        Ok(plaintext) => {
            print!("{}", plaintext);
            io::stdout().flush().unwrap();
        },
        Err(e) => {
            eprintln!("Error: Decryption failed: {}", e);
            std::process::exit(1);
        }
    }
}
