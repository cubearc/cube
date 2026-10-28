use cube_core::{
    errors::ErrorCode,
    operations::{
        self as o, Descriptor, Discovery, DiscoveryFormat as F, DiscoveryRequest as R,
        metadata::{Classification, Scope},
    },
};
use std::collections::BTreeSet;
#[test]
fn old_literals_and_strict_legacy_consumers_keep_their_original_contract() {
    const LEGACY: Descriptor = Descriptor {
        id: "legacy.custom",
        category: "custom",
        description: "A client-created descriptor.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: "custom",
        example: "custom",
    };
    let view = Discovery {
        descriptor: &LEGACY,
        pipeline: None,
    };
    let value = serde_json::to_value(view).unwrap();
    assert!(value.get("metadata").is_none());
    assert!(LEGACY.metadata().is_none());
    // A strict legacy client rejects extra keys: default discovery must remain exact.
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    struct LegacyConsumer {
        id: String,
        category: String,
        description: String,
        input_type: String,
        output_type: String,
        parameters: Vec<serde_json::Value>,
        deterministic: bool,
        reversible: Option<bool>,
        command: String,
        example: String,
        pipeline: Option<serde_json::Value>,
    }
    let old: LegacyConsumer = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap(), value);
    for d in o::catalog() {
        let old = serde_json::to_value(d.discovery()).unwrap();
        let _: LegacyConsumer = serde_json::from_value(old.clone()).unwrap();
        assert!(old.get("metadata").is_none());
        let mut extended = serde_json::to_value(d.profiled_discovery()).unwrap();
        let metadata = extended.as_object_mut().unwrap().remove("metadata");
        assert_eq!(metadata.is_some(), d.metadata().is_some());
        assert_eq!(extended, old);
        let copy = *d;
        assert_eq!(
            serde_json::to_value(copy.profiled_discovery()).unwrap(),
            serde_json::to_value(d.profiled_discovery()).unwrap()
        );
    }
    let real = o::require("hash.sha256").unwrap();
    let changed = Descriptor {
        description: "Different semantics behind a reused name",
        ..*real
    };
    assert!(changed.metadata().is_none());
}
#[test]
fn owning_declarations_cover_the_measured_crypto_and_puzzle_scope() {
    let mut ids = BTreeSet::new();
    let mut annotated = 0;
    for d in o::catalog() {
        assert!(ids.insert(d.id));
        let expected = matches!(
            d.category,
            "cryptanalysis"
                | "ciphers"
                | "encoding"
                | "hashing"
                | "binary"
                | "blockchain"
                | "symbols"
        ) || d.id.starts_with("text.utf8.")
            || d.id.starts_with("text.keyboard.coordinates.")
            || d.id.starts_with("text.keyboard.shift.");
        assert_eq!(d.metadata().is_some(), expected, "{}", d.id);
        if let Some(m) = d.metadata() {
            annotated += 1;
            assert_eq!(m.schema_version, 1);
            assert!(!m.canonical_name.is_empty() && m.canonical_name.len() <= 128);
            assert!(!m.family.is_empty());
            assert_eq!(m.profile.variant, d.description);
            for text in [
                m.security.limitations,
                m.profile.alphabet,
                m.profile.key,
                m.profile.key_advancement,
                m.resource_notes,
                m.rust_api,
                m.python_api,
            ] {
                assert!(!text.is_empty() && text.len() <= 4096, "{}", d.id);
            }
            assert!(m.aliases.len() <= 32 && !m.references.is_empty());
            for a in m.aliases {
                assert!(!a.name.is_empty() && !a.usage.is_empty());
                assert!(
                    o::describe(a.name).is_none(),
                    "aliases never become lookup IDs"
                );
            }
            for p in m.security.sensitive_parameters {
                assert!(d.parameters.iter().any(|v| v.name == *p));
            }
            if m.security.private_input || m.security.private_output {
                assert!(d.discovery().pipeline.is_none());
            }
            for b in m.resources {
                assert!(b.maximum > 0 && !b.unit.is_empty());
                if b.scope == Scope::Pipeline {
                    assert!(d.discovery().pipeline.is_some());
                }
            }
        }
    }
    assert_eq!(annotated, 191);
    assert_eq!(ids.len(), 307);
    assert_eq!(cube_core::pipeline::supported_adapters().len(), 53);
}
#[test]
fn aliases_security_variants_and_surface_limits_are_precise() {
    let affine = o::require("cipher.affine.encode")
        .unwrap()
        .metadata()
        .unwrap();
    assert!(
        affine
            .aliases
            .iter()
            .any(|a| a.name == "Multiplicative" && a.usage.contains("b=0"))
    );
    let variant = o::require("cipher.vigenere.decode")
        .unwrap()
        .metadata()
        .unwrap();
    assert!(
        variant
            .aliases
            .iter()
            .any(|a| a.name == "Variant Beaufort" && a.usage.contains("vigenere_decode"))
    );
    assert_eq!(
        o::require("hash.md5")
            .unwrap()
            .metadata()
            .unwrap()
            .security
            .classification,
        Classification::LegacyDigest
    );
    assert_eq!(
        o::require("hash.crc32")
            .unwrap()
            .metadata()
            .unwrap()
            .security
            .classification,
        Classification::Checksum
    );
    let sha = o::require("hash.sha256").unwrap().metadata().unwrap();
    assert_eq!(sha.profile.output_bits, Some(256));
    assert_eq!(sha.profile.block_bits, Some(512));
    assert!(
        !sha.resources
            .iter()
            .any(|b| b.name == "input_bytes" && b.scope == Scope::Core)
    );
    let caesar = o::require("cipher.caesar.encode")
        .unwrap()
        .metadata()
        .unwrap();
    assert!(
        caesar
            .resources
            .iter()
            .any(|b| b.scope == Scope::Cli && b.maximum == 1048576)
    );
    assert!(caesar.resource_notes.contains("No enforced direct"));
    let keccak = o::require("hash.keccak256").unwrap().metadata().unwrap();
    assert_eq!(keccak.profile.block_bits, Some(1088));
    assert!(keccak.profile.variant.contains("not standardized SHA3"));
    let private = o::require("blockchain.private.export").unwrap();
    assert!(private.metadata().unwrap().security.private_output);
    assert!(private.discovery().pipeline.is_none());
    assert!(o::require("cipher.aes.encrypt").is_err());
    assert!(o::require("Variant Beaufort").is_err());
}
#[test]
fn bounded_discovery_has_exact_byte_boundaries_and_never_returns_partial_json() {
    for r in [
        R::List { category: None },
        R::List {
            category: Some("ciphers"),
        },
        R::Describe {
            id: "cipher.polybius.encode",
        },
        R::Describe {
            id: "blockchain.private.export",
        },
        R::Describe { id: "image.decode" },
    ] {
        for f in [
            F::DirectJson { metadata: false },
            F::DirectJson { metadata: true },
            F::CliJson { metadata: false },
            F::CliJson { metadata: true },
            F::Text,
        ] {
            let result = o::discovery_document(r, f, o::MAX_DISCOVERY_BYTES).unwrap();
            assert!(result.len() < o::MAX_DISCOVERY_BYTES);
            assert_eq!(o::discovery_document(r, f, result.len()).unwrap(), result);
            assert_eq!(
                o::discovery_document(r, f, result.len() - 1)
                    .unwrap_err()
                    .code,
                ErrorCode::ResourceLimit
            );
            if !matches!(f, F::Text) {
                let _: serde_json::Value = serde_json::from_str(&result).unwrap();
            }
        }
    }
    for r in [
        R::Describe {
            id: "missing.operation",
        },
        R::List {
            category: Some("missing"),
        },
    ] {
        assert_eq!(
            o::discovery_document(r, F::DirectJson { metadata: true }, o::MAX_DISCOVERY_BYTES)
                .unwrap_err()
                .code,
            ErrorCode::InvalidParameter
        );
    }
    let text = "x".repeat(256);
    assert_eq!(
        o::discovery_document(R::Describe { id: &text }, F::Text, 10000)
            .unwrap_err()
            .code,
        ErrorCode::InvalidParameter
    );
    let text = "x".repeat(257);
    assert_eq!(
        o::discovery_document(R::Describe { id: &text }, F::Text, 10000)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        o::discovery_document(
            R::List { category: None },
            F::Text,
            o::MAX_DISCOVERY_BYTES + 1
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        o::discovery_document(R::List { category: None }, F::Text, 0)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
