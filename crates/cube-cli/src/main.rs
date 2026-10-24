use std::error::Error;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cube_core::{binary, ciphers, encoding, hashing, operations};

#[derive(Parser)]
#[command(
    name = "cube",
    version,
    about = "Encoding, classical ciphers, hashing, and binary tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Discover available operations
    Ops {
        #[command(subcommand)]
        command: Ops,
    },
    /// Encode bytes as text
    Encode {
        #[command(subcommand)]
        command: Encode,
    },
    /// Decode text into bytes (hex output by default)
    Decode {
        #[command(subcommand)]
        command: Decode,
    },
    /// Apply a classical text cipher
    Cipher {
        #[command(subcommand)]
        command: Cipher,
    },
    /// Compute a cryptographic digest or non-cryptographic checksum of bytes
    Hash {
        #[command(subcommand)]
        command: Hash,
    },
    /// Transform arbitrary bytes
    Binary {
        #[command(subcommand)]
        command: Binary,
    },
}

#[derive(Subcommand)]
enum Ops {
    /// List supported commands, one per line
    List {
        /// Select an exact operation category
        #[arg(long)]
        category: Option<String>,
        /// Emit schema-versioned JSON sorted by stable operation ID
        #[arg(long)]
        json: bool,
    },
    /// Describe an exact stable operation ID
    Describe {
        id: String,
        /// Emit a schema-versioned JSON description
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum Encode {
    /// Encode as padded RFC 4648 standard Base32 with uppercase letters
    Base32(Input),
    /// Encode as Bitcoin Base58, preserving leading zero bytes as 1
    Base58(Input),
    /// Encode as padded RFC 4648 standard Base64
    Base64(Input),
    /// Encode as lowercase hexadecimal
    Hex(Input),
}

#[derive(Subcommand)]
enum Decode {
    /// Decode strict uppercase standard Base32; padding is required when needed
    Base32(DecodeArgs),
    /// Decode case-sensitive Bitcoin Base58; whitespace and padding are rejected
    Base58(DecodeArgs),
    /// Decode strict, padded standard Base64; whitespace is rejected
    Base64(DecodeArgs),
    /// Decode hexadecimal digits; prefixes and whitespace are rejected
    Hex(DecodeArgs),
}

#[derive(Subcommand)]
enum Cipher {
    /// Rotate ASCII letters by a signed shift (not secure encryption)
    Caesar(CaesarArgs),
    /// Rotate ASCII letters by 13, leaving non-ASCII characters unchanged
    Rot13(Input),
}

#[derive(Subcommand)]
enum Hash {
    /// Compute CRC-32/ISO-HDLC (IEEE) as eight lowercase hex digits; not cryptographic
    Crc32(Input),
    /// Compute SHA-256 as lowercase hexadecimal
    Sha256(Input),
}

#[derive(Subcommand)]
enum Binary {
    /// XOR bytes with a repeating key (not secure encryption)
    Xor(XorArgs),
}

#[derive(Args)]
struct Input {
    /// Use UTF-8 text instead of stdin; stdin bytes are read exactly, including newlines
    #[arg(long, value_name = "TEXT")]
    text: Option<String>,
}

impl Input {
    fn read(self) -> io::Result<Vec<u8>> {
        match self.text {
            Some(text) => Ok(text.into_bytes()),
            None => {
                let mut bytes = Vec::new();
                io::stdin().lock().read_to_end(&mut bytes)?;
                Ok(bytes)
            }
        }
    }
}

#[derive(Args)]
struct DecodeArgs {
    #[command(flatten)]
    input: Input,
    /// Binary output representation: hex adds a newline; raw writes exact bytes
    #[arg(long, value_enum, default_value_t = Output::Hex)]
    output: Output,
}

#[derive(Args)]
struct CaesarArgs {
    #[command(flatten)]
    input: Input,
    /// Signed shift from -2147483648 to 2147483647, normalized modulo 26
    #[arg(long, allow_negative_numbers = true)]
    shift: i32,
    /// Apply the inverse of --shift to decode; otherwise encode
    #[arg(long)]
    decode: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false, args = ["key", "key_hex"])]
struct XorArgs {
    #[command(flatten)]
    input: Input,
    /// Nonempty UTF-8 key
    #[arg(long)]
    key: Option<String>,
    /// Nonempty key as hexadecimal digits
    #[arg(long, value_name = "HEX")]
    key_hex: Option<String>,
    /// Binary output representation: hex adds a newline; raw writes exact bytes
    #[arg(long, value_enum, default_value_t = Output::Hex)]
    output: Output,
}

#[derive(Clone, Copy, ValueEnum)]
enum Output {
    Hex,
    Raw,
}

fn write_text(output: &mut impl Write, text: &str) -> io::Result<()> {
    writeln!(output, "{text}")
}

fn write_bytes(output: &mut impl Write, bytes: &[u8], format: Output) -> io::Result<()> {
    match format {
        Output::Hex => write_text(output, &encoding::hex_encode(bytes)),
        Output::Raw => output.write_all(bytes),
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    match cli.command {
        Command::Ops { command } => match command {
            Ops::List { category, json } => {
                let catalog = operations::catalog();
                if let Some(category) = &category
                    && !catalog.iter().any(|d| d.category == category)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "unknown operation category; use cube ops list --json",
                    )
                    .into());
                }
                let mut selected: Vec<_> = catalog
                    .into_iter()
                    .filter(|d| category.as_ref().is_none_or(|c| d.category == c))
                    .collect();
                if json {
                    selected.sort_by_key(|d| d.id);
                    write_text(
                        &mut output,
                        &serde_json::to_string_pretty(
                            &serde_json::json!({"schema_version": 1, "operations": selected}),
                        )?,
                    )?;
                } else {
                    let mut seen = std::collections::HashSet::new();
                    for descriptor in selected {
                        if seen.insert(descriptor.command) {
                            write_text(&mut output, descriptor.command)?;
                        }
                    }
                }
            }
            Ops::Describe { id, json } => {
                let descriptor = operations::describe(&id).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "unknown operation ID; use cube ops list --json",
                    )
                })?;
                if json {
                    write_text(
                        &mut output,
                        &serde_json::to_string_pretty(
                            &serde_json::json!({"schema_version": 1, "operation": descriptor}),
                        )?,
                    )?;
                } else {
                    write_text(
                        &mut output,
                        &format!(
                            "{} — {}\nCategory: {}\nInput: {}\nOutput: {}\nDeterministic: {}\nReversible: {:?}\nExample: {}",
                            descriptor.id,
                            descriptor.description,
                            descriptor.category,
                            descriptor.input_type,
                            descriptor.output_type,
                            descriptor.deterministic,
                            descriptor.reversible,
                            descriptor.example
                        ),
                    )?;
                    for parameter in descriptor.parameters {
                        write_text(
                            &mut output,
                            &format!(
                                "{}: {} (required: {}, constraints: {:?}, default: {:?})",
                                parameter.name,
                                parameter.description,
                                parameter.required,
                                parameter.constraints,
                                parameter.default
                            ),
                        )?;
                    }
                }
            }
        },
        Command::Encode { command } => {
            let text = match command {
                Encode::Base32(input) => encoding::base32_encode(&input.read()?),
                Encode::Base58(input) => encoding::base58_encode(&input.read()?),
                Encode::Base64(input) => encoding::base64_encode(&input.read()?),
                Encode::Hex(input) => encoding::hex_encode(&input.read()?),
            };
            write_text(&mut output, &text)?;
        }
        Command::Decode { command } => {
            let (bytes, format) = match command {
                Decode::Base32(args) => (
                    encoding::base32_decode(&String::from_utf8(args.input.read()?)?)?,
                    args.output,
                ),
                Decode::Base58(args) => (
                    encoding::base58_decode(&String::from_utf8(args.input.read()?)?)?,
                    args.output,
                ),
                Decode::Base64(args) => (
                    encoding::base64_decode(&String::from_utf8(args.input.read()?)?)?,
                    args.output,
                ),
                Decode::Hex(args) => (
                    encoding::hex_decode(&String::from_utf8(args.input.read()?)?)?,
                    args.output,
                ),
            };
            write_bytes(&mut output, &bytes, format)?;
        }
        Command::Cipher { command } => {
            let text = match command {
                Cipher::Caesar(args) => {
                    let input = String::from_utf8(args.input.read()?)?;
                    if args.decode {
                        ciphers::caesar_decode(&input, args.shift)
                    } else {
                        ciphers::caesar_encode(&input, args.shift)
                    }
                }
                Cipher::Rot13(input) => ciphers::rot13(&String::from_utf8(input.read()?)?),
            };
            write_text(&mut output, &text)?;
        }
        Command::Hash { command } => {
            let digest = match command {
                Hash::Crc32(input) => hashing::crc32(&input.read()?),
                Hash::Sha256(input) => hashing::sha256(&input.read()?),
            };
            write_text(&mut output, &digest)?;
        }
        Command::Binary {
            command: Binary::Xor(args),
        } => {
            let key = match (args.key, args.key_hex) {
                (Some(key), None) => key.into_bytes(),
                (None, Some(key)) => encoding::hex_decode(&key)?,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provide exactly one of --key or --key-hex",
                    )
                    .into());
                }
            };
            let bytes = binary::xor(&args.input.read()?, &key)?;
            write_bytes(&mut output, &bytes, args.output)?;
        }
    }
    output.flush()?;
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe)
            {
                return ExitCode::SUCCESS;
            }
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
