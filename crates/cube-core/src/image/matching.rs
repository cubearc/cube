//! Valid nonwrapping template locations over exact physical channel moments.
use super::{
    Channel, Image, Region, invalid, limit,
    plane::{MIB, Plane},
    spatial::PairMoments,
};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BinaryHeap};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Ssd,
    Ncc,
}

fn default_best() -> usize {
    16
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemplateOptions {
    pub metric: Metric,
    #[serde(default = "default_best")]
    #[schemars(range(min = 1, max = 256))]
    pub best: usize,
    #[serde(default)]
    pub dense: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Match {
    pub x: usize,
    pub y: usize,
    pub score: Option<f64>,
    pub reason: Option<&'static str>,
    pub moments: PairMoments,
}

struct Ranked {
    rank: f64,
    result: Match,
}
impl Ord for Ranked {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank
            .total_cmp(&other.rank)
            .then_with(|| (self.result.y, self.result.x).cmp(&(other.result.y, other.result.x)))
    }
}
impl PartialOrd for Ranked {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Ranked {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for Ranked {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub options: TemplateOptions,
    pub width: usize,
    pub height: usize,
    pub evaluated: usize,
    pub matches: Vec<Match>,
    pub dense: Option<Vec<Option<f64>>>,
}

fn evaluate(source: Plane<'_>, template: Plane<'_>, x: usize, y: usize) -> PairMoments {
    let mut moments = PairMoments::default();
    for ty in 0..template.height {
        for tx in 0..template.width {
            moments.observe(source.sample(x + tx, y + ty), template.sample(tx, ty));
        }
    }
    moments
}

/// Compare all valid placements using SSD or zero-mean NCC. Null NCC scores
/// follow valid scores in the ranking. Equal scores sort by row, then column.
pub fn match_template(
    source: &Image,
    channel: Channel,
    template: &Image,
    template_channel: Channel,
    options: TemplateOptions,
) -> Result<Report, OperationError> {
    (|| {
        let source = Plane::new(source, Region::full(source), channel, MIB)?;
        let template = Plane::new(template, Region::full(template), template_channel, 65_536)?;
        if template.width > source.width || template.height > source.height {
            return Err(invalid("template must fit inside the image"));
        }
        if !(1..=256).contains(&options.best) {
            return Err(invalid("best must be in 1..256"));
        }
        let (width, height) = (
            source.width - template.width + 1,
            source.height - template.height + 1,
        );
        let evaluated = width * height;
        if evaluated > 16 * MIB / template.len()
            || (options.dense && evaluated > (16 * MIB - 128 * 1024) / 26)
        {
            return Err(limit(
                "template exceeds 16 Mi visits or 16 MiB dense report",
            ));
        }
        let mut ranked: BinaryHeap<Ranked> = BinaryHeap::with_capacity(options.best + 1);
        let mut dense = options.dense.then(|| Vec::with_capacity(evaluated));
        for y in 0..height {
            for x in 0..width {
                let moments = evaluate(source, template, x, y);
                let (score, rank) = match options.metric {
                    Metric::Ssd => {
                        let ssd = moments.squared_difference as f64;
                        (Some(ssd), ssd)
                    }
                    Metric::Ncc => {
                        let ncc = moments.correlation().map(|v| if v == 0. { 0. } else { v });
                        (ncc, ncc.map_or(f64::INFINITY, |s| -s))
                    }
                };
                if let Some(map) = &mut dense {
                    map.push(score);
                }
                let result = Match {
                    x,
                    y,
                    score,
                    reason: score.is_none().then_some("zero_variance"),
                    moments,
                };
                ranked.push(Ranked { rank, result });
                if ranked.len() > options.best {
                    ranked.pop();
                }
            }
        }
        Ok(Report {
            options,
            width,
            height,
            evaluated,
            matches: ranked
                .into_sorted_vec()
                .into_iter()
                .map(|item| item.result)
                .collect(),
            dense,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.template.match"))
}
