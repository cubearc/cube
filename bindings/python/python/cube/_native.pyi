"""Native operations. Invalid encoded input or an empty XOR key raises ValueError."""

from collections.abc import Iterator
from os import PathLike
from typing import Any, BinaryIO, Literal, NotRequired, TypedDict

class SolresolParameters(TypedDict):
    profile: Literal["gajewski_1902_monosyllables_v1"]

class SolresolProvenance(TypedDict):
    source_id: str
    profile_id: Literal["gajewski_1902_monosyllables_v1"]
    source_url: str
    source_sha256: str
    definition_sha256: str

class SolresolReading(TypedDict):
    token_index: int
    alternatives: list[str]

class SolresolEncodedReport(TypedDict):
    tokens: list[str]
    provenance: SolresolProvenance

class SolresolDecodedReport(SolresolEncodedReport):
    readings: list[SolresolReading]

def solresol_encode(
    terms: list[str], parameters: SolresolParameters
) -> SolresolEncodedReport: ...
def solresol_decode(
    tokens: list[str], parameters: SolresolParameters
) -> SolresolDecodedReport: ...

class KeyPhraseParameters(TypedDict):
    phrase: str

class KeyPhraseRecord(TypedDict):
    byte_offset: int
    input: str
    alternatives: list[str]
    mapped: bool

class KeyPhraseReadings(TypedDict):
    ambiguous: bool
    records: list[KeyPhraseRecord]
    definition_sha256: str

class KeyPhraseEncodedReport(KeyPhraseReadings):
    text: str

class KeyPhraseDecodedReport(KeyPhraseReadings):
    text: str | None

class ArnoldParameters(TypedDict):
    profile: Literal["arnold_paged_words_v1"]
    source_id: str
    index_base: Literal[0, 1]
    choice: Literal["first", "round_robin"]

class WordCoordinate(TypedDict):
    page: int
    line: int
    word: int

class ArnoldReport(TypedDict):
    words: list[str]
    coordinates: list[WordCoordinate]
    source_id: str
    source_sha256: str
    definition_sha256: str

def key_phrase_encode(
    text: str, parameters: KeyPhraseParameters
) -> KeyPhraseEncodedReport: ...
def key_phrase_decode(
    text: str, parameters: KeyPhraseParameters
) -> KeyPhraseDecodedReport: ...
def arnold_encode(
    pages: list[str], words: list[str], parameters: ArnoldParameters
) -> ArnoldReport: ...
def arnold_decode(
    pages: list[str], coordinates: list[WordCoordinate], parameters: ArnoldParameters
) -> ArnoldReport: ...

class HeadlinesParameters(TypedDict):
    profile: Literal["aca_base"]
    hat: str
    key: str
    setting: str
    punctuation: Literal["preserve", "aca_printed"]

class HeadlinesReport(TypedDict):
    headlines: list[str]
    alphabet: str
    mappings: list[str]
    definition_sha256: str

def headlines_encode(
    headlines: list[str], parameters: HeadlinesParameters
) -> HeadlinesReport: ...
def headlines_decode(
    headlines: list[str], parameters: HeadlinesParameters
) -> HeadlinesReport: ...

class NumberedKeyParameters(TypedDict):
    alphabet: str
    key: str
    preparation: Literal["exact", "ascii_lowercase_letters"]
    rotation: int
    origin: str
    width: int
    choice: Literal["first", "round_robin"]

class NumberedKeyEntry(TypedDict):
    symbol: str
    tokens: list[str]

class NumberedKeyReport(TypedDict):
    text: str
    tokens: list[str]
    table: list[NumberedKeyEntry]
    definition_sha256: str

def numbered_key_encode(
    text: str, parameters: NumberedKeyParameters
) -> NumberedKeyReport: ...
def numbered_key_decode(
    tokens: list[str], parameters: NumberedKeyParameters
) -> NumberedKeyReport: ...

class CadenusParameters(TypedDict):
    keyword: str
    padding: NotRequired[str | None]
    original_length: NotRequired[int | str | None]

class SwagmanParameters(TypedDict):
    key: list[list[int]]
    padding: NotRequired[str | None]
    original_length: NotRequired[int | str | None]

class NihilistTranspositionParameters(TypedDict):
    key: list[int]
    convention: Literal["writing_in", "taking_out"]
    readout: Literal["rows", "columns"]
    padding: NotRequired[str | None]
    original_length: NotRequired[int | str | None]

class TableReport(TypedDict):
    text: str
    original_length: int
    padding_count: int

class UbchiParameters(TypedDict):
    key_phrase: str
    nulls: str

class UbchiReport(TypedDict):
    text: str
    original_length: int
    null_count: int

class CmBifidParameters(TypedDict):
    input_square: str
    output_square: str
    period: int | str

class VicEnglish2010Parameters(TypedDict):
    phrase: str
    mnemonic: str
    date_digits: str
    personal_number: int | str
    message_indicator: str
    split_at: int | str
    padding: str

def cadenus_encode(text: str, parameters: CadenusParameters) -> TableReport: ...
def cadenus_decode(text: str, parameters: CadenusParameters) -> TableReport: ...
def swagman_encode(text: str, parameters: SwagmanParameters) -> TableReport: ...
def swagman_decode(text: str, parameters: SwagmanParameters) -> TableReport: ...
def nihilist_transposition_encode(
    text: str, parameters: NihilistTranspositionParameters
) -> TableReport: ...
def nihilist_transposition_decode(
    text: str, parameters: NihilistTranspositionParameters
) -> TableReport: ...
def ubchi_encode(text: str, parameters: UbchiParameters) -> UbchiReport: ...
def ubchi_decode(text: str, parameters: UbchiParameters) -> UbchiReport: ...
def cm_bifid_encode(text: str, parameters: CmBifidParameters) -> str: ...
def cm_bifid_decode(text: str, parameters: CmBifidParameters) -> str: ...
def vic_encode(text: str, parameters: VicEnglish2010Parameters) -> str: ...
def vic_decode(text: str, parameters: VicEnglish2010Parameters) -> str: ...

class StringCombinations(Iterator[str]):
    def __iter__(self) -> StringCombinations: ...
    def __next__(self) -> str: ...
    def __len__(self) -> int: ...
    def plan(self) -> dict[str, Any]: ...

def string_combinations(
    groups: list[list[str]], *, separator: str = "", max_results: int = 1000
) -> StringCombinations: ...
def clock_directions(
    time: str | None = None,
    *,
    hour_millidegrees: int | None = None,
    minute_millidegrees: int | None = None,
    zero_axis: str = "N",
    clockwise: bool = True,
    decode: bool = False,
) -> dict[str, Any]: ...
def semaphore_encode(text: str) -> str: ...
def semaphore_decode(text: str) -> str: ...
def image_pairs(
    images: list[Image],
    operation: str,
    *,
    max_pairs: int = 28,
    max_differences: int = 1000,
    plan_only: bool = False,
) -> dict[str, Any]: ...
def grid_factors(
    text: str, *, drop_whitespace: bool = False, max_candidates: int = 128
) -> dict[str, Any]: ...
def text_grid(
    text: str,
    width: int,
    *,
    drop_whitespace: bool = False,
    traversal: str = "row",
    colors: dict[str, list[int]] | None = None,
) -> dict[str, Any]: ...

class ProfileAlias(TypedDict):
    name: str
    usage: str

class ProfileSecurity(TypedDict):
    classification: Literal[
        "historical_cipher",
        "encoding",
        "unkeyed_digest",
        "legacy_digest",
        "checksum",
        "binary_transform",
        "private_material",
        "public_derivation",
        "puzzle_code",
    ]
    limitations: str
    private_input: bool
    private_output: bool
    sensitive_parameters: list[str]

class ProfileVariant(TypedDict):
    variant: str
    alphabet: str
    key: str
    key_advancement: str
    block_bits: int | None
    output_bits: int | None

