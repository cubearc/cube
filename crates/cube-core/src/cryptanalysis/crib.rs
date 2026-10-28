//! Exact known-plaintext positions and optional supplied-key re-encoding evidence.
use super::*;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Data {
    Text { text: String },
    Bytes { bytes: Vec<u8> },
}
impl Data {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Text { text } => text.as_bytes(),
            Self::Bytes { bytes } => bytes,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Bytes,
    Scalars,
    AsciiLetters,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub data: Data,
    #[serde(default)]
    pub key: Option<candidates::Key>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub candidates: Vec<Candidate>,
    pub crib: Data,
    #[serde(deserialize_with = "wire::integer")]
    pub offset: usize,
    pub view: View,
    #[serde(default)]
    pub ciphertext: Option<Data>,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Position {
    pub crib_index: usize,
    pub candidate_offset: usize,
    pub original_byte_offset: Option<usize>,
    pub expected: u32,
    pub actual: Option<u32>,
    pub matched: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Verification {
    pub input_index: usize,
    pub positions: Vec<Position>,
    pub all_match: Option<bool>,
    pub reencode_matches: Option<bool>,
    pub reencode_error: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub status: Status,
    pub view: View,
    pub scope: String,
    pub candidates: Vec<Verification>,
}
fn view(data: &Data, profile: View) -> Result<Vec<(u32, usize)>, OperationError> {
    let bytes = data.bytes();
    if matches!(profile, View::Bytes) {
        return Ok(bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| (u32::from(b), i))
            .collect());
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| invalid("scalar views require valid UTF-8"))?;
    Ok(text
        .char_indices()
        .filter(|(_, c)| !matches!(profile, View::AsciiLetters) || c.is_ascii_alphabetic())
        .map(|(i, c)| {
            (
                if matches!(profile, View::AsciiLetters) {
                    c.to_ascii_uppercase()
                } else {
                    c
                } as u32,
                i,
            )
        })
        .collect())
}
fn encode(text: &str, key: &candidates::Key) -> Result<String, OperationError> {
    use candidates::Key;
    match key {
        Key::Caesar { shift } if (0..26).contains(shift) => {
            Ok(crate::ciphers::caesar_encode(text, *shift))
        }
        Key::Affine { a, b } => crate::ciphers::classical::affine_encode(text, *a, *b),
        Key::Vigenere { key } => crate::ciphers::classical::vigenere_encode(text, key),
        Key::Transposition { configuration } => transposition::encode(text, configuration),
        _ => Err(invalid("invalid explicit key")),
    }
}
fn reencode(
    candidate: &Candidate,
    ciphertext: &Option<Data>,
    b: &mut Budget<'_>,
) -> (Option<bool>, Option<String>) {
    let Some(key) = &candidate.key else {
        return (None, None);
    };
    let Some(ciphertext) = ciphertext else {
        return (None, Some("supplied key requires ciphertext".into()));
    };
    let Data::Text { text } = &candidate.data else {
        return (
            None,
            Some("classical reencoding requires explicit text data".into()),
        );
    };
    if !b.take(text.len() * 2 + ciphertext.bytes().len()) {
        return (None, None);
    }
    match encode(text, key) {
        Ok(encoded) => (Some(encoded.as_bytes() == ciphertext.bytes()), None),
        Err(_) => (
            None,
            Some("invalid supplied classical key/configuration".into()),
        ),
    }
}
/// A crib match proves only the listed positions under the declared view. Empty
/// or interrupted comparisons have no all_match conclusion. Re-encoding compares
/// complete, exact bytes; no normalization or key discovery is inferred there.
pub fn verify(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    bound(r.candidates.len(), 4096)?;
    json_bound(r, MAX_REQUEST)?;
    let total = r
        .candidates
        .iter()
        .fold(r.crib.bytes().len(), |n, c| {
            n.saturating_add(c.data.bytes().len())
        })
        .saturating_add(r.ciphertext.as_ref().map_or(0, |c| c.bytes().len()));
    bound(total, MAX_TEXT)?;
    let crib = view(&r.crib, r.view)?;
    bound(crib.len(), 65536)?;
    if r.offset.checked_add(crib.len()).is_none() {
        return Err(invalid("crib offset overflows"));
    }
    let mut b = Budget::new(&r.limits, cancel)?;
    let mut candidates = Vec::new();
    let mut retained = 0;
    for (i, c) in r.candidates.iter().enumerate() {
        if !b.take(1) {
            break;
        }
        if candidates.len() == r.limits.max_results {
            b.stop(Stop::ResultLimit);
            break;
        }
        let data = view(&c.data, r.view)?;
        let mut positions = Vec::new();
        for (j, &(expected, _)) in crib.iter().enumerate() {
            if retained == 65536 {
                b.stop(Stop::ResultLimit);
                break;
            }
            if !b.take(1) {
                break;
            }
            let actual = data.get(r.offset + j);
            positions.push(Position {
                crib_index: j,
                candidate_offset: r.offset + j,
                original_byte_offset: actual.map(|x| x.1),
                expected,
                actual: actual.map(|x| x.0),
                matched: actual.is_some_and(|x| x.0 == expected),
            });
            retained += 1;
        }
        let all_match = (!crib.is_empty() && positions.len() == crib.len())
            .then(|| positions.iter().all(|p| p.matched));
        let (reencode_matches, reencode_error) = if b.status.complete {
            reencode(c, &r.ciphertext, &mut b)
        } else {
            (None, None)
        };
        candidates.push(Verification {
            input_index: i,
            positions,
            all_match,
            reencode_matches,
            reencode_error,
        });
        if !b.status.complete {
            break;
        }
    }
    finish(Report {status:b.status,view:r.view,scope:"Only listed crib positions and explicit reencoding are evidence; no key identity or authenticity claim.".into(),candidates})
}
