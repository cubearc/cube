use super::*;
use std::collections::BTreeSet;
/// Ten-letter orthographic Malespín profile: a/e, i/o, b/t, f/g, p/m.
/// ASCII case is retained. Other scalars (including accented vowels) remain
/// exact. This frozen written profile does not infer regional sound changes.
pub fn malespin(text: &str) -> Result<String, OperationError> {
    super::super::alphabet::Mapping::new("aeiobtfgpmAEIOBTFGPM", "eaoitbgfmpEAOITBGFMP")
        .and_then(|map| map.encode(text, "preserve"))
        .map_err(|e| e.operation("cipher.malespin"))
}
/// Reciprocal caller-defined disjoint pairs. Every symbol has exactly one
/// different partner; no fixed points, odd alphabets, folding or guessed key.
pub fn vatsyayana(text: &str, parameters: &Pairs) -> Result<String, OperationError> {
    paired_mapping(parameters)
        .and_then(|mapping| mapping.encode(text, parameters.unmapped.as_str()))
        .map_err(|e| e.operation("cipher.vatsyayana"))
}
fn paired_mapping(parameters: &Pairs) -> Result<super::super::alphabet::Mapping, OperationError> {
    bound("cipher.vatsyayana", "pairs", parameters.pairs.len(), 2048)?;
    if parameters.pairs.is_empty() {
        return Err(invalid("pairs"));
    }
    let mut source = String::new();
    let mut target = String::new();
    for [a, b] in &parameters.pairs {
        if a == b {
            return Err(invalid("pairs"));
        }
        source.extend([*a, *b]);
        target.extend([*b, *a]);
    }
    super::super::alphabet::Mapping::new(&source, &target)
}
/// Explicit ACA 10x10 multi-character keysquare, row-major cells and required
/// row/column permutations of 0..9. Units are already segmented by the caller.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = MAX_BYTES))]
pub struct Syllabary {
    #[schemars(
        length(min = 100, max = 100),
        with = "Vec<super::super::homophonic::TokenText>"
    )]
    pub cells: Vec<String>,
    #[schemars(length(equal = 10))]
    pub row_labels: String,
    #[schemars(length(equal = 10))]
    pub column_labels: String,
}
impl Syllabary {
    pub fn book(self) -> Result<Book, OperationError> {
        if self.cells.len() != 100 || !digits(&self.row_labels) || !digits(&self.column_labels) {
            return Err(invalid("definition"));
        }
        let rows = self.row_labels.as_bytes();
        let columns = self.column_labels.as_bytes();
        let entries = self
            .cells
            .into_iter()
            .enumerate()
            .map(|(index, plain)| Entry {
                alternatives: vec![plain],
                codes: vec![format!(
                    "{}{}",
                    char::from(rows[index / 10]),
                    char::from(columns[index % 10])
                )],
            })
            .collect();
        Book::new(Definition {
            entries,
            null_codes: vec![],
            cancel_codes: vec![],
        })
    }
}
fn digits(s: &str) -> bool {
    s.len() == 10
        && s.bytes().all(|b| b.is_ascii_digit())
        && s.bytes().collect::<BTreeSet<_>>().len() == 10
}
/// Bazeries' Grand Chiffre de 1691, second deciphering table, 1893 pp.280–289.
/// Fixed ASCII transcription profile: accents omitted, original capitalization
/// retained; historical I/J and U/V readings remain explicit alternatives.
/// Blank entries reject. 519 retains the author's question mark as literal
/// `Geneve?`, never silently asserting a historical interpretation.
pub fn great_cipher() -> Result<Book, OperationError> {
    let mut entries = Vec::new();
    let mut null_codes = Vec::new();
    let mut cancel_codes = Vec::new();
    for line in include_str!("grand_chiffre_1691.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (code, plain) = line.split_once('\t').expect("reviewed fixed table");
        match plain {
            "?" => {}
            "NULL" => null_codes.push(code.to_owned()),
            "CANCEL" => cancel_codes.push(code.to_owned()),
            _ => entries.push(Entry {
                alternatives: readings(plain),
                codes: vec![code.to_owned()],
            }),
        }
    }
    Book::new(Definition {
        entries,
        null_codes,
        cancel_codes,
    })
}
fn readings(plain: &str) -> Vec<String> {
    let separator = match plain {
        "I-J" | "U-V" | "Ia-ja" | "Ie-je" | "Ie-Je" | "Iu-ju" | "Iu-Ju" | "Io-jo" | "Ua-Va"
        | "Ue-Ve" | "Eu-Ev" => "-",
        "Rait ou roit" | "Hi ou Hy" => " ou ",
        "Persuad (ou) Asseur" => " (ou) ",
        _ => return vec![plain.to_owned()],
    };
    plain.split(separator).map(str::to_owned).collect()
}
