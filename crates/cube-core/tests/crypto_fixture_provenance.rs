//! Offline checked static corpora: no runtime vector engine or network input.
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeSet, io::Read, path::Path};
const MAX_FILE: usize = 16 * 1024 * 1024;
const MAX_CASES: usize = 65536;
const MAX_FIELD: usize = 512 * 1024;
const MAX_CASE: usize = 1024 * 1024;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format_version: u32,
    total_cases: usize,
    corpora: Vec<Entry>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    corpus: Corpus,
    formats: Vec<Format>,
    sha256: String,
    cases: usize,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum Format {
    NistSpec,
    NistCavp,
    BackendKat,
    IndependentCapture,
    Cctv,
    Wycheproof,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum Corpus {
    AesModes,
    KeyWrap,
    KeyWrapInvalid,
    Aead,
    Siv,
    Fpe,
    Openssl,
    OpensslWrongPad,
    Age,
    Wycheproof,
}
impl Corpus {
    fn file(self) -> &'static str {
        match self {
            Self::AesModes => "aes_modes/vectors.json",
            Self::KeyWrap => "key_wrap/vectors.json",
            Self::KeyWrapInvalid => "key_wrap/invalid_vectors.json",
            Self::Aead => "modern_aead/vectors.json",
            Self::Siv => "modern_aead/siv_vectors.json",
            Self::Fpe => "fpe/vectors.json",
            Self::Openssl => "openssl/vectors.json",
            Self::OpensslWrongPad => "openssl/valid_pad_wrong_password.json",
            Self::Age => "age/vectors.json",
            Self::Wycheproof => "modern_aead/wycheproof_vectors.json",
        }
    }
}
fn manifest(document: &[u8]) -> Result<Manifest, String> {
    if document.len() > MAX_FILE {
        return Err("manifest limit".into());
    }
    let m: Manifest = serde_json::from_slice(document).map_err(|_| "manifest schema")?;
    if m.format_version != 1 || m.corpora.len() != 10 || m.total_cases > MAX_CASES {
        return Err("manifest limits".into());
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for e in &m.corpora {
        if !seen.insert(e.corpus)
            || e.cases == 0
            || e.cases > MAX_CASES
            || e.formats.is_empty()
            || e.formats.len() > 3
            || e.formats.iter().copied().collect::<BTreeSet<_>>().len() != e.formats.len()
            || e.sha256.len() != 64
            || e.sha256
                .bytes()
                .any(|b| !b.is_ascii_hexdigit() || b.is_ascii_uppercase())
        {
            return Err("entry limits".into());
        }
        total += e.cases;
        if total > MAX_CASES {
            return Err("total case limit".into());
        }
    }
    if total != m.total_cases {
        return Err("case count mismatch".into());
    }
    Ok(m)
}
fn shape(value: &Value, depth: usize) -> Result<(), String> {
    if depth > 8 {
        return Err("nesting limit".into());
    }
    match value {
        Value::String(s) if s.len() > MAX_FIELD => Err("field limit".into()),
        Value::Array(a) => {
            if a.len() > MAX_CASES {
                return Err("item limit".into());
            }
            for v in a {
                shape(v, depth + 1)?;
            }
            Ok(())
        }
        Value::Object(o) => {
            if o.len() > 64 {
                return Err("field count limit".into());
            }
            for (k, v) in o {
                if k.len() > 128 {
                    return Err("field name limit".into());
                }
                shape(v, depth + 1)?;
            }
            Ok(())
        }
        Value::Number(n) if !n.is_u64() => Err("exact unsigned integer required".into()),
        _ => Ok(()),
    }
}
fn checked(bytes: &[u8], entry: &Entry) -> Result<(), String> {
    if bytes.len() > MAX_FILE {
        return Err("fixture file limit".into());
    }
    // Approved byte identity precedes interpretation. Modified/duplicate/unknown
    // fields cannot silently redefine a reviewed static oracle.
    if cube_core::hashing::sha256(bytes) != entry.sha256 {
        return Err("fixture hash mismatch".into());
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "fixture JSON")?;
    let cases = if let Value::Array(a) = &value {
        a.as_slice()
    } else {
        std::slice::from_ref(&value)
    };
    if cases.len() != entry.cases || cases.len() > MAX_CASES {
        return Err("fixture case count".into());
    }
    for case in cases {
        shape(case, 0)?;
        if serde_json::to_vec(case).map_err(|_| "case JSON")?.len() > MAX_CASE {
            return Err("case byte limit".into());
        }
    }
    Ok(())
}
#[test]
fn every_checked_corpus_has_exact_provenance_validity_and_bounded_offline_shape() {
    let m = manifest(include_bytes!("fixtures/crypto/provenance.json")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ciphers");
    for entry in &m.corpora {
        let mut bytes = Vec::new();
        std::fs::File::open(root.join(entry.corpus.file()))
            .unwrap()
            .take(MAX_FILE as u64 + 1)
            .read_to_end(&mut bytes)
            .unwrap();
        checked(&bytes, entry).unwrap_or_else(|e| panic!("{}: {e}", entry.corpus.file()));
    }
    assert_eq!(m.total_cases, 585);
}
#[test]
fn manifest_refuses_duplicate_unknown_algorithms_and_excessive_counts() {
    let source = include_str!("fixtures/crypto/provenance.json");
    for changed in [
        source.replacen(
            "\"format_version\": 1",
            "\"format_version\": 1, \"format_version\": 1",
            1,
        ),
        source.replacen(
            "\"format_version\": 1",
            "\"format_version\": 1, \"unknown\": true",
            1,
        ),
        source.replacen("\"aes_modes\"", "\"unknown_algorithm\"", 1),
        source.replacen("\"nist_cavp\"", "\"unknown_format\"", 1),
        source.replacen("\"cases\": 71", "\"cases\": 65537", 1),
    ] {
        assert_ne!(changed, source);
        assert!(manifest(changed.as_bytes()).is_err());
    }
    assert!(manifest(&vec![b' '; MAX_FILE + 1]).is_err());
}
#[test]
fn changed_or_oversized_oracles_fail_before_interpretation() {
    let m = manifest(include_bytes!("fixtures/crypto/provenance.json")).unwrap();
    let e = &m.corpora[0];
    for modified in [
        br#"[{"algorithm":"unknown"}]"#.as_slice(),
        br#"[{"key":"first","key":"second"}]"#.as_slice(),
    ] {
        assert_eq!(checked(modified, e), Err("fixture hash mismatch".into()));
    }
    assert_eq!(
        checked(&vec![0; MAX_FILE + 1], e),
        Err("fixture file limit".into())
    );
    assert!(shape(&Value::String("x".repeat(MAX_FIELD + 1)), 0).is_err());
    assert!(shape(&serde_json::json!([[[[[[[[[0]]]]]]]]]), 0).is_err());
}