class ProfileResourceBound(TypedDict):
    name: str
    maximum: int
    unit: str
    scope: Literal["core", "cli", "python", "pipeline"]

class DiscoveryMetadata(TypedDict):
    """Optional descriptive facts; not invocation/result schemas or permissions."""

    schema_version: Literal[1]
    canonical_name: str
    family: str
    aliases: list[ProfileAlias]
    security: ProfileSecurity
    profile: ProfileVariant
    resources: list[ProfileResourceBound]
    resource_notes: str
    rust_api: str
    python_api: str
    references: list[str]

def operations(
    category: str | None = None,
    *,
    include_metadata: bool = False,
    include_contracts: bool = False,
    max_output_bytes: int | None = None,
) -> list[dict[str, Any]]:
    """Sorted descriptors; opt in to metadata: DiscoveryMetadata where annotated.

    Legacy fields remain unchanged. pipeline is {"version": int} or None.
    Encoded output <=1 MiB, or <=4 MiB with include_contracts=True.
    Contract schemas use lossless wide-integer decimal strings.
    """
    ...

def describe_operation(
    id: str,
    *,
    include_metadata: bool = False,
    include_contracts: bool = False,
    max_output_bytes: int | None = None,
) -> dict[str, Any]:
    """Exact ID with optional metadata: DiscoveryMetadata; aliases are not IDs.

    Unknown IDs raise ValueError. Output budget semantics match operations().
    """
    ...

def base32_encode(data: bytes) -> str: ...
def base32_decode(text: str) -> bytes: ...
def base58check_encode(data: bytes, *, version: int) -> str: ...
def base58check_decode(text: str) -> dict[str, Any]:
    """Verified version, payload bytes and exact payload_hex; <=1421 text bytes."""
    ...

def base58_encode(data: bytes) -> str: ...
def base58_decode(text: str) -> bytes: ...
def base64_encode(data: bytes) -> str: ...
def base64_decode(text: str) -> bytes: ...
def hex_encode(data: bytes) -> str: ...
def hex_decode(text: str) -> bytes: ...
def caesar_encode(text: str, shift: int) -> str:
    """Rotate ASCII letters; shift must be in [-2147483648, 2147483647]."""
    ...

def caesar_decode(text: str, shift: int) -> str:
    """Invert Caesar encoding with the same signed 32-bit shift."""
    ...

def rot13(text: str) -> str: ...
def crc32(data: bytes) -> str: ...
def sha256(data: bytes) -> str: ...
def xor(data: bytes, key: bytes) -> bytes: ...
def file_signatures(data: bytes) -> list[dict[str, Any]]: ...
def inspect_file(path: str | PathLike[str]) -> dict[str, Any]: ...
def scan_file(
    path: str | PathLike[str],
    *,
    offset: int = 0,
    length: int | None = None,
    format: str | None = None,
    max_results: int = 1000,
) -> dict[str, Any]: ...
def extract_bytes(data: bytes, offset: int, length: int) -> bytes: ...
def extract_file_range(
    source: str | PathLike[str],
    destination: str | PathLike[str],
    offset: int,
    length: int,
) -> int: ...
def copy_file_range(
    source: str | PathLike[str], writer: BinaryIO, offset: int, length: int
) -> int: ...
def file_hex(
    path: str | PathLike[str],
    *,
    offset: int = 0,
    length: int | None = None,
    width: int = 16,
) -> list[dict[str, Any]]: ...
def file_strings(
    path: str | PathLike[str],
    *,
    offset: int = 0,
    length: int | None = None,
    min_length: int = 4,
    max_results: int = 1000,
) -> dict[str, Any]: ...
def compare_files(
    first: str | PathLike[str],
    second: str | PathLike[str],
    *,
    offset: int = 0,
    length: int | None = None,
    max_results: int = 1000,
) -> dict[str, Any]: ...
def inspect_exif(data: bytes) -> dict[str, Any]: ...
def file_exif(path: str | PathLike[str]) -> dict[str, Any]: ...
def appended_data(path: str | PathLike[str]) -> dict[str, Any]: ...
def validate_zip(data: bytes, *, offset: int = 0) -> dict[str, Any]:
    """Classic ZIP structure; payload CRCs and authenticity are not verified."""
    ...

def validate_zip_file(
    path: str | PathLike[str], *, offset: int = 0
) -> dict[str, Any]: ...
def carve_file(
    source: str | PathLike[str],
    destination: str | PathLike[str],
    offset: int,
    *,
    length: int | None = None,
    max_bytes: int = 16777216,
) -> dict[str, Any]:
    """Recover an exact range or validated candidate; never overwrite or unpack."""
    ...

def png_chunks(
    data: bytes, *, strict: bool = True, payload_budget: int = 0
) -> dict[str, Any]: ...

class Image:
    def autocorrelation(
        self,
        channel: str,
        lags: list[SpatialLag] | str,
        *,
        region: tuple[int, int, int, int],
        expand_palette: bool = False,
        limits: DiscreteLimits | None = None,
    ) -> SpatialReport: ...
    def to_csv(self) -> str: ...
    @staticmethod
    def from_csv(text: str) -> Image: ...
    def color_layers(
        self,
        colors: list[list[int]] | None = None,
        *,
        representation: str = "mask",
        expand_palette: bool = False,
        max_colors: int = 16,
        max_outputs: int = 16,
    ) -> dict[str, Any]: ...
    def qr(self, *, max_symbols: int = 8) -> dict[str, Any]: ...
    @staticmethod
    def inspect_layout(
        data: bytes,
        width: int,
        height: int,
        layout: str,
        *,
        row_stride: int | None = None,
        offset: int = 0,
    ) -> dict[str, Any]: ...
    def exact_colors(
        self,
        colors: list[list[int]],
        *,
        representation: str = "mask",
        expand_palette: bool = False,
    ) -> dict[str, Any]: ...
    @staticmethod
    def from_raw(
        data: bytes,
        width: int,
        height: int,
        layout: str,
        *,
        row_stride: int | None = None,
        offset: int = 0,
    ) -> Image: ...
    def provenance(self) -> dict[str, Any]: ...
    @staticmethod
    def decode(data: bytes) -> Image: ...
    @staticmethod
    def open(path: str | PathLike[str]) -> Image: ...
    @staticmethod
    def from_samples(
        width: int,
        height: int,
        model: str,
        samples: bytes,
        palette: list[list[int]] | None = None,
        transparency: bytes | None = None,
    ) -> Image: ...
    @property
    def width(self) -> int: ...
    @property
    def height(self) -> int: ...
    @property
    def samples(self) -> bytes: ...
    def info(self) -> dict[str, Any]: ...
    def exif(self) -> dict[str, Any] | None: ...
    def expand_palette(self) -> Image: ...
    def encode_png(self) -> bytes: ...
    def save_png(self, path: str | PathLike[str]) -> None: ...
    def pixel(self, x: int, y: int, *, expand_palette: bool = False) -> list[int]: ...
    def pixels(
        self,
        *,
        region: tuple[int, int, int, int] | None = None,
        channels: list[str] | None = None,
        expand_palette: bool = False,
        traversal: str = "row",
        start_corner: str = "top_left",
        reverse: bool = False,
    ) -> dict[str, Any]: ...
    def channel(
        self,
        channel: str,
        *,
        region: tuple[int, int, int, int] | None = None,
        expand_palette: bool = False,
    ) -> dict[str, Any]: ...
    def histogram(
        self,
        *,
        region: tuple[int, int, int, int] | None = None,
        channels: list[str] | None = None,
        max_colors: int = 1000,
        expand_palette: bool = False,
    ) -> dict[str, Any]: ...
    def compare(
        self,
        other: Image,
        *,
        region: tuple[int, int, int, int] | None = None,
        channels: list[str] | None = None,
        max_results: int = 1000,
        mask: bool = False,
    ) -> dict[str, Any]: ...
    def bitwise(self, other: Image, operation: str) -> Image: ...
    def rotate(self, degrees: int) -> Image: ...
    def mirror(self, axis: str) -> Image: ...
    def crop(self, region: tuple[int, int, int, int]) -> Image: ...
    def invert(
        self, *, channels: list[str] | None = None, expand_palette: bool = False
    ) -> Image: ...
    def reorder(
        self, channels: list[str], *, expand_palette: bool = False
    ) -> Image: ...
    def grayscale(self, *, expand_palette: bool = False) -> Image: ...
    def bit_plane(
        self,
        channel: str,
        bit: int,
        *,
        region: tuple[int, int, int, int] | None = None,
        expand_palette: bool = False,
        visualization: bool = False,
    ) -> dict[str, Any]: ...
    def lsb(
        self,
        *,
        region: tuple[int, int, int, int] | None = None,
        channels: list[str] | None = None,
        bits: list[int] = ...,
        traversal: str = "row",
        start_corner: str = "top_left",
        reverse: bool = False,
        bit_order: str = "msb",
        incomplete: str = "reject",
        expand_palette: bool = False,
    ) -> dict[str, Any]: ...

