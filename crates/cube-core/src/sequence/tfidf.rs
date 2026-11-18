//! Immutable explicit document frequencies with an exact model identity.
use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAX_DOCUMENTS: usize = 1024;
const PROFILE: &str = "raw_tf_smooth_idf_l2_v1";

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    pub documents: Vec<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frequency {
    pub token: String,
    pub df: u32,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TfidfModel {
    pub profile: String,
    pub documents: u32,
    pub frequencies: Vec<Frequency>,
    pub sha256: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TfidfRequest {
    pub model: TfidfModel,
    pub left: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct TfidfReport {
    pub profile: &'static str,
    pub model_sha256: String,
    pub similarity: Option<f64>,
    pub reason: Option<&'static str>,
}

fn validate_documents<'a>(batch: impl Iterator<Item = &'a [String]>) -> Result<(), OperationError> {
    let (mut count, mut bytes) = (0, 0);
    for document in batch {
        count += document.len();
        if count > MAX_UNITS {
            return Err(resource("TF-IDF exceeds 65536 total tokens"));
        }
        for token in document {
            bytes += token.len();
            if bytes > TEXT_BYTES {
                return Err(resource("TF-IDF exceeds 1 MiB of token bytes"));
            }
        }
    }
    Ok(())
}

fn identity(documents: u32, frequencies: &[Frequency]) -> String {
    let mut hash = Sha256::new();
    hash.update(PROFILE);
    hash.update([0]);
    hash.update(documents.to_be_bytes());
    for entry in frequencies {
        hash.update((entry.token.len() as u32).to_be_bytes());
        hash.update(entry.token.as_bytes());
        hash.update(entry.df.to_be_bytes());
    }
    hex::encode(hash.finalize())
}

/// Build document frequencies from caller tokens without lexical processing.
pub fn tfidf_model(corpus: &Corpus) -> Result<TfidfModel, OperationError> {
    if corpus.documents.is_empty() {
        return Err(invalid("TF-IDF requires a nonempty corpus"));
    }
    if corpus.documents.len() > MAX_DOCUMENTS {
        return Err(resource("TF-IDF exceeds 1024 documents"));
    }
    validate_documents(corpus.documents.iter().map(Vec::as_slice))?;
    let mut counts = BTreeMap::<&str, u32>::new();
    for document in &corpus.documents {
        let unique = document.iter().map(String::as_str).collect::<BTreeSet<_>>();
        for token in unique {
            *counts.entry(token).or_default() += 1;
            if counts.len() > inventory::DISTINCT {
                return Err(resource("TF-IDF vocabulary exceeds 10000 terms"));
            }
        }
    }
    let documents = corpus.documents.len() as u32;
    let frequencies: Vec<_> = counts
        .into_iter()
        .map(|(token, df)| Frequency {
            token: token.into(),
            df,
        })
        .collect();
    let sha256 = identity(documents, &frequencies);
    let model = TfidfModel {
        profile: PROFILE.into(),
        documents,
        frequencies,
        sha256,
    };
    serialized_bound(&model, 4 << 20, "TF-IDF model exceeds 4 MiB encoded")?;
    Ok(model)
}

fn validate_model(model: &TfidfModel) -> Result<(), OperationError> {
    if model.profile != PROFILE
        || !(1..=MAX_DOCUMENTS as u32).contains(&model.documents)
        || model.frequencies.len() > inventory::DISTINCT
        || model
            .frequencies
            .iter()
            .map(|r| r.token.len())
            .sum::<usize>()
            > TEXT_BYTES
        || model
            .frequencies
            .iter()
            .any(|row| row.df == 0 || row.df > model.documents)
        || !model
            .frequencies
            .windows(2)
            .all(|pair| pair[0].token < pair[1].token)
        || identity(model.documents, &model.frequencies) != model.sha256
    {
        return Err(invalid(
            "TF-IDF model profile, counts or identity are invalid",
        ));
    }
    Ok(())
}

/// Compare two documents under a validated immutable model.
pub fn tfidf_compare(input: &TfidfRequest) -> Result<TfidfReport, OperationError> {
    validate_model(&input.model)?;
    validate_documents([input.left.as_slice(), input.right.as_slice()].into_iter())?;
    let mut counts = BTreeMap::<&str, (u32, u32)>::new();
    for term in &input.left {
        counts.entry(term).or_default().0 += 1;
    }
    for term in &input.right {
        counts.entry(term).or_default().1 += 1;
    }
    let total = f64::from(input.model.documents) + 1.0;
    let coordinates: Vec<_> = counts
        .into_iter()
        .map(|(key, (left, right))| {
            let df = input
                .model
                .frequencies
                .binary_search_by(|entry| entry.token.as_str().cmp(key))
                .map_or(0, |i| input.model.frequencies[i].df);
            let idf = (total / (f64::from(df) + 1.0)).ln() + 1.0;
            (idf * f64::from(left), idf * f64::from(right))
        })
        .collect();
    let similarity = vector::cosine(&coordinates);
    Ok(TfidfReport {
        profile: PROFILE,
        model_sha256: input.model.sha256.clone(),
        similarity,
        reason: similarity.is_none().then_some("zero_vector"),
    })
}
