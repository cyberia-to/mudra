// ---
// tags: mudra, rust, cli
// crystal-type: source
// crystal-domain: comp
// ---
//! mudra — the seal of the neuron.
//!
//!   mudra                       banner + help
//!   mudra new                   generate a fresh wallet
//!   mudra derive <mnemonic…>    seed → addresses, pubkey, native neuron
//!   mudra neuron <mnemonic…>    native neuron id = Hemera(pubkey)
//!   mudra claim  <mnemonic…>    sign a migration claim (legacy key → native neuron)
//!   mudra verify <claim>        verify a migration claim
//!
//! The bridge for migrating a Cosmos-SDK account (spacepussy first) into a
//! native neuron. `--bostrom` switches the address prefix from `pussy`.

use std::io::{self, IsTerminal};
use std::process::exit;

use mudra::claim::{self, Claim};
use mudra::{cosmos, seed};

// ── color ───────────────────────────────────────────────────────────────────

fn tty() -> bool {
    io::stdout().is_terminal()
}

fn paint(code: &str, s: &str) -> String {
    if tty() { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
}
fn dim(s: &str) -> String {
    paint("90", s)
}
fn cyan(s: &str) -> String {
    paint("36", s)
}
fn green(s: &str) -> String {
    paint("32", s)
}
fn yellow(s: &str) -> String {
    paint("33", s)
}
fn red(s: &str) -> String {
    paint("31", s)
}
fn bold(s: &str) -> String {
    paint("1", s)
}

const LOGO: &str = "\
\x1b[31m███╗   ███╗██╗   ██╗██████╗ ██████╗  █████╗ \x1b[0m
\x1b[33m████╗ ████║██║   ██║██╔══██╗██╔══██╗██╔══██╗\x1b[0m
\x1b[32m██╔████╔██║██║   ██║██║  ██║██████╔╝███████║\x1b[0m
\x1b[36m██║╚██╔╝██║██║   ██║██║  ██║██╔══██╗██╔══██║\x1b[0m
\x1b[34m██║ ╚═╝ ██║╚██████╔╝██████╔╝██║  ██║██║  ██║\x1b[0m
\x1b[35m╚═╝     ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝\x1b[0m";

fn banner() {
    if !tty() {
        return;
    }
    println!("{LOGO}");
    println!("{}", paint("37", "    the seal of the neuron"));
    println!(
        "{}",
        dim(
            "\n    secp256k1 · BIP-39/32 · coin type 118\n    \
             ripemd160 ∘ sha256 → bech32 · ADR-036 claims\n    \
             legacy key → Hemera(pubkey) · post-quantum ahead\n"
        )
    );
}

fn help() {
    println!(
        "{}\n  {}\n  {}\n  {}\n  {}\n  {}\n\n  {}\n    {}\n\n  {}",
        dim("commands"),
        format!("{}                    {}", bold("new"), dim("generate a fresh 24-word wallet")),
        format!("{}     {}", bold("derive <mnemonic…>"), dim("seed → pussy/bostrom address, pubkey, neuron")),
        format!("{}     {}", bold("neuron <mnemonic…>"), dim("native neuron id = Hemera(pubkey)")),
        format!("{}      {}", bold("claim <mnemonic…>"), dim("sign a claim binding the legacy key → neuron")),
        format!("{}          {}", bold("verify <claim>"), dim("verify a migration claim")),
        dim("flags"),
        format!("{}   {}", bold("--bostrom"), dim("use the bostrom prefix instead of pussy")),
        dim("mnemonic words follow the subcommand; default prefix is pussy"),
    );
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Print an error and exit non-zero.
fn die(msg: impl std::fmt::Display) -> ! {
    eprintln!("  {}: {msg}", red("error"));
    exit(1);
}

/// A labeled field line: dim label, bright value.
fn field(label: &str, value: &str) {
    println!("  {}  {}", dim(&format!("{label:<9}")), value);
}

// ── commands ────────────────────────────────────────────────────────────────

fn cmd_derive(mnemonic: &str, hrp: &str) {
    let key = seed::cosmos_key(mnemonic, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let addr = cosmos::address(&pubkey, hrp).unwrap_or_else(|e| die(e));
    let neuron = claim::neuron_of(&pubkey);
    field("account", &cyan(&addr));
    field("pubkey", &dim(&hex(&pubkey)));
    field("neuron", &green(&hex(&neuron)));
}

fn cmd_neuron(mnemonic: &str) {
    let key = seed::cosmos_key(mnemonic, "").unwrap_or_else(|e| die(e));
    let neuron = claim::neuron_of(&cosmos::compressed(key.verifying_key()));
    println!("  {}", green(&hex(&neuron)));
}

fn cmd_claim(mnemonic: &str, hrp: &str) {
    let key = seed::cosmos_key(mnemonic, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let neuron = claim::neuron_of(&pubkey);
    let c = claim::create(&key, hrp, neuron).unwrap_or_else(|e| die(e));
    field("account", &cyan(&c.address));
    field("neuron", &green(&hex(&c.neuron)));
    println!("  {} {}", dim("claim"), yellow(&c.encode()));
    println!("  {}", dim("↑ paste this into `mudra verify`"));
}

fn cmd_verify(input: &str, hrp: &str) {
    let Some(c) = Claim::decode(input) else {
        die("malformed claim (expected: address pubkey neuron signature)");
    };
    if claim::verify(&c, hrp) {
        field("account", &cyan(&c.address));
        field("neuron", &green(&hex(&c.neuron)));
        println!("  {} {}", green("✓"), bold("claim verifies"));
    } else {
        println!("  {} {}", red("✗"), bold("claim does NOT verify"));
        exit(1);
    }
}

fn cmd_new(hrp: &str) {
    let mnemonic = bip39::Mnemonic::generate(24).unwrap_or_else(|e| die(e));
    let phrase = mnemonic.to_string();
    let key = seed::cosmos_key(&phrase, "").unwrap_or_else(|e| die(e));
    let pubkey = cosmos::compressed(key.verifying_key());
    let addr = cosmos::address(&pubkey, hrp).unwrap_or_else(|e| die(e));
    field("mnemonic", &yellow(&phrase));
    field("account", &cyan(&addr));
    field("neuron", &green(&hex(&claim::neuron_of(&pubkey))));
    println!("  {}", dim("↑ write the mnemonic down — it is the only key to this account"));
}

/// `mudra`'s argv split, before any command dispatches: the `--bostrom`
/// prefix flag (stripped wherever it appears, first match wins), the
/// subcommand name, and its remaining words rejoined into one mnemonic
/// string.
struct ParsedArgs {
    bostrom: bool,
    cmd: String,
    rest: String,
}

/// Pure argv parser behind [`main`]'s dispatch, split out so the split
/// itself is testable without a real process argv.
fn parse_cli_args(args: &[String]) -> ParsedArgs {
    let mut args = args.to_vec();
    let bostrom = if let Some(i) = args.iter().position(|a| a == "--bostrom") {
        args.remove(i);
        true
    } else {
        false
    };
    let cmd = args.first().cloned().unwrap_or_default();
    let rest = args.get(1..).unwrap_or(&[]).join(" ");
    ParsedArgs { bostrom, cmd, rest }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ParsedArgs { bostrom, cmd, rest } = parse_cli_args(&args);
    let hrp = if bostrom { cosmos::BOSTROM } else { cosmos::PUSSY };

    match cmd.as_str() {
        "" | "help" | "-h" | "--help" => {
            banner();
            help();
        }
        "new" => cmd_new(hrp),
        "derive" | "address" => {
            if rest.is_empty() {
                die("usage: mudra derive <mnemonic…>");
            }
            cmd_derive(&rest, hrp);
        }
        "neuron" => {
            if rest.is_empty() {
                die("usage: mudra neuron <mnemonic…>");
            }
            cmd_neuron(&rest);
        }
        "claim" => {
            if rest.is_empty() {
                die("usage: mudra claim <mnemonic…>");
            }
            cmd_claim(&rest, hrp);
        }
        "verify" => {
            if rest.is_empty() {
                die("usage: mudra verify <claim>");
            }
            cmd_verify(&rest, hrp);
        }
        other => die(format!("unknown command: {other}  (try: mudra help)")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn hex_encodes_empty_as_empty() {
        assert_eq!(hex(&[]), "");
    }

    #[test]
    fn hex_encodes_known_bytes_lowercase_padded() {
        assert_eq!(hex(&[0x00, 0x0f, 0xff, 0xa0]), "000fffa0");
    }

    #[test]
    fn parse_cli_args_no_flag_no_command() {
        let p = parse_cli_args(&[]);
        assert!(!p.bostrom);
        assert_eq!(p.cmd, "");
        assert_eq!(p.rest, "");
    }

    #[test]
    fn parse_cli_args_command_with_multi_word_mnemonic() {
        let p = parse_cli_args(&v(&["derive", "word1", "word2", "word3"]));
        assert!(!p.bostrom);
        assert_eq!(p.cmd, "derive");
        assert_eq!(p.rest, "word1 word2 word3");
    }

    #[test]
    fn parse_cli_args_bostrom_flag_leading_is_stripped() {
        let p = parse_cli_args(&v(&["--bostrom", "derive", "word1"]));
        assert!(p.bostrom);
        assert_eq!(p.cmd, "derive");
        assert_eq!(p.rest, "word1");
    }

    #[test]
    fn parse_cli_args_bostrom_flag_trailing_is_stripped() {
        let p = parse_cli_args(&v(&["derive", "word1", "--bostrom"]));
        assert!(p.bostrom);
        assert_eq!(p.cmd, "derive");
        assert_eq!(p.rest, "word1");
    }

    #[test]
    fn parse_cli_args_bostrom_flag_mid_command_is_stripped() {
        let p = parse_cli_args(&v(&["derive", "--bostrom", "word1", "word2"]));
        assert!(p.bostrom);
        assert_eq!(p.cmd, "derive");
        assert_eq!(p.rest, "word1 word2");
    }

    #[test]
    fn parse_cli_args_only_first_bostrom_occurrence_is_removed() {
        // Documents current behavior: a second literal "--bostrom" is not a
        // flag occurrence, it becomes ordinary command/mnemonic text.
        let p = parse_cli_args(&v(&["--bostrom", "--bostrom", "word1"]));
        assert!(p.bostrom);
        assert_eq!(p.cmd, "--bostrom");
        assert_eq!(p.rest, "word1");
    }

    #[test]
    fn parse_cli_args_no_bostrom_flag_present() {
        let p = parse_cli_args(&v(&["verify", "claim-line"]));
        assert!(!p.bostrom);
        assert_eq!(p.cmd, "verify");
        assert_eq!(p.rest, "claim-line");
    }
}
