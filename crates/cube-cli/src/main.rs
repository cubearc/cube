use std::error::Error;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cube_core::{ciphers, encoding, hashing, operations};

mod audiocmd;
mod binarycmd;
mod blockchaincmd;
mod constantcmd;
mod filecmd;
mod imagecmd;
mod investigatecmd;
mod pipelinecmd;
mod puzzlecmd;
mod recipecmd;
mod textcmd;
mod videocmd;
mod webcmd;

#[derive(Parser)]
#[command(
    name = "cube",
    version,
    about = "Data transformation, cryptography, digital forensics, and investigative workflows"
)]
struct Cli {
    /// Emit failures as schema-versioned JSON on stderr (including usage errors)
    #[arg(long, global = true)]
    json_errors: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Bounded offline key formats; secret validation is separate from export
    Blockchain {
        #[command(subcommand)]
        command: blockchaincmd::Command,
    },
    /// Inspect caller-supplied offline web artifacts without network access
    Web {
        #[command(subcommand)]
        command: webcmd::WebCommand,
    },
    /// Inspect video containers and extract/compare supported JPEG frames or PCM8 tracks
    Video {
        #[command(subcommand)]
        command: videocmd::VideoCommand,
    },
    /// Local explainable triage and bounded investigations
    Investigate {
        #[command(subcommand)]
        command: investigatecmd::InvestigateCommand,
    },
    /// Validate, execute or independently test versioned JSON recipes
    Recipe {
        #[command(subcommand)]
        command: recipecmd::RecipeCommand,
    },
    /// Validate or execute bounded typed linear pipelines
    Pipeline {
        #[command(subcommand)]
        command: pipelinecmd::PipelineCommand,
    },
    /// Inspect local bounded PCM WAVE audio without a multimedia backend
    Audio {
        #[command(subcommand)]
        command: audiocmd::AudioCommand,
    },
    /// Exact integer representations, separate from byte encodings
    Number {
        #[command(subcommand)]
        command: Number,
    },
    /// Explicit Unicode scalar transformations and normalization (bounded input)
    Text {
        #[command(subcommand)]
        command: textcmd::TextCommand,
    },
    /// Inspect and transform bounded images without a graphical dependency
    Image {
        #[command(subcommand)]
        command: imagecmd::ImageCommand,
    },
    /// Inspect basic file facts and signature evidence (read-only)
    Inspect {
        path: std::path::PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Inspect files or explicitly extract selected bytes
    File {
        #[command(subcommand)]
        command: filecmd::FileCommand,
    },
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
        command: binarycmd::BinaryCommand,
    },
}

