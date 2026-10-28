"""Audit catalog coverage against the compiled SDK and shipped public stubs."""

import ast
from pathlib import Path

import cube
import pytest
from cube import _native

# Compatibility fixture for existing IDs and their actual SDK entry points.
# This is test evidence, never a runtime catalog or execution registry.
SDK_BINDINGS = {
    "cipher.enigma": "enigma",
    "cipher.book.encode": "book_encode",
    "text.keyboard.coordinates.encode": "keyboard_coordinates_encode",
    "text.keyboard.coordinates.decode": "keyboard_coordinates_decode",
    "text.keyboard.shift.encode": "keyboard_shift_encode",
    "text.keyboard.shift.decode": "keyboard_shift_decode",
    "symbol.tokens.decode": "symbol_tokens_decode",
    "symbol.tokens.encode": "symbol_tokens_encode",
    "cipher.scytale.encode": "scytale_encode",
    "cipher.scytale.decode": "scytale_decode",
    "cipher.rail_fence.encode": "rail_fence_encode",
    "cipher.rail_fence.decode": "rail_fence_decode",
    "cipher.redefence.encode": "redefence_encode",
    "cipher.redefence.decode": "redefence_decode",
    "cipher.sequence_transposition.encode": "sequence_transposition_encode",
    "cipher.sequence_transposition.decode": "sequence_transposition_decode",
    "cipher.skip.encode": "skip_encode",
    "cipher.skip.decode": "skip_decode",
    "cipher.columnar.encode": "columnar_encode",
    "cipher.columnar.decode": "columnar_decode",
    "cipher.complete_columnar.encode": "complete_columnar_encode",
    "cipher.complete_columnar.decode": "complete_columnar_decode",
    "cipher.double_columnar.encode": "double_columnar_encode",
    "cipher.double_columnar.decode": "double_columnar_decode",
    "cipher.myszkowski.encode": "myszkowski_encode",
    "cipher.myszkowski.decode": "myszkowski_decode",
    "cipher.amsco.encode": "amsco_encode",
    "cipher.amsco.decode": "amsco_decode",
    "cipher.route.encode": "route_encode",
    "cipher.route.decode": "route_decode",
    "cipher.spiral.encode": "spiral_encode",
    "cipher.spiral.decode": "spiral_decode",
    "cipher.caesar_box.encode": "caesar_box_encode",
    "cipher.caesar_box.decode": "caesar_box_decode",
    "cipher.turning_grille.encode": "turning_grille_encode",
    "cipher.turning_grille.decode": "turning_grille_decode",
    "cipher.bifid.encode": "bifid_encode",
    "cipher.bifid.decode": "bifid_decode",
    "cipher.trifid.encode": "trifid_encode",
    "cipher.trifid.decode": "trifid_decode",
    "cipher.adfgx.encode": "adfgx_encode",
    "cipher.adfgx.decode": "adfgx_decode",
    "cipher.adfgvx.encode": "adfgvx_encode",
    "cipher.adfgvx.decode": "adfgvx_decode",
    "cipher.nihilist_substitution.encode": "nihilist_substitution_encode",
    "cipher.nihilist_substitution.decode": "nihilist_substitution_decode",
    "cipher.bazeries.encode": "bazeries_encode",
    "cipher.bazeries.decode": "bazeries_decode",
    "cipher.fractionated_morse.encode": "fractionated_morse_encode",
    "cipher.fractionated_morse.decode": "fractionated_morse_decode",
    "cipher.morbit.encode": "morbit_encode",
    "cipher.morbit.decode": "morbit_decode",
    "cipher.pollux.encode": "pollux_encode",
    "cipher.pollux.decode": "pollux_decode",
    "cipher.cardan.extract": "cardan_extract",
    "cipher.cardan.overlay": "cardan_overlay",
    "cipher.gronsfeld.encode": "gronsfeld_encode",
    "cipher.gronsfeld.decode": "gronsfeld_decode",
    "cipher.trithemius.encode": "trithemius_encode",
    "cipher.trithemius.decode": "trithemius_decode",
    "cipher.autokey.encode": "autokey_encode",
    "cipher.autokey.decode": "autokey_decode",
    "cipher.running_key.encode": "running_key_encode",
    "cipher.running_key.decode": "running_key_decode",
    "cipher.porta": "porta",
    "cipher.gromark.encode": "gromark_encode",
    "cipher.gromark.decode": "gromark_decode",
    "cipher.periodic_gromark.encode": "periodic_gromark_encode",
    "cipher.periodic_gromark.decode": "periodic_gromark_decode",
    "cipher.solitaire.encode": "solitaire_encode",
    "cipher.solitaire.decode": "solitaire_decode",
    "cipher.slidefair.encode": "slidefair_encode",
    "cipher.slidefair.decode": "slidefair_decode",
    "cipher.portax": "portax",
    "cipher.ragbaby.encode": "ragbaby_encode",
    "cipher.ragbaby.decode": "ragbaby_decode",
    "cipher.condi.encode": "condi_encode",
    "cipher.condi.decode": "condi_decode",
    "cipher.nicodemus.encode": "nicodemus_encode",
    "cipher.nicodemus.decode": "nicodemus_decode",
    "cipher.quagmire.encode": "quagmire_encode",
    "cipher.quagmire.decode": "quagmire_decode",
    "cipher.phillips.encode": "phillips_encode",
    "cipher.phillips.decode": "phillips_decode",
    "cipher.chaocipher.encode": "chaocipher_encode",
    "cipher.chaocipher.decode": "chaocipher_decode",
    "cipher.playfair.encode": "playfair_encode",
    "cipher.playfair.decode": "playfair_decode",
    "cipher.seriated_playfair.encode": "seriated_playfair_encode",
    "cipher.seriated_playfair.decode": "seriated_playfair_decode",
    "cipher.two_square.encode": "two_square_encode",
    "cipher.two_square.decode": "two_square_decode",
    "cipher.four_square.encode": "four_square_encode",
    "cipher.four_square.decode": "four_square_decode",
    "cipher.hill.encode": "hill_encode",
    "cipher.hill.decode": "hill_decode",
    "cipher.digrafid.encode": "digrafid_encode",
    "cipher.digrafid.decode": "digrafid_decode",
    "cipher.tri_square.encode": "tri_square_encode",
    "cipher.tri_square.decode": "tri_square_decode",
    "cipher.substitution.encode": "substitution_encode",
    "cipher.substitution.decode": "substitution_decode",
    "cipher.rot5": "rot5",
    "cipher.rot18": "rot18",
    "cipher.rot47": "rot47",
    "cipher.homophonic.encode": "homophonic_encode",
    "cipher.homophonic.decode": "homophonic_decode",
    "cipher.straddling.encode": "straddling_encode",
    "cipher.straddling.decode": "straddling_decode",
    "cipher.monome_dinome.encode": "monome_dinome_encode",
    "cipher.monome_dinome.decode": "monome_dinome_decode",
    "cipher.tridigital.encode": "tridigital_encode",
    "cipher.tridigital.decode": "tridigital_decode",
    "cipher.grandpre.encode": "grandpre_encode",
    "cipher.grandpre.decode": "grandpre_decode",
    "text.anagram.search": "anagram_search",
    "text.word_ladder.validate": "word_ladder_validate",
    "text.word_ladder.shortest": "word_ladder_shortest",
    "probability.finite.analyze": "finite_probability",
    "probability.dice.sum": "dice_sum",
    "probability.dice.compare": "dice_compare",
    "image.spatial.autocorrelation": "Image.autocorrelation",
    "number.permutations": "permutations",
    "number.binomial": "binomial",
    "number.multinomial": "multinomial",
    "number.pascal": "pascal_triangle",
    "text.multiset.compare": "multiset_compare",
    "text.multiset.subtract": "multiset_subtract",
    "logic.boolean.evaluate": "boolean_evaluate",
    "logic.boolean.analyze": "boolean_analyze",
    "graph.components": "graph_components",
    "graph.shortest_path": "graph_shortest_path",
    "graph.cycle": "graph_cycle",
    "graph.euler_trail": "graph_euler_trail",
    **{
        f"binary.{name}": name
        for name in (
            "rotate_left",
            "rotate_right",
            "xor_exact",
            "add_wrapping",
            "sub_wrapping",
        )
    },
    **{
        f"esolang.brainfuck.{name}": f"brainfuck_{name}"
        for name in ("execute", "report", "compare")
    },
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
