use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use cube_core::{ciphers, encoding, hashing, operations};

mod alphabetcmd;
mod audiocmd;
mod binarycmd;
mod blockchaincmd;
mod capture;
mod combinatoricscmd;
mod composecmd;
mod constantcmd;
mod digitalcmd;
mod esolangcmd;
mod filecmd;
mod graphcmd;
mod imagecmd;
mod input;
mod investigatecmd;
mod lexicalcmd;
mod logiccmd;
mod multisetcmd;
mod probabilitycmd;
use input::Input;
mod aescmd;
mod cryptanalysiscmd;
mod enigmacmd;
mod fractioncmd;
mod historicalcmd;
mod modularcmd;
mod monocmd;
mod pipelinecmd;
mod playcmd;
mod polycmd;
mod puzzlecmd;
mod puzzlecodescmd;
mod recipecmd;
mod streamcmd;
mod textcmd;
mod traversalcmd;
mod unicodecmd;
mod valuescmd;
mod videocmd;
mod webcmd;
mod wordcmd;

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
    /// Save a bounded typed result; omit PATH only inside a configured Play Runner
    #[arg(long, global = true, num_args = 0..=1, require_equals = true, value_name = "PATH")]
    lab_capture: Option<Option<std::path::PathBuf>>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compose explicitly ordered text or byte operands with bounded Core operations
    Compose {
        #[command(subcommand)]
        command: composecmd::Command,
    },
    /// Bounded classical-cipher evidence using caller-owned models
    Cryptanalysis {
        #[command(subcommand)]
        command: cryptanalysiscmd::Command,
    },
    /// Exact finite probability and independent dice models (JSON)
    Probability {
        #[command(subcommand)]
        command: probabilitycmd::Command,
    },
    /// Exact bounded caller-supplied multigraph queries
    Graph {
        #[command(subcommand)]
        command: graphcmd::Command,
    },
    /// Explicit bounded propositional formulas and truth tables
    Logic {
        #[command(subcommand)]
        command: logiccmd::Command,
    },
    /// Execute bounded languages without host capabilities
    Esolang {
        #[command(subcommand)]
        command: esolangcmd::Command,
    },
    /// Open Cube Play or control an attached native session
    Play(playcmd::PlayArgs),
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
    /// Apply historical ciphers or unauthenticated AES and stream transforms
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
    /// Exact ordered k-selections, n<=128, u128 result
    Permutations(combinatoricscmd::Pair),
    /// Exact C(n,k), cancellation avoids factorial overflow
    Binomial(combinatoricscmd::Pair),
    /// Exact count from explicit group sizes, total<=128
    Multinomial(combinatoricscmd::Groups),
    /// Bounded Pascal triangle, exact decimal JSON strings
    Pascal(combinatoricscmd::Pascal),
    /// Decimal digital root of a signed 128-bit magnitude
    DigitalRoot(digitalcmd::Args),
    /// Exact u64 GCD, including gcd(0,0)=0
    Gcd(modularcmd::Pair),
    /// Deterministic GCD and signed Bezout coefficients
    ExtendedGcd(modularcmd::Pair),
    /// Canonical inverse for coprime u64 operands, modulus >=2
    ModInverse(modularcmd::Inverse),
    /// Exact u64 modular power, <=64 exponent-bit rounds
    ModPow(modularcmd::Power),
    /// Exact integers over explicit ordered Unicode scalar alphabets
    Alphabet(alphabetcmd::Args),
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
        /// Complete discovery output ceiling, including JSON envelope/whitespace
        #[arg(long)]
        max_output_bytes: Option<usize>,
        /// Opt in to verified canonical names, aliases, safety and profile facts
        #[arg(long, requires = "json")]
        metadata: bool,
        /// Include complete versioned input/parameter/output/error schemas
        #[arg(long, requires = "json")]
        contracts: bool,
    },
    /// Describe an exact stable operation ID
    Describe {
        id: String,
        /// Emit a schema-versioned JSON description
        #[arg(long)]
        json: bool,
        /// Complete discovery output ceiling, including JSON envelope/whitespace
        #[arg(long)]
        max_output_bytes: Option<usize>,
        /// Opt in to verified canonical names, aliases, safety and profile facts
        #[arg(long, requires = "json")]
        metadata: bool,
        /// Include complete versioned input/parameter/output/error schemas
        #[arg(long, requires = "json")]
        contracts: bool,
    },
}