def a1z26_encode(text: str) -> str: ...
def a1z26_decode(text: str) -> str: ...
def and_bytes(data: bytes, key: bytes) -> bytes: ...
def or_bytes(data: bytes, key: bytes) -> bytes: ...
def not_bytes(data: bytes) -> bytes: ...
def reverse_bytes(data: bytes) -> bytes: ...
def shift_left(data: bytes, bits: int, *, unit_bytes: int = 1) -> bytes: ...
def shift_right(data: bytes, bits: int, *, unit_bytes: int = 1) -> bytes: ...
def swap_endian(data: bytes, *, unit_bytes: int = 1) -> bytes: ...
def atbash(text: str) -> str: ...
def vigenere_encode(text: str, key: str) -> str: ...
def vigenere_decode(text: str, key: str) -> str: ...
def beaufort(text: str, key: str) -> str: ...
def affine_encode(text: str, a: int, b: int) -> str: ...
def affine_decode(text: str, a: int, b: int) -> str: ...

UNICODE_VERSION: tuple[int, int, int]
WORD_UNICODE_VERSION: tuple[int, int, int]

def select_text(text: str, selector: str) -> str: ...
def text_statistics(
    text: str,
    *,
    ngram_size: int = 1,
    normalization: str = "none",
    case: str = "preserve",
    filter: str = "all",
    max_results: int = 1000,
) -> dict[str, Any]: ...
def normalize_text(text: str, form: str) -> str: ...
def reverse_text(text: str) -> str: ...
def extract_characters(
    text: str,
    *,
    indices: list[int] | None = None,
    start: int | None = None,
    stride: int | None = None,
    stop: int | None = None,
) -> str: ...
def morse_encode(text: str) -> str: ...
def morse_decode(text: str) -> str: ...
def braille_encode(data: bytes) -> str: ...
def braille_decode(text: str) -> bytes: ...
def constant_digits(
    constant: str,
    *,
    start: int | None = None,
    count: int | None = None,
    indices: list[int] | None = None,
    max_digits: int = 256,
) -> dict[str, Any]:
    """Certified fractional digits; explicit prefix budget 1..4096, default 256."""
    ...

def digital_root(value: int, *, radix: int = 10) -> int:
    """Decimal digital root, using magnitude<=2^128-1; builtin int, no bool/float."""
    ...

def gcd(a: int, b: int) -> int:
    """Exact u64 GCD, (0,0)=0; builtin ints only, no bool/float."""
    ...

def extended_gcd(a: int, b: int) -> dict[str, int]:
    """Deterministic gcd/x/y satisfying a*x+b*y=gcd; exact Python ints."""
    ...

def mod_inverse(a: int, modulus: int) -> int:
    """Canonical u64 inverse; modulus>=2 and coprime inputs required."""
    ...

def mod_pow(base: int, exponent: int, modulus: int) -> int:
    """Exact u64 power; positive modulus, 0^0=1 mod modulus."""
    ...

def convert_base(text: str, from_base: int, to_base: int) -> str: ...
def inspect_wave(data: bytes) -> dict[str, Any]: ...
def inspect_wave_file(path: str | PathLike[str]) -> dict[str, Any]: ...
def wave_samples(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    include_values: bool = False,
) -> dict[str, Any]: ...
def wave_file_samples(
    path: str | PathLike[str],
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    include_values: bool = False,
) -> dict[str, Any]: ...
def wave_envelope(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    bucket_frames: int = 1024,
    max_buckets: int = 1000,
) -> dict[str, Any]: ...
def wave_file_envelope(
    path: str | PathLike[str],
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    bucket_frames: int = 1024,
    max_buckets: int = 1000,
) -> dict[str, Any]: ...
def utf8_encode(text: str) -> bytes: ...
def concat_text(left: str, right: str, separator: str = "") -> str: ...
def parse_json(document: str) -> Any: ...
def select_field(document: str, pointer: str = "") -> Any: ...
def json_text(document: str) -> str: ...
def parse_integer(text: str) -> int: ...
def maximum(values: list[int]) -> int: ...
def add_integers(left: int, right: int) -> int: ...
def image_dimensions(image: Image) -> dict[str, list[int]]: ...
def concat_bytes(left: bytes, right: bytes) -> bytes: ...
def utf8_decode(data: bytes) -> str: ...

class CancellationToken:
    def __init__(self) -> None: ...
    def cancel(self) -> None: ...
    @property
    def is_cancelled(self) -> bool: ...

def pipeline_adapters() -> dict[str, Any]:
    """Adapters declare pipeline kinds, version and the direct descriptor."""
    ...

def validate_pipeline(
    steps: list[PipelineStep],
    input_type: str,
    *,
    limits: dict[str, Any] | None = None,
) -> dict[str, Any]: ...
def execute_pipeline(
    steps: list[PipelineStep],
    input: bytes | str | dict[str, Any],
    *,
    limits: dict[str, Any] | None = None,
    capture: list[int] | None = None,
    cancellation: CancellationToken | None = None,
) -> PipelineExecution: ...
def validate_recipe(document: str) -> dict[str, Any]: ...
def execute_recipe(
    document: str,
    inputs: dict[str, bytes | str | dict[str, Any]],
    *,
    capture: list[int] | None = None,
    cancellation: CancellationToken | None = None,
) -> PipelineExecution: ...
def check_recipe(
    document: str,
    inputs: dict[str, bytes | str | dict[str, Any]],
    expected: bytes | str | dict[str, Any],
) -> RecipeCheck: ...
def triage_input(data: bytes, *, max_read_bytes: int = 12) -> dict[str, Any]: ...

class InvestigationSession:
    def __init__(self, data: bytes, *, identify: bool = False) -> None: ...
    @staticmethod
    def from_json(document: str) -> InvestigationSession: ...
    def run(
        self,
        data: bytes,
        definition: str,
        *,
        record_parameters: bool = False,
        retain_result: bool = False,
        cancellation: CancellationToken | None = None,
    ) -> int: ...
    def execute_pipeline(
        self,
        data: bytes,
        steps: list[PipelineStep],
        *,
        record_parameters: bool = False,
        retain_result: bool = False,
        cancellation: CancellationToken | None = None,
    ) -> int: ...
    def add_hypothesis(self, text: str, findings: list[int] | None = None) -> int: ...
    def redact_parameters(self, run_id: int) -> None: ...
    def discard_result(self, run_id: int) -> None: ...
    def to_json(
        self,
        *,
        include_results: list[int] | None = None,
        include_parameters: list[int] | None = None,
        include_hypotheses: bool = False,
    ) -> str: ...
    def report(self) -> str: ...
    def replay(
        self, data: bytes, *, cancellation: CancellationToken | None = None
    ) -> InvestigationSession: ...

