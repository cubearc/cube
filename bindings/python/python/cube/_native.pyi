"""Native operations. Invalid encoded input or an empty XOR key raises ValueError."""

from os import PathLike
from typing import Any, BinaryIO

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
def operations(category: str | None = None) -> list[dict[str, Any]]:
    """Sorted descriptors; pipeline is {\"version\": int} or None without an adapter."""
    ...

def describe_operation(id: str) -> dict[str, Any]:
    """One descriptor with pipeline availability; unknown IDs raise ValueError."""
    ...

def base32_encode(data: bytes) -> str: ...
def base32_decode(text: str) -> bytes: ...
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
    steps: list[dict[str, Any]],
    input_type: str,
    *,
    limits: dict[str, Any] | None = None,
) -> dict[str, Any]: ...
def execute_pipeline(
    steps: list[dict[str, Any]],
    input: bytes | str | dict[str, Any],
    *,
    limits: dict[str, Any] | None = None,
    capture: list[int] | None = None,
    cancellation: CancellationToken | None = None,
) -> dict[str, Any]: ...
def validate_recipe(document: str) -> dict[str, Any]: ...
def execute_recipe(
    document: str,
    inputs: dict[str, bytes | str | dict[str, Any]],
    *,
    capture: list[int] | None = None,
    cancellation: CancellationToken | None = None,
) -> dict[str, Any]: ...
def check_recipe(
    document: str,
    inputs: dict[str, bytes | str | dict[str, Any]],
    expected: bytes | str | dict[str, Any],
) -> dict[str, Any]: ...
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
        steps: list[dict[str, Any]],
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