#[derive(Subcommand)]
enum Encode {
    /// Exact caller-owned symbolic token table; no font or OCR
    Tokens(puzzlecodescmd::Parameters),
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
    /// Exact caller-owned symbolic token table; preserve aliases/positions
    Tokens(puzzlecodescmd::Parameters),
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
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "chacha8")]
    ChaCha8(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "chacha12")]
    ChaCha12(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "chacha20")]
    ChaCha20(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "xchacha20")]
    XChaCha20(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "salsa20-12")]
    Salsa12(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "salsa20")]
    Salsa20(streamcmd::Parameters),
    /// Exact private raw transform; requires explicit key file and destination
    #[command(name = "vernam")]
    Vernam(streamcmd::Parameters),

    /// Deterministic inverse book coordinates with exact caller source provenance
    BookEncode(puzzlecodescmd::Book),
    /// Qualified cadenus profile; explicit bounded JSON parameters
    Cadenus(polycmd::Parameters),
    /// Qualified swagman profile; explicit bounded JSON parameters
    Swagman(polycmd::Parameters),
    /// Qualified nihilist transposition profile; explicit bounded JSON parameters
    NihilistTransposition(polycmd::Parameters),
    /// Qualified ubchi profile; explicit bounded JSON parameters
    Ubchi(polycmd::Parameters),
    /// Qualified cm bifid profile; explicit bounded JSON parameters
    CmBifid(polycmd::Parameters),
    /// Qualified vic profile; explicit bounded JSON parameters
    Vic(polycmd::Parameters),
    /// Explicit bounded scytale profile; JSON parameters required
    Scytale(polycmd::Parameters),
    /// Explicit bounded rail fence profile; JSON parameters required
    RailFence(polycmd::Parameters),
    /// Explicit bounded redefence profile; JSON parameters required
    Redefence(polycmd::Parameters),
    /// Explicit bounded sequence transposition profile; JSON parameters required
    SequenceTransposition(polycmd::Parameters),
    /// Explicit bounded skip profile; JSON parameters required
    Skip(polycmd::Parameters),
    /// Explicit bounded columnar profile; JSON parameters required
    Columnar(polycmd::Parameters),
    /// Explicit bounded complete columnar profile; JSON parameters required
    CompleteColumnar(polycmd::Parameters),
    /// Explicit bounded double columnar profile; JSON parameters required
    DoubleColumnar(polycmd::Parameters),
    /// Explicit bounded myszkowski profile; JSON parameters required
    Myszkowski(polycmd::Parameters),
    /// Explicit bounded amsco profile; JSON parameters required
    Amsco(polycmd::Parameters),
    /// Explicit bounded route profile; JSON parameters required
    Route(polycmd::Parameters),
    /// Explicit bounded spiral profile; JSON parameters required
    Spiral(polycmd::Parameters),
    /// Explicit bounded caesar box profile; JSON parameters required
    CaesarBox(polycmd::Parameters),
    /// Explicit bounded turning grille profile; JSON parameters required
    TurningGrille(polycmd::Parameters),
    /// Explicit bounded bifid profile; JSON parameters required
    Bifid(polycmd::Parameters),
    /// Explicit bounded trifid profile; JSON parameters required
    Trifid(polycmd::Parameters),
    /// Explicit bounded adfgx profile; JSON parameters required
    Adfgx(polycmd::Parameters),
    /// Explicit bounded adfgvx profile; JSON parameters required
    Adfgvx(polycmd::Parameters),
    /// Explicit bounded nihilist substitution profile; JSON parameters required
    NihilistSubstitution(polycmd::Parameters),
    /// Explicit bounded bazeries profile; JSON parameters required
    Bazeries(polycmd::Parameters),
    /// Explicit bounded fractionated morse profile; JSON parameters required
    FractionatedMorse(polycmd::Parameters),
    /// Explicit bounded morbit profile; JSON parameters required
    Morbit(polycmd::Parameters),
    /// Explicit bounded pollux profile; JSON parameters required
    Pollux(polycmd::Parameters),
    /// Static caller-owned Cardan cover extract
    CardanExtract(polycmd::Parameters),
    /// Static caller-owned Cardan cover overlay
    CardanOverlay(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Gronsfeld(polycmd::Parameters),
    /// Vigenere with explicit ASCII-letter reset positions
    InterruptedKey(polycmd::Parameters),
    /// Vigenere with a signed increment per key-length group
    ProgressiveKey(polycmd::Parameters),
    /// Qualified rs-58356/v1 Vigenere then multiplicative composition
    VigenereMultiplicative(polycmd::Parameters),
    /// Historical Enigma I/M3; reciprocal transform with final windows and optional trace
    Enigma(enigmacmd::Parameters),
    /// Hazardous raw AES blocks; key file only, no padding or authentication
    Aes {
        #[command(subcommand)]
        command: aescmd::Command,
    },
    /// Explicit bounded historical profile; typed JSON parameters
    Trithemius(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Autokey(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    RunningKey(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Porta(polycmd::Parameters),
    /// Bellaso's fixed reciprocal table of 1553; canonical 22-letter text/key
    Bellaso(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Gromark(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    PeriodicGromark(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Solitaire(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Slidefair(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Portax(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Ragbaby(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Condi(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Nicodemus(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Quagmire(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Phillips(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Chaocipher(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Playfair(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    SeriatedPlayfair(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    TwoSquare(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    FourSquare(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Hill(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    Digrafid(polycmd::Parameters),
    /// Explicit bounded historical profile; typed JSON parameters
    TriSquare(polycmd::Parameters),

    /// Exact Unicode alphabet or keyword permutation
    Substitution(monocmd::Substitution),
    /// Deterministic caller-owned homophonic token mapping
    Homophonic(monocmd::Homophonic),
    /// Explicit 10-column digit grammar with two row prefixes
    Straddling(monocmd::TableArgs),
    /// Explicit ACA 3x8 digit grammar and canonical aliases
    MonomeDinome(monocmd::TableArgs),
    /// Lossy ACA digit mapping; decode returns alternatives
    Tridigital(monocmd::TableArgs),
    /// Repeated-letter square with explicit coordinate labels
    Grandpre(monocmd::TableArgs),
    /// Rotate ASCII digits by five, preserving all other scalars
    Rot5(Input),
    /// Compose ASCII letter ROT13 with digit ROT5
    Rot18(Input),
    /// Rotate printable ASCII ! through ~ by 47
    Rot47(Input),
    /// Apply the fixed self-inverse ROT8000 BMP mapping; preserve non-BMP text
    Rot8000(Input),
    /// Apply the reciprocal ten-letter written Malespin profile
    Malespin(Input),
    /// Apply caller-defined reciprocal scalar pairs
    Vatsyayana(historicalcmd::Pairs),
    /// Encode/decode exact presegmented units with an explicit codebook
    Nomenclator(historicalcmd::Table),
    /// Use an explicit ACA 10x10 syllabary keysquare
    Syllabary(historicalcmd::Table),
    /// Use the fixed Bazeries Grand Chiffre de 1691 table
    #[command(name = "great-cipher")]
    Great(historicalcmd::Great),
    /// Construct and apply the explicit deterministic ACA Numbered Key table
    NumberedKey(polycmd::Parameters),
    /// Transform exactly five caller headlines with the qualified ACA base profile
    Headlines(polycmd::Parameters),
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
    #[arg(short = 'k', long)]
    key: String,
    #[arg(long)]
    decode: bool,
}
#[derive(Args)]
struct BeaufortArgs {
    #[command(flatten)]
    input: Input,
    /// Nonempty ASCII-letter key, at most 64 KiB; apply twice to decode
    #[arg(short = 'k', long)]
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
    /// Exact UTF-8, validated without replacement or an added newline
    Text,
}

fn write_text(output: &mut impl Write, text: &str) -> io::Result<()> {
    writeln!(output, "{text}")
}

fn write_bytes(
    output: &mut impl Write,
    bytes: &[u8],
    format: Output,
) -> Result<(), Box<dyn Error>> {
    match format {
        Output::Hex => write_text(output, &encoding::hex_encode(bytes))?,
        Output::Raw => output.write_all(bytes)?,
        Output::Text => {
            let text = std::str::from_utf8(bytes).map_err(|e| {
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::InvalidInput,
                    "text output requires valid UTF-8; select raw or hex for arbitrary bytes",
                )
                .parameter("output")
                .offset(e.valid_up_to() as u64)
            })?;
            output.write_all(text.as_bytes())?;
        }
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    if let Command::Play(args) = &cli.command
        && let Some(command) = &args.command
    {
        if cli.lab_capture.is_some() || args.path.is_some() {
            return Err(cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidInput,
                "context_options_unavailable",
            )
            .into());
        }
        return if let Some(path) = &args.context {
            cube_cli::play::run_external(command, path)
        } else {
            cube_cli::play::run(command)
        }
        .map_err(Into::into);
    }
    let capture = capture::Target::requested(&cli)?;
    let mut output = io::stdout().lock();
    match cli.command {
        Command::Compose { command } => composecmd::run(command, &mut output)?,
        Command::Cryptanalysis { command } => cryptanalysiscmd::run(command, &mut output)?,
        Command::Probability { command } => probabilitycmd::run(command, &mut output)?,
        Command::Graph { command } => graphcmd::run(command, &mut output)?,
        Command::Logic { command } => logiccmd::run(command, &mut output)?,
        Command::Number {
            command: Number::Permutations(args),
        } => combinatoricscmd::pair(args, true, &mut output)?,
        Command::Number {
            command: Number::Binomial(args),
        } => combinatoricscmd::pair(args, false, &mut output)?,
        Command::Number {
            command: Number::Multinomial(args),
        } => combinatoricscmd::groups(args, &mut output)?,
        Command::Number {
            command: Number::Pascal(args),
        } => combinatoricscmd::pascal(args, &mut output)?,
        Command::Play(args) => playcmd::run(&args)?,
        Command::Number {
            command: Number::Gcd(args),
        } => modularcmd::pair(args, false, &mut output)?,
        Command::Number {
            command: Number::ExtendedGcd(args),
        } => modularcmd::pair(args, true, &mut output)?,
        Command::Number {
            command: Number::ModInverse(args),
        } => modularcmd::inverse(args, &mut output)?,
        Command::Number {
            command: Number::ModPow(args),
        } => modularcmd::power(args, &mut output)?,
        Command::Number {
            command: Number::DigitalRoot(args),
        } => digitalcmd::run(args, &mut output)?,
        Command::Blockchain { command } => blockchaincmd::run(command, &mut output)?,
        Command::Number {
            command: Number::Alphabet(args),
        } => alphabetcmd::run(args, &mut output)?,
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
        Command::Text { command } => textcmd::run(command, &mut output, capture.as_ref())?,
        Command::Image { command } => imagecmd::run(command, &mut output, capture.as_ref())?,
        Command::Inspect { path, json } => filecmd::info(path, json, &mut output)?,
        Command::File { command } => filecmd::run(command, &mut output)?,
        Command::Ops { command } => {
            let (request, json, max_output_bytes, metadata, contracts) = match command {
                Ops::List {
                    ref category,
                    json,
                    max_output_bytes,
                    metadata,
                    contracts,
                } => (
                    operations::DiscoveryRequest::List {
                        category: category.as_deref(),
                    },
                    json,
                    max_output_bytes,
                    metadata,
                    contracts,
                ),
                Ops::Describe {
                    ref id,
                    json,
                    max_output_bytes,
                    metadata,
                    contracts,
                } => (
                    operations::DiscoveryRequest::Describe { id },
                    json,
                    max_output_bytes,
                    metadata,
                    contracts,
                ),
            };
            let format = if json {
                operations::DiscoveryFormat::CliJson { metadata }
            } else {
                operations::DiscoveryFormat::Text
            };
            let maximum = max_output_bytes.unwrap_or(if contracts {
                operations::contracts::MAX_DOCUMENT_BYTES
            } else {
                operations::MAX_DISCOVERY_BYTES
            });
            let document = if contracts {
                operations::contracts::discovery_document(request, format, maximum)?
            } else {
                operations::discovery_document(request, format, maximum)?
            };
            output.write_all(document.as_bytes())?;
        }
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
                Encode::Tokens(args) => puzzlecodescmd::tokens(args, true)?,
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
            Decode::Tokens(args) => write_text(&mut output, &puzzlecodescmd::tokens(args, false)?)?,
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
            Decode::Hex(args) => {
                let input = if capture.is_some() {
                    args.input.read_text_limit(
                        cube_core::investigation::session::MAX_INPUT_BYTES,
                        "encoding.hex.decode",
                    )?
                } else {
                    String::from_utf8(args.input.read()?)?
                };
                let bytes = encoding::hex_decode(&input)?;
                let pending = capture.as_ref().map(|target| target.prepare(
                    "encoding.hex.decode", &serde_json::json!({"input":input}),
                    &serde_json::json!({}), &serde_json::json!({"output":cube_core::operations::contracts::values::Bytes{data:bytes.clone()}}),
                )).transpose()?;
                write_bytes(&mut output, &bytes, args.output)?;
                if let Some(pending) = pending {
                    pending.publish()?;
                }
            }
        },
        Command::Cipher { command } => {
            let text = match command {
                Cipher::Gronsfeld(args) => polycmd::gronsfeld(args)?,
                Cipher::InterruptedKey(args) => polycmd::interrupted_key(args)?,
                Cipher::ProgressiveKey(args) => polycmd::progressive_key(args)?,
                Cipher::VigenereMultiplicative(args) => polycmd::vigenere_multiplicative(args)?,
                Cipher::Enigma(args) => enigmacmd::run(args)?,
                Cipher::Aes { command } => return aescmd::run(command, &mut output),
                Cipher::ChaCha8(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::ChaCha8),
                        args,
                        &mut output,
                    );
                }
                Cipher::ChaCha12(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::ChaCha12),
                        args,
                        &mut output,
                    );
                }
                Cipher::ChaCha20(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::ChaCha20),
                        args,
                        &mut output,
                    );
                }
                Cipher::XChaCha20(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::XChaCha20),
                        args,
                        &mut output,
                    );
                }
                Cipher::Salsa12(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::Salsa12),
                        args,
                        &mut output,
                    );
                }
                Cipher::Salsa20(args) => {
                    return streamcmd::run(
                        Some(ciphers::stream::Profile::Salsa20),
                        args,
                        &mut output,
                    );
                }
                Cipher::Vernam(args) => return streamcmd::run(None, args, &mut output),
                Cipher::Cadenus(args) => fractioncmd::cadenus(args)?,
                Cipher::Swagman(args) => fractioncmd::swagman(args)?,
                Cipher::NihilistTransposition(args) => fractioncmd::nihilist_transposition(args)?,
                Cipher::Ubchi(args) => fractioncmd::ubchi(args)?,
                Cipher::CmBifid(args) => fractioncmd::cm_bifid(args)?,
                Cipher::Vic(args) => fractioncmd::vic(args)?,
                Cipher::Scytale(args) => fractioncmd::scytale(args)?,
                Cipher::RailFence(args) => fractioncmd::rail_fence(args)?,
                Cipher::Redefence(args) => fractioncmd::redefence(args)?,
                Cipher::SequenceTransposition(args) => fractioncmd::sequence_transposition(args)?,
                Cipher::Skip(args) => fractioncmd::skip(args)?,
                Cipher::Columnar(args) => fractioncmd::columnar(args)?,
                Cipher::CompleteColumnar(args) => fractioncmd::complete_columnar(args)?,
                Cipher::DoubleColumnar(args) => fractioncmd::double_columnar(args)?,
                Cipher::Myszkowski(args) => fractioncmd::myszkowski(args)?,
                Cipher::Amsco(args) => fractioncmd::amsco(args)?,
                Cipher::Route(args) => fractioncmd::route(args)?,
                Cipher::Spiral(args) => fractioncmd::spiral(args)?,
                Cipher::CaesarBox(args) => fractioncmd::caesar_box(args)?,
                Cipher::TurningGrille(args) => fractioncmd::turning_grille(args)?,
                Cipher::Bifid(args) => fractioncmd::bifid(args)?,
                Cipher::Trifid(args) => fractioncmd::trifid(args)?,
                Cipher::Adfgx(args) => fractioncmd::adfgx(args)?,
                Cipher::Adfgvx(args) => fractioncmd::adfgvx(args)?,
                Cipher::NihilistSubstitution(args) => fractioncmd::nihilist_substitution(args)?,
                Cipher::Bazeries(args) => fractioncmd::bazeries(args)?,
                Cipher::FractionatedMorse(args) => fractioncmd::fractionated_morse(args)?,
                Cipher::Morbit(args) => fractioncmd::morbit(args)?,
                Cipher::Pollux(args) => fractioncmd::pollux(args)?,
                Cipher::CardanExtract(args) => fractioncmd::cardan_extract(args)?,
                Cipher::CardanOverlay(args) => fractioncmd::cardan_overlay(args)?,
                Cipher::Trithemius(args) => polycmd::trithemius(args)?,
                Cipher::Autokey(args) => polycmd::autokey(args)?,
                Cipher::RunningKey(args) => polycmd::running_key(args)?,
                Cipher::Porta(args) => polycmd::porta(args)?,
                Cipher::Bellaso(args) => polycmd::bellaso(args)?,
                Cipher::Gromark(args) => polycmd::gromark(args)?,
                Cipher::PeriodicGromark(args) => polycmd::periodic_gromark(args)?,
                Cipher::Solitaire(args) => polycmd::solitaire(args)?,
                Cipher::Slidefair(args) => polycmd::slidefair(args)?,
                Cipher::Portax(args) => polycmd::portax(args)?,
                Cipher::Ragbaby(args) => polycmd::ragbaby(args)?,
                Cipher::Condi(args) => polycmd::condi(args)?,
                Cipher::Nicodemus(args) => polycmd::nicodemus(args)?,
                Cipher::Quagmire(args) => polycmd::quagmire(args)?,
                Cipher::Phillips(args) => polycmd::phillips(args)?,
                Cipher::Chaocipher(args) => polycmd::chaocipher(args)?,
                Cipher::Playfair(args) => polycmd::playfair(args)?,
                Cipher::SeriatedPlayfair(args) => polycmd::seriated_playfair(args)?,
                Cipher::TwoSquare(args) => polycmd::two_square(args)?,
                Cipher::FourSquare(args) => polycmd::four_square(args)?,
                Cipher::Hill(args) => polycmd::hill(args)?,
                Cipher::Digrafid(args) => polycmd::digrafid(args)?,
                Cipher::TriSquare(args) => polycmd::tri_square(args)?,
                Cipher::Substitution(args) => monocmd::substitution(args)?,
                Cipher::Homophonic(args) => monocmd::homophonic(args)?,
                Cipher::Straddling(args) => {
                    monocmd::digits(args, ciphers::mono::checkerboard::Profile::Straddling)?
                }
                Cipher::MonomeDinome(args) => {
                    monocmd::digits(args, ciphers::mono::checkerboard::Profile::MonomeDinome)?
                }
                Cipher::Tridigital(args) => {
                    monocmd::digits(args, ciphers::mono::checkerboard::Profile::Tridigital)?
                }
                Cipher::Grandpre(args) => monocmd::grandpre(args)?,
                Cipher::Rot5(input) => {
                    ciphers::mono::rotations::rot5(&input.read_text("cipher.rot5")?)?
                }
                Cipher::Rot18(input) => {
                    ciphers::mono::rotations::rot18(&input.read_text("cipher.rot18")?)?
                }
                Cipher::Malespin(input) => {
                    ciphers::mono::historical::malespin(&input.read_text("cipher.malespin")?)?
                }
                Cipher::Vatsyayana(args) => historicalcmd::paired(args)?,
                Cipher::Nomenclator(args) => historicalcmd::table(args, false)?,
                Cipher::Syllabary(args) => historicalcmd::table(args, true)?,
                Cipher::Great(args) => historicalcmd::great(args)?,
                Cipher::NumberedKey(args) => historicalcmd::numbered_key(args)?,
                Cipher::Headlines(args) => historicalcmd::headlines(args)?,
                Cipher::Rot8000(input) => {
                    ciphers::mono::rotations::rot8000(&input.read_text("cipher.rot8000")?)?
                }
                Cipher::Rot47(input) => {
                    ciphers::mono::rotations::rot47(&input.read_text("cipher.rot47")?)?
                }
                Cipher::Bacon(args) => puzzlecmd::bacon(args)?,
                Cipher::Polybius(args) => puzzlecmd::polybius(args)?,
                Cipher::BookEncode(args) => puzzlecodescmd::book_encode(args)?,
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
        Command::Esolang { command } => esolangcmd::run(command, &mut output)?,
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
            if let Some(status) = error.downcast_ref::<playcmd::DesktopExit>() {
                return ExitCode::from(status.0);
            }
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