def plan_investigation(
    input_size: int, profile_json: str | None = None
) -> dict[str, Any]: ...
def investigate(
    data: bytes,
    profile_json: str | None = None,
    *,
    cancellation: CancellationToken | None = None,
) -> dict[str, Any]: ...
def book_decode(
    source: str,
    coordinates: str,
    *,
    scheme: str = "word_character",
    index_base: int = 1,
) -> str: ...
def sha512(data: bytes) -> str: ...
def sha1(data: bytes) -> str: ...
def md5(data: bytes) -> str: ...
def crc32c(data: bytes) -> str: ...
def crc16_ibm3740(data: bytes) -> str: ...
def wave_spectrum(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    fft_size: int = 1024,
    window: str = "hann",
) -> dict[str, Any]: ...
def wave_spectrogram(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    fft_size: int = 1024,
    hop_size: int = 512,
    window: str = "hann",
    final_window: str = "drop",
) -> dict[str, Any]: ...
def wave_bits(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    bits: list[int] = ...,
    reverse_frames: bool = False,
    bit_order: str = "msb",
    incomplete: str = "reject",
) -> dict[str, Any]: ...
def reverse_wave(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
) -> bytes: ...
def detect_dtmf(
    data: bytes,
    *,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    window_ms: int = 40,
    min_tone_ms: int = 80,
    min_gap_ms: int = 40,
    min_level_dbfs: int = -40,
    dominance_db: int = 6,
    max_twist_db: int = 8,
    purity_percent: int = 75,
) -> dict[str, Any]: ...
def detect_morse_intervals(
    data: bytes,
    *,
    frequency_hz: int,
    channels: list[int] | None = None,
    start_frame: int = 0,
    frames: int | None = None,
    window_ms: int = 10,
    min_level_dbfs: int = -40,
    purity_percent: int = 80,
) -> dict[str, Any]: ...
def transcribe_morse(
    intervals: list[dict[str, Any]],
    *,
    sample_rate: int,
    unit_ms: int,
    tolerance_percent: int = 20,
) -> dict[str, Any]: ...
def inspect_audio_metadata(data: bytes) -> dict[str, Any]: ...
def inspect_video(data: bytes) -> dict[str, Any]: ...
def inspect_html(document: str) -> dict[str, Any]: ...
def inspect_har(document: str) -> dict[str, Any]: ...
def extract_html_asset(
    document: str, index: int, *, selector: str = "inline"
) -> dict[str, Any]: ...
def extract_har_asset(document: str, index: int) -> dict[str, Any]: ...
def extract_video_frames(
    data: bytes, track_id: int, indexes: list[int]
) -> list[dict[str, Any]]: ...
def compare_video_frames(
    data: bytes,
    track_id: int,
    pairs: list[tuple[int, int]],
    *,
    channels: list[str] | None = None,
    max_results: int = 1000,
) -> dict[str, Any]: ...
def extract_video_audio(data: bytes, track_id: int) -> dict[str, Any]: ...
def validate_wif(text: str, *, expected_network: str | None = None) -> dict[str, Any]:
    """Safe structural metadata only; no secret or derived fingerprint."""
    ...

def validate_private_scalar(
    data: bytes | str, *, format: str = "raw"
) -> dict[str, Any]:
    """Validate exact raw32 or hex64 secp256k1 material without private export."""
    ...

def ripemd160(data: bytes) -> str: ...
def derive_bitcoin_address(
    data: bytes | str, *, network: str, compressed: bool, format: str = "raw"
) -> dict[str, Any]:
    """Public SEC1/P2PKH only; WIF network and encoding intent must agree."""
    ...

def export_private_key(
    data: bytes | str,
    *,
    output: str,
    format: str = "raw",
    network: str | None = None,
    compressed: bool | None = None,
) -> bytes | str:
    """Explicit private export; returned plaintext stays caller-owned."""
    ...

def keccak256(data: bytes) -> str:
    """Ethereum Keccak-256, distinct from SHA3-256; <=16 MiB exact bytes."""
    ...

def derive_ethereum_address(
    data: bytes | str, *, format: str = "raw", checksum: str = "eip55"
) -> dict[str, Any]:
    """Public x||y/address only; exact raw32/hex64 scalar, lowercase or EIP-55."""
    ...

class CandidateValidation(Iterator[dict[str, Any]]):
    """Bounded native iterator; progress.complete stays false until consumed."""
    def __iter__(self) -> CandidateValidation: ...
    def __next__(self) -> dict[str, Any]: ...
    def __len__(self) -> int: ...
    def plan(self) -> dict[str, Any]: ...
    def progress(self) -> dict[str, Any]: ...

def validate_key_candidates(
    candidates: list[dict[str, str]],
    *,
    derive: str | None = None,
    network: str | None = None,
    compressed: bool | None = None,
    checksum: str | None = None,
) -> CandidateValidation:
    """WIF/hex64 validation only by default; public derivation requires opt-in."""
    ...

class MaskedSelector(TypedDict, total=False):
    kind: Literal["indices", "every_nth", "fibonacci"]
    indices: list[int]
    n: int
    exclude: list[int]

class MaskedTransformation(TypedDict, total=False):
    kind: Literal["atbash", "caesar", "rot13", "identity"]
    shift: int

class MaskedOverride(TypedDict):
    position: int
    replacement: str

class AppliedMaskedOverride(MaskedOverride):
    previous: str

class MaskedConfig(TypedDict):
    index_base: int
    selector: MaskedSelector
    selected: MaskedTransformation
    otherwise: MaskedTransformation
    overrides: list[MaskedOverride]

class MaskedReport(TypedDict):
    original_input: str
    config: MaskedConfig
    selected_positions: list[int]
    deterministic_text: str
    overrides: list[AppliedMaskedOverride]
    final_text: str

def transform_masked(
    text: str,
    *,
    selector: Literal["indices", "every_nth", "fibonacci"],
    selected: Literal["atbash", "caesar", "rot13", "identity"],
    indices: list[int] | None = None,
    nth: int | None = None,
    exclude: list[int] | None = None,
    index_base: int = 0,
    selected_shift: int | None = None,
    otherwise: Literal["atbash", "caesar", "rot13", "identity"] = "identity",
    otherwise_shift: int | None = None,
    overrides: list[tuple[int, str]] | None = None,
) -> MaskedReport:
    """Every scalar counts; return deterministic text and explicit override audit."""
    ...

class PipelineMaskedAudits(TypedDict, total=False):
    masked_audits: dict[int, MaskedReport]

class PipelineExecution(PipelineMaskedAudits):
    output: Any
    output_type: str
    intermediates: dict[int, Any]
    work_bytes_upper_bound: int

class RecipeCheck(PipelineMaskedAudits):
    passed: bool
    format_version: int
    engine_api: int
    engine_version: str

class UnsignedListParameter(TypedDict):
    unsigned_list: list[int]

class TextListParameter(TypedDict):
    text_list: list[str]

class BytesParameter(TypedDict):
    base64: str

class PipelineStepHeader(TypedDict):
    operation: str
    version: int

class PipelineStep(PipelineStepHeader, total=False):
    parameters: dict[
        str,
        int | bool | str | BytesParameter | UnsignedListParameter | TextListParameter,
    ]