#[derive(Subcommand)]
enum Number {
    /// Select certified fractional digits with an explicit bounded prefix budget
    Digits(constantcmd::DigitsArgs),
    /// Map one exact clock reading to directions; optional caller-convention Semaphore
    #[command(group(clap::ArgGroup::new("clock_input").required(true).multiple(false).args(["time","hour_millidegrees"])))]
    Clock {
        #[arg(long,conflicts_with_all=["hour_millidegrees","minute_millidegrees"])]
        time: Option<String>,
        #[arg(long, requires = "minute_millidegrees", allow_hyphen_values = true)]
        hour_millidegrees: Option<i64>,
        #[arg(long, requires = "hour_millidegrees", allow_hyphen_values = true)]
        minute_millidegrees: Option<i64>,
        #[arg(long, default_value = "N")]
        zero_axis: String,
        #[arg(long)]
        counterclockwise: bool,
        #[arg(long)]
        decode: bool,
        #[arg(long)]
        json: bool,
    },
    /// Convert signed ASCII integers; 2..36 radices, <=4096 digits and 128-bit magnitude; no prefixes/floats
    Base {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        from_base: u32,
        #[arg(long)]
        to_base: u32,
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
    /// Explicit telephone or delimited multi-tap keypad mapping
    Keypad(puzzlecmd::KeypadArgs),
    /// Bounded single-byte-version Base58Check; <=1024 payload bytes, empty allowed
    Base58check {
        #[command(flatten)]
        input: Input,
        /// Decimal data-prefix byte 0..255, independent of network policy
        #[arg(long)]
        version_byte: u8,
    },
    /// Letter-only flag Semaphore; anatomical sender LH,RH, viewed facing sender
    Semaphore(Input),
    /// ASCII letters to 1..26 hyphen tokens, preserving ASCII whitespace
    A1z26(Input),
    /// International Morse A-Z/0-9; collapse ASCII spaces; <=1 MiB input
    Morse(Input),
    /// Raw dot-mask bytes to Unicode Braille cells (not language transcription); <=1 MiB
    Braille(Input),
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
    /// Return canonical multi-tap text or telephone alternatives as JSON
    Keypad(puzzlecmd::KeypadArgs),
    /// Verify Base58Check and report its version plus exact payload hex; <=1421 text bytes
    Base58check {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        json: bool,
    },
    /// Exact LH,RH direction pairs or / spaces; no numeral/control state
    Semaphore(Input),
    /// Strict 1..26 hyphen tokens to uppercase text; preserve ASCII whitespace
    A1z26(Input),
    /// Canonical Morse tokens (single spaces, / between words) to uppercase text
    Morse(Input),
    /// Unicode Braille cells to exact dot-mask bytes; hex by default, --output raw for binary
    Braille(DecodeArgs),
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
    /// Five-symbol Bacon encoding with explicit alphabet and notation
    Bacon(puzzlecmd::BaconArgs),
    /// Shared Polybius/Tap square, explicit row,column pairs and word tokens
    Polybius(puzzlecmd::PolybiusArgs),
    /// Extract Unicode scalars from caller-supplied book coordinates; source <=1 MiB
    Book {
        #[command(flatten)]
        input: Input,
        /// Exact local UTF-8 source; never fetched or modified
        #[arg(long)]
        source: std::path::PathBuf,
        /// word_character, line_character or line_word_character
        #[arg(long, default_value = "word_character")]
        scheme: String,
        #[arg(long, default_value_t = 1)]
        index_base: u8,
    },
    /// Reverse ASCII alphabets, preserving other Unicode; <=1 MiB, not secure encryption
    Atbash(Input),
    /// Vigenère; repeat ASCII key on ASCII letters only, preserve other Unicode; <=1 MiB
    Vigenere(KeyCipherArgs),
    /// Standard reciprocal Beaufort C=(K-P) mod 26; <=1 MiB, not secure encryption
    Beaufort(BeaufortArgs),
    /// Affine C=(aP+b) mod 26; preserve other Unicode; <=1 MiB, not secure encryption
    Affine(AffineArgs),
    /// Rotate ASCII letters by a signed shift (not secure encryption)
    Caesar(CaesarArgs),
    /// Rotate ASCII letters by 13, leaving non-ASCII characters unchanged
    Rot13(Input),
}

#[derive(Subcommand)]
enum Hash {
    /// RIPEMD-160 for protocol compatibility; exact bytes, <=16 MiB
    Ripemd160(Input),
    /// Ethereum Keccak-256, distinct from SHA3-256; exact bytes, <=16 MiB
    Keccak256(Input),
    /// Compute SHA-512 as 128 lowercase hex digits; input <=16 MiB
    Sha512(Input),
    /// Legacy SHA-1; unsuitable for collision-resistant security; input <=16 MiB
    Sha1(Input),
    /// Legacy MD5; unsuitable for collision-resistant security; input <=16 MiB
    Md5(Input),
    /// CRC-32/ISCSI (CRC32C) checksum; not cryptographic; input <=16 MiB
    Crc32c(Input),
    /// CRC-16/IBM-3740 checksum; not cryptographic; input <=16 MiB
    Crc16Ibm3740(Input),
    /// Compute CRC-32/ISO-HDLC (IEEE) as eight lowercase hex digits; not cryptographic
    Crc32(Input),
    /// Compute SHA-256 as lowercase hexadecimal
    Sha256(Input),
}

#[derive(Args)]
struct Input {
    /// Use UTF-8 text instead of stdin; stdin bytes are read exactly, including newlines
    #[arg(long, value_name = "TEXT")]
    text: Option<String>,
}

impl Input {
    fn read_bounded(
        self,
        limit: usize,
        operation: &'static str,
    ) -> Result<Vec<u8>, cube_core::errors::OperationError> {
        use cube_core::errors::{ErrorCode, OperationError};
        let bytes = match self.text {
            Some(text) => text.into_bytes(),
            None => {
                let mut bytes = Vec::new();
                io::stdin()
                    .lock()
                    .take(limit as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| OperationError::io(e).operation(operation))?;
                bytes
            }
        };
        if bytes.len() > limit {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "input exceeds documented operation byte limit",
            )
            .operation(operation));
        }
        Ok(bytes)
    }
    fn read_text(
        self,
        operation: &'static str,
    ) -> Result<String, cube_core::errors::OperationError> {
        self.read_text_limit(cube_core::text::MAX_INPUT, operation)
    }
    fn read_text_limit(
        self,
        limit: usize,
        operation: &'static str,
    ) -> Result<String, cube_core::errors::OperationError> {
        let bytes = self.read_bounded(limit, operation)?;
        String::from_utf8(bytes).map_err(|e| {
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidInput,
                "input must be valid UTF-8",
            )
            .operation(operation)
            .offset(e.utf8_error().valid_up_to() as u64)
            .caused_by(e)
        })
    }
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
struct KeyCipherArgs {
    #[command(flatten)]
    input: Input,
    /// Nonempty ASCII-letter key, at most 64 KiB
    #[arg(long)]
    key: String,
    #[arg(long)]
    decode: bool,
}
#[derive(Args)]
struct BeaufortArgs {
    #[command(flatten)]
    input: Input,
    /// Nonempty ASCII-letter key, at most 64 KiB; apply twice to decode
    #[arg(long)]
    key: String,
}
#[derive(Args)]
struct AffineArgs {
    #[command(flatten)]
    input: Input,
    #[arg(long, allow_negative_numbers = true)]
    a: i32,
    #[arg(long, allow_negative_numbers = true)]
    b: i32,
    #[arg(long)]
    decode: bool,
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
        Command::Blockchain { command } => blockchaincmd::run(command, &mut output)?,
        Command::Number {
            command: Number::Digits(args),
        } => constantcmd::run(args, &mut output)?,
        Command::Number {
            command:
                Number::Clock {
                    time,
                    hour_millidegrees,
                    minute_millidegrees,
                    zero_axis,
                    counterclockwise,
                    decode,
                    json,
                },
        } => {
            let report = cube_core::symbols::clock::inspect(
                time.as_deref(),
                hour_millidegrees,
                minute_millidegrees,
                &zero_axis,
                !counterclockwise,
                decode,
            )?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                writeln!(
                    output,
                    "Hour: {}{}; minute: {}{}",
                    report.hour.directions.join("|"),
                    if report.hour.ambiguous {
                        " (ambiguous)"
                    } else {
                        ""
                    },
                    report.minute.directions.join("|"),
                    if report.minute.ambiguous {
                        " (ambiguous)"
                    } else {
                        ""
                    }
                )?;
                if let Some(letter) = report.decoded {
                    writeln!(output, "Semaphore (hour=LH, minute=RH): {letter}")?;
                }
            }
        }
        Command::Web { command } => webcmd::run(command, &mut output)?,
        Command::Video { command } => videocmd::run(command, &mut output)?,
        Command::Investigate { command } => investigatecmd::run(command, &mut output)?,
        Command::Pipeline { command } => pipelinecmd::run(command, &mut output)?,
        Command::Recipe { command } => recipecmd::run(command, &mut output)?,
        Command::Audio { command } => audiocmd::run(command, &mut output)?,
        Command::Number {
            command:
                Number::Base {
                    input,
                    from_base,
                    to_base,
                },
        } => write_text(
            &mut output,
            &cube_core::numbers::convert_base(
                &input
                    .read_text_limit(cube_core::numbers::MAX_DIGITS + 1, "number.base.convert")?,
                from_base,
                to_base,
            )?,
        )?,
        Command::Text { command } => textcmd::run(command, &mut output)?,
        Command::Image { command } => imagecmd::run(command, &mut output)?,
        Command::Inspect { path, json } => filecmd::info(path, json, &mut output)?,
        Command::File { command } => filecmd::run(command, &mut output)?,
        Command::Ops { command } => match command {
            Ops::List { category, json } => {
                let mut selected = operations::filtered(category.as_deref())?;
                if json {
                    selected.sort_by_key(|d| d.id);
                    write_text(
                        &mut output,
                        &serde_json::to_string_pretty(
                            &serde_json::json!({"schema_version": 1, "operations": selected.iter().map(|d| d.discovery()).collect::<Vec<_>>()}),
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
                let descriptor = operations::require(&id)?;
                if json {
                    write_text(
                        &mut output,
                        &serde_json::to_string_pretty(
                            &serde_json::json!({"schema_version": 1, "operation": descriptor.discovery()}),
                        )?,
                    )?;
                } else {
                    write_text(
                        &mut output,
                        &format!(
                            "{} — {}\nCategory: {}\nInput: {}\nOutput: {}\nDeterministic: {}\nReversible: {}\nPipeline: {}\nExample: {}",
                            descriptor.id,
                            descriptor.description,
                            descriptor.category,
                            descriptor.input_type,
                            descriptor.output_type,
                            descriptor.deterministic,
                            descriptor
                                .reversible
                                .map_or("not applicable", |value| if value { "yes" } else { "no" }),
                            descriptor.discovery().pipeline.map_or_else(
                                || "unavailable (no bounded adapter)".to_owned(),
                                |support| format!("adapter version {}", support.version),
                            ),
                            descriptor.example
                        ),
                    )?;
                    for parameter in descriptor.parameters {
                        write_text(
                            &mut output,
                            &format!(
                                "{}: {} (required: {}, constraints: {}, default: {})",
                                parameter.name,
                                parameter.description,
                                parameter.required,
                                parameter.constraints,
                                parameter
                                    .default
                                    .map(|value| value.to_string())
                                    .unwrap_or_else(|| "none".into())
                            ),
                        )?;
                    }
                }
            }
        },
        Command::Encode { command } => {
            let text = match command {
                Encode::Base58check {
                    input,
                    version_byte,
                } => encoding::base58check::encode(
                    version_byte,
                    &input.read_bounded(
                        encoding::base58check::MAX_PAYLOAD,
                        "encoding.base58check.encode",
                    )?,
                )?,
                Encode::Keypad(args) => puzzlecmd::keypad(args, false)?,
                Encode::Semaphore(input) => cube_core::symbols::semaphore::encode(
                    &input.read_text("symbol.semaphore.encode")?,
                )?,
                Encode::A1z26(input) => {
                    cube_core::symbols::a1z26::encode(&input.read_text("symbol.a1z26.encode")?)?
                }
                Encode::Morse(input) => {
                    cube_core::symbols::morse_encode(&input.read_text("symbol.morse.encode")?)?
                }
                Encode::Braille(input) => cube_core::symbols::braille_encode(
                    &input.read_bounded(cube_core::text::MAX_INPUT, "symbol.braille.encode")?,
                )?,
                Encode::Base32(input) => encoding::base32_encode(&input.read()?),
                Encode::Base58(input) => encoding::base58_encode(&input.read()?),
                Encode::Base64(input) => encoding::base64_encode(&input.read()?),
                Encode::Hex(input) => encoding::hex_encode(&input.read()?),
            };
            write_text(&mut output, &text)?;
        }
        Command::Decode { command } => match command {
            Decode::Base58check { input, json } => {
                let decoded = encoding::base58check::decode(&input.read_text_limit(
                    encoding::base58check::MAX_ENCODED,
                    "encoding.base58check.decode",
                )?)?;
                if json {
                    writeln!(output, "{}", serde_json::to_string_pretty(&decoded)?)?;
                } else {
                    writeln!(
                        output,
                        "Version: 0x{:02x}\nPayload (hex): {}\nChecksum verified: true",
                        decoded.version,
                        encoding::hex_encode(&decoded.payload)
                    )?;
                }
            }
            Decode::Keypad(args) => write_text(&mut output, &puzzlecmd::keypad(args, true)?)?,
            Decode::Semaphore(input) => write_text(
                &mut output,
                &cube_core::symbols::semaphore::decode(
                    &input.read_text("symbol.semaphore.decode")?,
                )?,
            )?,
            Decode::A1z26(input) => write_text(
                &mut output,
                &cube_core::symbols::a1z26::decode(&input.read_text_limit(
                    cube_core::symbols::a1z26::MAX_ENCODED,
                    "symbol.a1z26.decode",
                )?)?,
            )?,
            Decode::Morse(input) => write_text(
                &mut output,
                &cube_core::symbols::morse_decode(&input.read_text("symbol.morse.decode")?)?,
            )?,
            Decode::Braille(args) => write_bytes(
                &mut output,
                &cube_core::symbols::braille_decode(
                    &args.input.read_text("symbol.braille.decode")?,
                )?,
                args.output,
            )?,
            Decode::Base32(args) => write_bytes(
                &mut output,
                &encoding::base32_decode(&String::from_utf8(args.input.read()?)?)?,
                args.output,
            )?,
            Decode::Base58(args) => write_bytes(
                &mut output,
                &encoding::base58_decode(&String::from_utf8(args.input.read()?)?)?,
                args.output,
            )?,
            Decode::Base64(args) => write_bytes(
                &mut output,
                &encoding::base64_decode(&String::from_utf8(args.input.read()?)?)?,
                args.output,
            )?,
            Decode::Hex(args) => write_bytes(
                &mut output,
                &encoding::hex_decode(&String::from_utf8(args.input.read()?)?)?,
                args.output,
            )?,
        },
        Command::Cipher { command } => {
            let text = match command {
                Cipher::Bacon(args) => puzzlecmd::bacon(args)?,
                Cipher::Polybius(args) => puzzlecmd::polybius(args)?,
                Cipher::Book {
                    input,
                    source,
                    scheme,
                    index_base,
                } => {
                    let source = pipelinecmd::read_file(&source, cube_core::text::MAX_INPUT)?;
                    let source = std::str::from_utf8(&source).map_err(|_| {
                        cube_core::errors::OperationError::new(
                            cube_core::errors::ErrorCode::InvalidInput,
                            "source must be valid UTF-8",
                        )
                        .operation("cipher.book.decode")
                        .parameter("source")
                    })?;
                    ciphers::book::decode(
                        source,
                        &input.read_text("cipher.book.decode")?,
                        ciphers::book::Scheme::parse(&scheme)?,
                        index_base,
                    )?
                }
                Cipher::Atbash(input) => ciphers::atbash(&input.read_text("cipher.atbash")?)?,
                Cipher::Vigenere(args) => {
                    let op = if args.decode {
                        "cipher.vigenere.decode"
                    } else {
                        "cipher.vigenere.encode"
                    };
                    let input = args.input.read_text(op)?;
                    if args.decode {
                        ciphers::vigenere_decode(&input, &args.key)?
                    } else {
                        ciphers::vigenere_encode(&input, &args.key)?
                    }
                }
                Cipher::Beaufort(args) => {
                    ciphers::beaufort(&args.input.read_text("cipher.beaufort")?, &args.key)?
                }
                Cipher::Affine(args) => {
                    let op = if args.decode {
                        "cipher.affine.decode"
                    } else {
                        "cipher.affine.encode"
                    };
                    let input = args.input.read_text(op)?;
                    if args.decode {
                        ciphers::affine_decode(&input, args.a, args.b)?
                    } else {
                        ciphers::affine_encode(&input, args.a, args.b)?
                    }
                }
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
                Hash::Keccak256(input) => hashing::keccak256(
                    &input.read_bounded(hashing::protocol::MAX_INPUT, "hash.keccak256")?,
                )?,
                Hash::Ripemd160(input) => hashing::ripemd160(
                    &input
                        .read_bounded(cube_core::hashing::protocol::MAX_INPUT, "hash.ripemd160")?,
                )?,
                Hash::Sha512(input) => hashing::sha512(
                    &input.read_bounded(cube_core::binary::transform::MAX_INPUT, "hash.sha512")?,
                ),
                Hash::Sha1(input) => hashing::sha1(
                    &input.read_bounded(cube_core::binary::transform::MAX_INPUT, "hash.sha1")?,
                ),
                Hash::Md5(input) => hashing::md5(
                    &input.read_bounded(cube_core::binary::transform::MAX_INPUT, "hash.md5")?,
                ),
                Hash::Crc32c(input) => hashing::crc32c(
                    &input.read_bounded(cube_core::binary::transform::MAX_INPUT, "hash.crc32c")?,
                ),
                Hash::Crc16Ibm3740(input) => hashing::crc16_ibm3740(&input.read_bounded(
                    cube_core::binary::transform::MAX_INPUT,
                    "hash.crc16_ibm3740",
                )?),
                Hash::Crc32(input) => hashing::crc32(&input.read()?),
                Hash::Sha256(input) => hashing::sha256(&input.read()?),
            };
            write_text(&mut output, &digest)?;
        }
        Command::Binary { command } => binarycmd::run(command, &mut output)?,
    }
    output.flush()?;
    Ok(())
}

fn main() -> ExitCode {
    let json_errors = std::env::args_os().any(|argument| argument == "--json-errors");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if json_errors && error.use_stderr() {
                eprintln!(
                    "{}",
                    serde_json::json!({"schema_version": 1, "error": {"code": "invalid_usage", "message": "invalid command usage; use --help", "operation": null, "parameter": null, "offset": null}})
                );
            } else if let Err(write_error) = error.print() {
                return if write_error.kind() == io::ErrorKind::BrokenPipe {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                };
            }
            return ExitCode::from(error.exit_code() as u8);
        }
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut cause: Option<&(dyn Error + 'static)> = Some(error.as_ref());
            let mut broken_pipe = false;
            while let Some(error) = cause {
                broken_pipe |= error
                    .downcast_ref::<io::Error>()
                    .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe);
                cause = error.source();
            }
            if broken_pipe {
                return ExitCode::SUCCESS;
            }
            if json_errors {
                let diagnostic = if let Some(error) =
                    error.downcast_ref::<cube_core::pipeline::Error>()
                {
                    error.diagnostic()
                } else if let Some(error) =
                    error.downcast_ref::<cube_core::errors::OperationError>()
                {
                    serde_json::json!(error)
                } else if let Some(error) = error.downcast_ref::<cube_core::Error>() {
                    serde_json::json!({"code": error.code(), "message": error.to_string(), "operation": error.operation_id(), "parameter": null, "offset": null})
                } else {
                    let code = if let Some(error) = error.downcast_ref::<io::Error>() {
                        if error.kind() == io::ErrorKind::InvalidInput {
                            "invalid_parameter"
                        } else {
                            "io"
                        }
                    } else {
                        "invalid_input"
                    };
                    serde_json::json!({"code": code, "message": error.to_string(), "operation": null, "parameter": null, "offset": null})
                };
                eprintln!(
                    "{}",
                    serde_json::json!({"schema_version": 1, "error": diagnostic})
                );
            } else {
                eprintln!("error: {error}");
            }
            ExitCode::FAILURE
        }
    }
}
