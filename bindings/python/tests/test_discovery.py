"""Audit catalog coverage against the compiled SDK and shipped public stubs."""

import ast
from pathlib import Path

import cube
import pytest
from cube import _native

# Compatibility fixture for existing IDs and their actual SDK entry points.
# This is test evidence, never a runtime catalog or execution registry.
SDK_BINDINGS = {
    "number.digital_root": "digital_root",
    **{
        f"number.{name}": name
        for name in ("gcd", "extended_gcd", "mod_inverse", "mod_pow")
    },
    "text.unicode.inspect": "inspect_unicode",
    "text.unicode.compare": "compare_unicode",
    "text.word.lengths": "word_lengths",
    "text.word.values": "word_values",
    "text.codepoints": "text_codepoints",
    "text.word.extract": "extract_word_characters",
    "text.skip.keyed": "keyed_skip",
    "text.josephus.extract": "josephus_extract",
    "number.alphabet.convert": "convert_alphabet",
    **{
        f"cipher.{name}.{direction}": f"{name}_{direction}"
        for name in ("bacon", "polybius")
        for direction in ("encode", "decode")
    },
    **{
        f"symbol.keypad.{direction}": f"keypad_{direction}"
        for direction in ("encode", "decode")
    },
    "text.keyboard.remap": "keyboard_remap",
    "text.class.map": "map_character_classes",
    "blockchain.private.export": "export_private_key",
    "hash.ripemd160": "ripemd160",
    "hash.keccak256": "keccak256",
    "blockchain.candidates.validate": "validate_key_candidates",
    "blockchain.ethereum.derive": "derive_ethereum_address",
    "blockchain.bitcoin.derive": "derive_bitcoin_address",
    "blockchain.wif.validate": "validate_wif",
    "blockchain.scalar.validate": "validate_private_scalar",
    "encoding.base58check.encode": "base58check_encode",
    "encoding.base58check.decode": "base58check_decode",
    **{
        f"encoding.{name}.{direction}": f"{name}_{direction}"
        for name in ("base32", "base58", "base64", "hex")
        for direction in ("encode", "decode")
    },
    **{
        f"cipher.{name}.{direction}": f"{name}_{direction}"
        for name in ("caesar", "vigenere", "affine")
        for direction in ("encode", "decode")
    },
    **{f"cipher.{name}": name for name in ("rot13", "atbash", "beaufort")},
    **{
        f"hash.{name}": name
        for name in (
            "crc32",
            "sha256",
            "sha512",
            "sha1",
            "md5",
            "crc32c",
            "crc16_ibm3740",
        )
    },
    **{
        f"symbol.{name}.{direction}": f"{name}_{direction}"
        for name in ("morse", "braille", "a1z26")
        for direction in ("encode", "decode")
    },
    "cipher.book.decode": "book_decode",
    "symbol.semaphore.encode": "semaphore_encode",
    "symbol.semaphore.decode": "semaphore_decode",
    "symbol.clock.directions": "clock_directions",
    "binary.xor": "xor",
    "binary.and": "and_bytes",
    "binary.or": "or_bytes",
    "binary.not": "not_bytes",
    "binary.reverse": "reverse_bytes",
    "binary.shift_left": "shift_left",
    "binary.shift_right": "shift_right",
    "binary.swap_endian": "swap_endian",
    "file.appended.detect": "appended_data",
    "file.binary.compare": "compare_files",
    "file.exif.inspect": "inspect_exif",
    "file.hex.inspect": "file_hex",
    "file.info.inspect": "inspect_file",
    "file.range.extract": "extract_file_range",
    "file.signature.detect": "file_signatures",
    "file.signature.scan": "scan_file",
    "file.strings.extract": "file_strings",
    "file.zip.validate": "validate_zip",
    "file.carve": "carve_file",
    "image.bit_plane.extract": "Image.bit_plane",
    "image.channel.extract": "Image.channel",
    "image.compare": "Image.compare",
    "image.pairs": "image_pairs",
    "image.bitwise": "Image.bitwise",
    "image.crop": "Image.crop",
    "image.decode": "Image.decode",
    "image.colors.extract": "Image.exact_colors",
    "image.csv.export": "Image.to_csv",
    "image.csv.import": "Image.from_csv",
    "image.colors.export": "Image.color_layers",
    "image.raw.interpret": "Image.from_raw",
    "image.geometry.inspect": "Image.inspect_layout",
    "image.qr.decode": "Image.qr",
    "image.encode": "Image.encode_png",
    "image.grayscale": "Image.grayscale",
    "image.histogram": "Image.histogram",
    "image.invert": "Image.invert",
    "image.lsb.extract": "Image.lsb",
    "image.mirror": "Image.mirror",
    "image.palette.expand": "Image.expand_palette",
    "image.pixels.export": "Image.pixels",
    "image.pixels.read": "Image.pixel",
    "image.png.inspect": "png_chunks",
    "image.reorder": "Image.reorder",
    "image.rotate": "Image.rotate",
    "text.extract": "extract_characters",
    "text.select": "select_text",
    "text.transform.masked": "transform_masked",
    "text.combinations": "string_combinations",
    "text.grid.factors": "grid_factors",
    "text.grid.render": "text_grid",
    "text.statistics": "text_statistics",
    "text.normalize": "normalize_text",
    "text.reverse": "reverse_text",
    "text.utf8.decode": "utf8_decode",
    "text.utf8.encode": "utf8_encode",
    "number.constant_digits": "constant_digits",
    "number.base.convert": "convert_base",
    "audio.spectrogram": "wave_spectrogram",
    "audio.bits.extract": "wave_bits",
    "audio.reverse": "reverse_wave",
    "audio.dtmf.detect": "detect_dtmf",
    "audio.morse.detect": "detect_morse_intervals",
    "audio.morse.transcribe": "transcribe_morse",
    "audio.metadata.inspect": "inspect_audio_metadata",
    "web.html.asset": "extract_html_asset",
    "web.har.asset": "extract_har_asset",
    "web.html.inspect": "inspect_html",
    "web.har.inspect": "inspect_har",
    "video.container.inspect": "inspect_video",
    "video.frames.extract": "extract_video_frames",
    "video.audio.extract": "extract_video_audio",
    "video.frames.compare": "compare_video_frames",
    "audio.spectrum": "wave_spectrum",
    "audio.wave.inspect": "inspect_wave",
    "audio.pcm.extract": "wave_samples",
    "audio.waveform.envelope": "wave_envelope",
    "pipeline.execute": "execute_pipeline",
    "pipeline.validate": "validate_pipeline",
    "recipe.check": "check_recipe",
    "recipe.execute": "execute_recipe",
    "recipe.validate": "validate_recipe",
    "investigation.triage": "triage_input",
    "investigation.session.record": "InvestigationSession.execute_pipeline",
    "investigation.report": "InvestigationSession.report",
    "investigation.replay": "InvestigationSession.replay",
    "investigation.profile.plan": "plan_investigation",
    "investigation.profile.run": "investigate",
}