# Explicit puzzle alphabets and physical/scalar mappings. No pipeline adapters.
def bacon_encode(text: str, variant: str, *, notation: str = "ab") -> str: ...
def bacon_decode(text: str, variant: str, *, notation: str = "ab") -> str: ...
def polybius_encode(
    text: str,
    *,
    preset: str | None = None,
    alphabet: str | None = None,
    side: int | None = None,
    aliases: list[tuple[str, str]] | None = None,
    index_base: int = 1,
) -> list[list[int] | None]: ...
def polybius_decode(
    tokens: list[list[int] | tuple[int, int] | None],
    *,
    preset: str | None = None,
    alphabet: str | None = None,
    side: int | None = None,
    aliases: list[tuple[str, str]] | None = None,
    index_base: int = 1,
) -> str: ...
def keyboard_remap(
    text: str,
    source: str | dict,
    target: str | dict,
    *,
    unmapped: str = "preserve",
) -> str: ...
def keypad_encode(text: str, mode: str, *, mapping: dict | None = None) -> str: ...
def keypad_decode(text: str, mode: str, *, mapping: dict | None = None) -> dict: ...
def map_character_classes(
    text: str,
    *,
    y: str | None = None,
    unmapped: str = "error",
    definition: dict | None = None,
) -> dict: ...
def convert_alphabet(
    text: str, source_alphabet: str, target_alphabet: str, from_base: int, to_base: int
) -> str: ...
def keyed_skip(
    text: str, steps: list[int], count: int, *, options: dict | None = None
) -> dict: ...
def josephus_extract(
    text: str,
    steps: list[int],
    *,
    count: int | None = None,
    options: dict | None = None,
) -> dict: ...
def extract_word_characters(
    text: str,
    *,
    positions: list[int] | None = None,
    per_word: list[list[int]] | None = None,
    index_base: int = 1,
    short_word: str = "error",
) -> dict: ...
def word_lengths(text: str) -> list[dict]: ...
def text_codepoints(text: str, *, format: str = "decimal") -> list[dict]: ...
def word_values(
    text: str,
    *,
    table: list[tuple[str, int]] | None = None,
    unmapped: str = "error",
    sums: bool = False,
    max_sum: int = 18446744073709551615,
) -> list[dict]: ...
def inspect_unicode(text: str, *, options: dict | None = None) -> dict: ...
def compare_unicode(
    left: str, right: str, *, normalization: str = "none", options: dict | None = None
) -> dict: ...
def rotate_left(data: bytes, count: int, *, unit_bytes: int | None = None) -> bytes: ...
def rotate_right(
    data: bytes, count: int, *, unit_bytes: int | None = None
) -> bytes: ...
def xor_exact(data: bytes, key: bytes) -> bytes: ...
def add_wrapping(
    data: bytes, key: bytes, *, unit_bytes: int | None = None
) -> bytes: ...
def sub_wrapping(
    data: bytes, key: bytes, *, unit_bytes: int | None = None
) -> bytes: ...

class BrainfuckOptions(TypedDict, total=False):
    cell_bits: Literal[8, 16, 32]
    eof: Literal["error", "zero", "unchanged"]

class BrainfuckLimits(TypedDict, total=False):
    max_instructions: int
    max_output_bytes: int
    max_memory_bytes: int

class BrainfuckDiagnostics(TypedDict, total=False):
    tape_start: int
    tape_len: int
    max_trace_records: int
    max_trace_bytes: int
    max_report_bytes: int

class BrainfuckComparisonWidths(TypedDict):
    widths: list[Literal[8, 16, 32]]

class BrainfuckComparisonOptions(BrainfuckComparisonWidths, total=False):
    eof: Literal["error", "zero", "unchanged"]

class BrainfuckCell(TypedDict):
    index: int
    value: int

class BrainfuckTraceRecord(TypedDict):
    instruction: int
    source_offset: int
    command: str
    pointer_before: int
    pointer_after: int
    cell_before: int
    cell_after: int
    output_byte: int | None
    input_consumed: bool

class BrainfuckReport(TypedDict):
    completed: bool
    budget_scope: Literal["per_run", "aggregate", "both"] | None
    overflow_policy: Literal["wrapping"]
    tape_cells: int
    stop_reason: Literal[
        "completed",
        "instruction_limit",
        "output_limit",
        "input_exhausted",
        "pointer_out_of_bounds",
    ]
    error: dict[str, Any] | None
    options: BrainfuckOptions
    limits: BrainfuckLimits
    diagnostics: BrainfuckDiagnostics
    instructions: int
    input_consumed: int
    final_pointer: int
    tape: list[BrainfuckCell]
    output: bytes
    trace: list[BrainfuckTraceRecord]
    trace_complete: bool
    memory_bytes_upper_bound: int
    report_bytes_upper_bound: int

class BrainfuckComparison(TypedDict):
    runs: list[BrainfuckReport]
    all_completed: bool
    equal_completed_outputs: bool | None
    limits: BrainfuckLimits
    per_run_limits: BrainfuckLimits
    instructions: int
    output_bytes: int
    memory_bytes_upper_bound: int

def brainfuck_execute(
    source: str,
    input: bytes = b"",
    *,
    options: BrainfuckOptions | None = None,
    limits: BrainfuckLimits | None = None,
) -> bytes: ...
def brainfuck_report(
    source: str,
    input: bytes = b"",
    *,
    options: BrainfuckOptions | None = None,
    limits: BrainfuckLimits | None = None,
    diagnostics: BrainfuckDiagnostics | None = None,
) -> BrainfuckReport: ...
def brainfuck_compare(
    source: str,
    input: bytes = b"",
    *,
    options: BrainfuckComparisonOptions,
    limits: BrainfuckLimits | None = None,
    aggregate: BrainfuckLimits | None = None,
) -> BrainfuckComparison: ...

class CountLimits(TypedDict, total=False):
    max_work: int
    max_report_bytes: int

class PascalLimits(TypedDict, total=False):
    max_work: int
    max_output_bytes: int

class MultisetProfile(TypedDict, total=False):
    normalization: Literal["none", "NFC", "NFD", "NFKC", "NFKD"]
    case: Literal["preserve", "ascii_lower"]
    filter: Literal["all", "ascii_letters"]

class DiscreteLimits(TypedDict, total=False):
    max_work: int
    max_memory_bytes: int
    max_report_bytes: int

class ScalarCount(TypedDict):
    scalar: str
    count: int

class MultisetReport(TypedDict):
    profile: MultisetProfile
    normalization_unicode_version: list[int]
    equal: bool
    contains: bool
    source: list[ScalarCount]
    requested: list[ScalarCount]
    residual: list[ScalarCount]
    deficits: list[ScalarCount]
    work: int
    memory_bytes_upper_bound: int
    report_bytes_upper_bound: int

class BooleanLimits(DiscreteLimits, total=False):
    max_rows: int

class BooleanModelRequired(TypedDict):
    variables: list[str]
    formula: dict[str, Any]

class BooleanModel(BooleanModelRequired, total=False):
    other: dict[str, Any] | None

class BooleanBinding(TypedDict):
    name: str
    value: bool

class DiscreteReportBounds(TypedDict):
    work: int
    memory_bytes_upper_bound: int
    report_bytes_upper_bound: int

class BooleanEvaluation(DiscreteReportBounds):
    value: bool
    other_value: bool | None

class BooleanTruthRow(TypedDict):
    assignment: list[BooleanBinding]
    value: bool
    other_value: bool | None

class BooleanRelationship(TypedDict):
    implies: bool
    equivalent: bool
    implication_counterexample: list[BooleanBinding] | None
    equivalence_counterexample: list[BooleanBinding] | None

class BooleanAnalysis(DiscreteReportBounds):
    variables: list[str]
    assignments: int
    true_count: int
    false_count: int
    satisfiable: bool
    tautology: bool
    first_satisfying: list[BooleanBinding] | None
    first_falsifying: list[BooleanBinding] | None
    relationship: BooleanRelationship | None
    rows: list[BooleanTruthRow]
    listing_complete: bool

# JSON input uses the literal key "from", which is a Python keyword; dict literals
# express it directly. Functional TypedDict preserves the actual transport key.
GraphEdge = TypedDict("GraphEdge", {"id": str, "from": str, "to": str})

class GraphDefinition(TypedDict):
    vertices: list[str]
    directed: bool
    edges: list[GraphEdge]

class GraphPath(TypedDict):
    vertices: list[str]
    edges: list[str]

class GraphComponents(DiscreteReportBounds):
    result: list[list[str]]

class GraphPathReport(DiscreteReportBounds):
    result: GraphPath | None

class EulerResult(TypedDict):
    exists: bool
    circuit: bool
    path: GraphPath | None
    reason: str | None

class GraphEulerReport(DiscreteReportBounds):
    result: EulerResult

def permutations(n: int, k: int, *, limits: CountLimits | None = None) -> int: ...
def binomial(n: int, k: int, *, limits: CountLimits | None = None) -> int: ...
def multinomial(groups: list[int], *, limits: CountLimits | None = None) -> int: ...
def pascal_triangle(
    rows: int, *, limits: PascalLimits | None = None
) -> list[list[int]]: ...
def multiset_compare(
    source: str,
    requested: str,
    *,
    profile: MultisetProfile | None = None,
    limits: DiscreteLimits | None = None,
) -> MultisetReport: ...
def multiset_subtract(
    source: str,
    requested: str,
    *,
    profile: MultisetProfile | None = None,
    limits: DiscreteLimits | None = None,
) -> MultisetReport: ...
def boolean_evaluate(
    model: BooleanModel | str,
    assignment: list[BooleanBinding] | str,
    *,
    limits: BooleanLimits | None = None,
) -> BooleanEvaluation: ...
def boolean_analyze(
    model: BooleanModel | str, *, limits: BooleanLimits | None = None
) -> BooleanAnalysis: ...
def graph_components(
    graph: GraphDefinition | str, *, limits: DiscreteLimits | None = None
) -> GraphComponents: ...
def graph_shortest_path(
    graph: GraphDefinition | str,
    source: str,
    target: str,
    *,
    limits: DiscreteLimits | None = None,
) -> GraphPathReport: ...
def graph_cycle(
    graph: GraphDefinition | str, *, limits: DiscreteLimits | None = None
) -> GraphPathReport: ...
def graph_euler_trail(
    graph: GraphDefinition | str,
    *,
    start: str | None = None,
    limits: DiscreteLimits | None = None,
) -> GraphEulerReport: ...

class AnagramOptions(DiscreteLimits, total=False):
    max_words: int
    reuse: bool
    max_results: int

class DictionaryEntry(TypedDict):
    text: str
    aliases: list[int]

class AnagramReport(DiscreteReportBounds):
    profile: MultisetProfile
    normalization_unicode_version: list[int]
    options: AnagramOptions
    dictionary: list[DictionaryEntry]
    solutions: list[list[int]]
    complete: bool
    stop_reason: Literal["work_limit", "result_limit"] | None
    nodes: int

class LadderProfile(TypedDict, total=False):
    normalization: Literal["none", "NFC", "NFD", "NFKC", "NFKD"]
    case: Literal["preserve", "ascii_lower"]

class LadderFailure(TypedDict):
    position: int
    step: int | None
    reason: Literal["not_in_dictionary", "not_one_substitution"]

class LadderReport(DiscreteReportBounds):
    profile: MultisetProfile
    normalization_unicode_version: list[int]
    dictionary: list[DictionaryEntry]
    path: list[int] | None
    valid: bool
    failure: LadderFailure | None
    status: Literal["valid", "invalid", "found", "no_path"]
    complete: bool
    edge_count: int

class ExactFraction(TypedDict):
    numerator: int
    denominator: int

class SignedFraction(ExactFraction):
    negative: bool

class FiniteOutcomeRequired(TypedDict):
    id: str
    weight: int | str

class FiniteOutcome(FiniteOutcomeRequired, total=False):
    value: int | None

class FiniteModelRequired(TypedDict):
    outcomes: list[FiniteOutcome]
    event_a: list[str]

class FiniteModel(FiniteModelRequired, total=False):
    event_b: list[str] | None

class OutcomeProbability(TypedDict):
    id: str
    probability: ExactFraction

class FiniteReport(DiscreteReportBounds):
    assumptions: FiniteModel
    total_mass: int
    outcomes: list[OutcomeProbability]
    probability_a: ExactFraction
    probability_b: ExactFraction | None
    intersection: ExactFraction | None
    conditional_a_given_b: ExactFraction | None
    posterior: list[OutcomeProbability] | None
    expected_value: SignedFraction | None

class DieFace(TypedDict):
    value: int
    weight: int | str

class DiceModel(TypedDict):
    dice: list[list[DieFace]]

class DicePair(TypedDict):
    left: DiceModel
    right: DiceModel

class DistributionPoint(TypedDict):
    value: int
    mass: int
    probability: ExactFraction

class DiceSumReport(DiscreteReportBounds):
    assumptions: DiceModel
    independence: str
    distribution: list[DistributionPoint]
    total_mass: int
    expected_value: SignedFraction

class DiceComparison(DiscreteReportBounds):
    assumptions: DicePair
    independence: str
    win: ExactFraction
    tie: ExactFraction
    loss: ExactFraction
    win_mass: int
    tie_mass: int
    loss_mass: int
    total_mass: int

class SpatialLag(TypedDict):
    dx: int
    dy: int

class ImageRegion(TypedDict):
    x: int
    y: int
    width: int
    height: int

class SpatialMeasurement(TypedDict):
    lag: SpatialLag
    source_overlap: ImageRegion
    target_overlap: ImageRegion
    count: int
    sum_x: int
    sum_y: int
    sum_x_squared: int
    sum_y_squared: int
    sum_xy: int
    squared_difference: int
    population_covariance: float
    correlation: float | None

class SpatialReport(DiscreteReportBounds):
    region: ImageRegion
    channel: str
    representation: str
    source_format: str
    measurements: list[SpatialMeasurement]

def anagram_search(
    source: str,
    dictionary: list[str] | str,
    *,
    profile: MultisetProfile | None = None,
    options: AnagramOptions | None = None,
) -> AnagramReport: ...
def word_ladder_validate(
    dictionary: list[str] | str,
    path: list[str] | str,
    *,
    profile: LadderProfile | None = None,
    limits: DiscreteLimits | None = None,
) -> LadderReport: ...
def word_ladder_shortest(
    dictionary: list[str] | str,
    start: str,
    end: str,
    *,
    profile: LadderProfile | None = None,
    limits: DiscreteLimits | None = None,
) -> LadderReport: ...
def finite_probability(
    model: FiniteModel | str, *, limits: DiscreteLimits | None = None
) -> FiniteReport: ...
def dice_sum(
    model: DiceModel | str, *, limits: DiscreteLimits | None = None
) -> DiceSumReport: ...
def dice_compare(
    model: DicePair | str, *, limits: DiscreteLimits | None = None
) -> DiceComparison: ...

class HomophonicEntry(TypedDict):
    symbol: str
    tokens: list[str]

class HomophonicDefinition(TypedDict):
    ascii_uppercase: bool
    entries: list[HomophonicEntry]

class HomophonicCounter(TypedDict):
    symbol: str
    index: int

DigitAlias = TypedDict("DigitAlias", {"from": str, "to": str})

class DigitDefinitionRequired(TypedDict):
    digits: str
    prefixes: str
    alphabet: str

class DigitDefinition(DigitDefinitionRequired, total=False):
    aliases: list[DigitAlias]
    digit_escape: str | None

class GrandpreDefinition(TypedDict):
    rows: list[str]
    row_labels: str
    column_labels: str

def substitution_encode(
    text: str,
    source: str,
    *,
    target: str | None = None,
    keyword: str | None = None,
    unmapped: Literal["reject", "preserve"] = "reject",
) -> str: ...
def substitution_decode(
    text: str,
    source: str,
    *,
    target: str | None = None,
    keyword: str | None = None,
    unmapped: Literal["reject", "preserve"] = "reject",
) -> str: ...
def rot5(text: str) -> str: ...
def rot18(text: str) -> str: ...
def rot47(text: str) -> str: ...
def rot8000(text: str) -> str: ...
def homophonic_encode(
    text: str,
    definition: HomophonicDefinition,
    *,
    counters: list[HomophonicCounter] | None = None,
) -> list[str]: ...
def homophonic_decode(tokens: list[str], definition: HomophonicDefinition) -> str: ...
def straddling_encode(text: str, definition: DigitDefinition) -> str: ...
def straddling_decode(text: str, definition: DigitDefinition) -> str: ...
def monome_dinome_encode(text: str, definition: DigitDefinition) -> str: ...
def monome_dinome_decode(text: str, definition: DigitDefinition) -> str: ...
def tridigital_encode(text: str, definition: DigitDefinition) -> str: ...
def tridigital_decode(text: str, definition: DigitDefinition) -> list[list[str]]: ...
def grandpre_encode(
    text: str, definition: GrandpreDefinition
) -> list[list[int] | None]: ...
def grandpre_decode(
    tokens: list[list[int] | None], definition: GrandpreDefinition
) -> str: ...