def test_catalog_ids_have_native_exports_and_public_stubs():
    descriptors = cube.operations()
    assert {d["id"] for d in descriptors} == SDK_BINDINGS.keys()
    stub = ast.parse(Path(cube.__file__).with_name("_native.pyi").read_text())
    for entry in SDK_BINDINGS.values():
        parts = entry.split(".")
        assert parts[0] in cube.__all__
        public, native, declarations = cube, _native, stub.body
        for part in parts:
            public, native = getattr(public, part), getattr(native, part)
            declaration = next(
                node for node in declarations if getattr(node, "name", None) == part
            )
            declarations = getattr(declaration, "body", [])
        assert callable(public) and callable(native)
        assert public is native


def test_discovery_pipeline_support_matches_real_adapters_and_preflight():
    adapters = {a["operation"]: a for a in cube.pipeline_adapters()["adapters"]}
    for descriptor in cube.operations():
        operation = descriptor["id"]
        assert cube.describe_operation(operation) == descriptor
        filtered = cube.operations(descriptor["category"])
        assert descriptor in filtered
        if operation in adapters:
            adapter = adapters[operation]
            assert descriptor["pipeline"] == {"version": adapter["version"]}
            assert {k: v for k, v in descriptor.items() if k != "pipeline"} == adapter[
                "descriptor"
            ]
        else:
            assert descriptor["pipeline"] is None
            with pytest.raises(
                ValueError, match="no permitted bounded pipeline adapter"
            ):
                cube.validate_pipeline(
                    [{"operation": operation, "version": 1}], "bytes"
                )
    assert cube.describe_operation("encoding.hex.encode")["pipeline"] == {"version": 1}
    assert cube.describe_operation("file.info.inspect")["pipeline"] is None
    with pytest.raises(ValueError, match="unknown operation ID"):
        cube.describe_operation("missing.operation")