# Exact Core historical profiles; bounds and canonicalization: docs/ciphers.md.
class HillPadding(TypedDict):
    symbol: str
    count: int

class PolyKeyParameters(TypedDict):
    key: str

class PolyInterruptedKeyParameters(TypedDict):
    key: str
    resets: list[int]

class PolyProgressiveKeyParameters(TypedDict):
    key: str
    increment: int

class PolyVigenereMultiplicativeParameters(TypedDict):
    key: str
    multiplier: int

def interrupted_key_encode(
    text: str, parameters: PolyInterruptedKeyParameters
) -> str: ...
def interrupted_key_decode(
    text: str, parameters: PolyInterruptedKeyParameters
) -> str: ...
def progressive_key_encode(
    text: str, parameters: PolyProgressiveKeyParameters
) -> str: ...
def progressive_key_decode(
    text: str, parameters: PolyProgressiveKeyParameters
) -> str: ...
def vigenere_multiplicative_encode(
    text: str, parameters: PolyVigenereMultiplicativeParameters
) -> str: ...
def vigenere_multiplicative_decode(
    text: str, parameters: PolyVigenereMultiplicativeParameters
) -> str: ...

class PolyGronsfeldParameters(TypedDict):
    key: str

class PolyTrithemiusParameters(TypedDict):
    start: int
    step: int

class PolyAutokeyParameters(TypedDict):
    seed: str
    feedback: Literal["plaintext", "ciphertext"]

class PolyRunningKeyParameters(TypedDict):
    key: str

class PolyGromarkParameters(TypedDict):
    alphabet: str
    primer: str

class PolyPeriodicGromarkParameters(TypedDict):
    keyword: str
    period: int

class PolySolitaireParameters(TypedDict):
    deck: list[int]
    passphrase: NotRequired[str]
    max_steps: NotRequired[int]

class PolyCondiParameters(TypedDict):
    alphabet: str
    shift: int

class PolyRagbabyParameters(TypedDict):
    alphabet: str
    merge_aliases: bool

class PolyChaocipherParameters(TypedDict):
    left: str
    right: str

class PolyNicodemusParameters(TypedDict):
    key: str
    depth: NotRequired[int]

class PolyQuagmireParameters(TypedDict):
    profile: Literal["i", "ii", "iii", "iv"]
    plain_alphabet: str
    cipher_alphabet: str
    indicator: str
    key: str

class PolyPhillipsParameters(TypedDict):
    square: str
    profile: Literal["rows", "columns", "rows_columns"]

class PolySquaresParameters(TypedDict):
    first: str
    second: str

class PolyPlayfairParameters(TypedDict):
    square: str
    filler: NotRequired[str]
    alternate: NotRequired[str]

class PolySeriatedPlayfairParameters(TypedDict):
    square: str
    period: int
    filler: NotRequired[str]
    alternate: NotRequired[str]

class PolyTriSquareParameters(TypedDict):
    first: str
    second: str
    middle: str
    rows: NotRequired[list[int]]
    columns: NotRequired[list[int]]

class PolyDigrafidParameters(TypedDict):
    horizontal: str
    vertical: str
    extra: str
    period: int
    padding: NotRequired[str | None]

class PolyHillParameters(TypedDict):
    alphabet: str
    dimension: int
    matrix: list[int]
    padding: NotRequired[HillPadding | None]

class PolyPreparedResult(TypedDict):
    text: str
    prepared: str
    fillers: list[int]

class PolyHillResult(TypedDict):
    text: str
    padding_count: int

class PolyDigrafidResult(TypedDict):
    text: str
    prepared: str
    padding_count: int

def gronsfeld_encode(text: str, parameters: PolyGronsfeldParameters) -> str: ...
def gronsfeld_decode(text: str, parameters: PolyGronsfeldParameters) -> str: ...
def trithemius_encode(text: str, parameters: PolyTrithemiusParameters) -> str: ...
def trithemius_decode(text: str, parameters: PolyTrithemiusParameters) -> str: ...
def autokey_encode(text: str, parameters: PolyAutokeyParameters) -> str: ...
def autokey_decode(text: str, parameters: PolyAutokeyParameters) -> str: ...
def running_key_encode(text: str, parameters: PolyRunningKeyParameters) -> str: ...
def running_key_decode(text: str, parameters: PolyRunningKeyParameters) -> str: ...
def porta(text: str, parameters: PolyKeyParameters) -> str: ...

class BellasoParameters(TypedDict):
    key: str
    unmapped: NotRequired[Literal["reject", "preserve"]]

def bellaso(text: str, parameters: BellasoParameters) -> str: ...
def gromark_encode(text: str, parameters: PolyGromarkParameters) -> str: ...
def gromark_decode(text: str, parameters: PolyGromarkParameters) -> str: ...
def periodic_gromark_encode(
    text: str, parameters: PolyPeriodicGromarkParameters
) -> str: ...
def periodic_gromark_decode(
    text: str, parameters: PolyPeriodicGromarkParameters
) -> str: ...
def solitaire_encode(text: str, parameters: PolySolitaireParameters) -> str: ...
def solitaire_decode(text: str, parameters: PolySolitaireParameters) -> str: ...
def slidefair_encode(text: str, parameters: PolyKeyParameters) -> str: ...
def slidefair_decode(text: str, parameters: PolyKeyParameters) -> str: ...
def portax(text: str, parameters: PolyKeyParameters) -> str: ...
def ragbaby_encode(text: str, parameters: PolyRagbabyParameters) -> str: ...
def ragbaby_decode(text: str, parameters: PolyRagbabyParameters) -> str: ...
def condi_encode(text: str, parameters: PolyCondiParameters) -> str: ...
def condi_decode(text: str, parameters: PolyCondiParameters) -> str: ...
def nicodemus_encode(text: str, parameters: PolyNicodemusParameters) -> str: ...
def nicodemus_decode(text: str, parameters: PolyNicodemusParameters) -> str: ...
def quagmire_encode(text: str, parameters: PolyQuagmireParameters) -> str: ...
def quagmire_decode(text: str, parameters: PolyQuagmireParameters) -> str: ...
def phillips_encode(text: str, parameters: PolyPhillipsParameters) -> str: ...
def phillips_decode(text: str, parameters: PolyPhillipsParameters) -> str: ...
def chaocipher_encode(text: str, parameters: PolyChaocipherParameters) -> str: ...
def chaocipher_decode(text: str, parameters: PolyChaocipherParameters) -> str: ...
def playfair_encode(
    text: str, parameters: PolyPlayfairParameters
) -> PolyPreparedResult: ...
def playfair_decode(
    text: str, parameters: PolyPlayfairParameters
) -> PolyPreparedResult: ...
def seriated_playfair_encode(
    text: str, parameters: PolySeriatedPlayfairParameters
) -> PolyPreparedResult: ...
def seriated_playfair_decode(
    text: str, parameters: PolySeriatedPlayfairParameters
) -> PolyPreparedResult: ...
def two_square_encode(text: str, parameters: PolySquaresParameters) -> str: ...
def two_square_decode(text: str, parameters: PolySquaresParameters) -> str: ...
def four_square_encode(text: str, parameters: PolySquaresParameters) -> str: ...
def four_square_decode(text: str, parameters: PolySquaresParameters) -> str: ...
def hill_encode(text: str, parameters: PolyHillParameters) -> PolyHillResult: ...
def hill_decode(text: str, parameters: PolyHillParameters) -> PolyHillResult: ...
def digrafid_encode(
    text: str, parameters: PolyDigrafidParameters
) -> PolyDigrafidResult: ...
def digrafid_decode(
    text: str, parameters: PolyDigrafidParameters
) -> PolyDigrafidResult: ...
def tri_square_encode(text: str, parameters: PolyTriSquareParameters) -> str: ...
def tri_square_decode(text: str, parameters: PolyTriSquareParameters) -> str: ...
def scytale_encode(text: str, parameters: dict[str, Any]) -> str: ...
def scytale_decode(text: str, parameters: dict[str, Any]) -> str: ...
def rail_fence_encode(text: str, parameters: dict[str, Any]) -> str: ...
def rail_fence_decode(text: str, parameters: dict[str, Any]) -> str: ...
def redefence_encode(text: str, parameters: dict[str, Any]) -> str: ...
def redefence_decode(text: str, parameters: dict[str, Any]) -> str: ...
def sequence_transposition_encode(text: str, parameters: dict[str, Any]) -> str: ...
def sequence_transposition_decode(text: str, parameters: dict[str, Any]) -> str: ...
def skip_encode(text: str, parameters: dict[str, Any]) -> str: ...
def skip_decode(text: str, parameters: dict[str, Any]) -> str: ...
def columnar_encode(text: str, parameters: dict[str, Any]) -> str: ...
def columnar_decode(text: str, parameters: dict[str, Any]) -> str: ...
def complete_columnar_encode(
    text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def complete_columnar_decode(
    text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def double_columnar_encode(text: str, parameters: dict[str, Any]) -> str: ...
def double_columnar_decode(text: str, parameters: dict[str, Any]) -> str: ...
def myszkowski_encode(text: str, parameters: dict[str, Any]) -> str: ...
def myszkowski_decode(text: str, parameters: dict[str, Any]) -> str: ...
def amsco_encode(text: str, parameters: dict[str, Any]) -> str: ...
def amsco_decode(text: str, parameters: dict[str, Any]) -> str: ...
def route_encode(text: str, parameters: dict[str, Any]) -> str: ...
def route_decode(text: str, parameters: dict[str, Any]) -> str: ...
def spiral_encode(text: str, parameters: dict[str, Any]) -> str: ...
def spiral_decode(text: str, parameters: dict[str, Any]) -> str: ...
def caesar_box_encode(text: str, parameters: dict[str, Any]) -> str: ...
def caesar_box_decode(text: str, parameters: dict[str, Any]) -> str: ...
def turning_grille_encode(text: str, parameters: dict[str, Any]) -> str: ...
def turning_grille_decode(text: str, parameters: dict[str, Any]) -> str: ...
def bifid_encode(text: str, parameters: dict[str, Any]) -> str: ...
def bifid_decode(text: str, parameters: dict[str, Any]) -> str: ...
def trifid_encode(text: str, parameters: dict[str, Any]) -> str: ...
def trifid_decode(text: str, parameters: dict[str, Any]) -> str: ...
def adfgx_encode(text: str, parameters: dict[str, Any]) -> str: ...
def adfgx_decode(text: str, parameters: dict[str, Any]) -> str: ...
def adfgvx_encode(text: str, parameters: dict[str, Any]) -> str: ...
def adfgvx_decode(text: str, parameters: dict[str, Any]) -> str: ...
def nihilist_substitution_encode(
    text: str, parameters: dict[str, Any]
) -> list[int]: ...
def nihilist_substitution_decode(
    tokens: list[int | str], parameters: dict[str, Any]
) -> str: ...
def bazeries_encode(text: str, parameters: dict[str, Any]) -> str: ...
def bazeries_decode(text: str, parameters: dict[str, Any]) -> str: ...
def fractionated_morse_encode(
    text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def fractionated_morse_decode(
    text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def morbit_encode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def morbit_decode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def pollux_encode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def pollux_decode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def cardan_extract(text: str, parameters: dict[str, Any]) -> str: ...
def cardan_overlay(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def aes_encrypt_blocks(data: bytes, key: bytes) -> bytes:
    """Raw AES-128/192/256 whole blocks, <=16 MiB; no padding or authentication."""
    ...

def aes_decrypt_blocks(data: bytes, key: bytes) -> bytes:
    """Raw AES inverse; bytes do not prove key correctness or authenticity."""
    ...

def enigma(text: str, parameters: dict[str, object]) -> dict[str, object]:
    """Enigma I/M3: required explicit settings; text/final_windows/optional trace.

    Default strict A-Z; text_mode='preserve' retains case and other scalars.
    Input <=1 MiB; trace <=65536 letters. Reset windows for reciprocal decoding.
    Historical only. Settings and traces are sensitive; no automatic capture.
    """
    ...

def book_encode(
    source: str, text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def keyboard_coordinates_encode(
    text: str, parameters: dict[str, Any]
) -> dict[str, Any]: ...
def keyboard_coordinates_decode(
    coordinates: list[list[int | str]], parameters: dict[str, Any]
) -> dict[str, Any]: ...
def keyboard_shift_encode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def keyboard_shift_decode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def symbol_tokens_decode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def symbol_tokens_encode(text: str, parameters: dict[str, Any]) -> dict[str, Any]: ...
def language_score(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def period_estimate(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def caesar_rank(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def affine_rank(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def vigenere_rank(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def transposition_rank(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def homophonic_inspect(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def crib_verify(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]: ...
def frequency_compare(
    request: dict[str, object], *, cancellation: CancellationToken | None = None
) -> dict[str, object]:
    """Compare caller frequencies with explicit counts/proportions; no p-value."""
    ...

def stream_transform(
    profile: str, data: bytes, key: bytes, nonce: bytes, counter: int
) -> bytes:
    """Apply an exact raw stream profile (<=16 MiB), with required nonce/counter.

    Profiles: chacha8, chacha12, chacha20, xchacha20, salsa20_12, salsa20.
    Key is exactly 32 bytes. Nonce is 12/24/8 bytes; counter is a block index.
    Unauthenticated, malleable; caller prevents nonce/position reuse.
    Reduced rounds are experimental; Salsa backend lacks an independent audit.
    """
    ...

def vernam(data: bytes, pad: bytes) -> bytes:
    """Educational exact-pad XOR (<=16 MiB each), including empty/empty.

    Uniform independent one-use pad delivery is external; no pad management.
    """
    ...

class CodebookEntry(TypedDict):
    alternatives: list[str]
    codes: list[str]

class CodebookDefinition(TypedDict):
    entries: list[CodebookEntry]
    null_codes: NotRequired[list[str]]
    cancel_codes: NotRequired[list[str]]

class SyllabaryDefinition(TypedDict):
    cells: list[str]
    row_labels: str
    column_labels: str

class CodebookUnit(TypedDict):
    token_index: int
    alternatives: list[str]

class CodebookReport(TypedDict):
    units: list[CodebookUnit]
    null_indices: list[int]
    canceled_indices: list[int]
    cancel_indices: list[int]

def malespin(text: str) -> str: ...
def vatsyayana(
    text: str, pairs: list[tuple[str, str]] | list[list[str]], unmapped: str = "reject"
) -> str: ...
def nomenclator_encode(
    units: list[str], definition: CodebookDefinition
) -> list[str]: ...
def nomenclator_decode(
    tokens: list[str], definition: CodebookDefinition
) -> CodebookReport: ...
def syllabary_encode(
    units: list[str], definition: SyllabaryDefinition
) -> list[str]: ...
def syllabary_decode(
    tokens: list[str], definition: SyllabaryDefinition
) -> CodebookReport: ...
def great_cipher_encode(units: list[str]) -> list[str]: ...
def great_cipher_decode(tokens: list[str]) -> CodebookReport: ...
