use super::*;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Face {
    pub value: i32,
    #[serde(serialize_with = "weight", deserialize_with = "read_weight")]
    pub weight: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiceModel {
    pub dice: Vec<Vec<Face>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DicePair {
    pub left: DiceModel,
    pub right: DiceModel,
}
#[derive(Debug, Serialize)]
pub struct Point {
    pub value: i64,
    #[serde(serialize_with = "decimal")]
    pub mass: u128,
    pub probability: Fraction,
}
#[derive(Debug, Serialize)]
pub struct SumReport {
    pub assumptions: DiceModel,
    pub independence: &'static str,
    pub distribution: Vec<Point>,
    #[serde(serialize_with = "decimal")]
    pub total_mass: u128,
    pub expected_value: SignedFraction,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
#[derive(Debug, Serialize)]
pub struct Comparison {
    pub assumptions: DicePair,
    pub independence: &'static str,
    pub win: Fraction,
    pub tie: Fraction,
    pub loss: Fraction,
    #[serde(serialize_with = "decimal")]
    pub win_mass: u128,
    #[serde(serialize_with = "decimal")]
    pub tie_mass: u128,
    #[serde(serialize_with = "decimal")]
    pub loss_mass: u128,
    #[serde(serialize_with = "decimal")]
    pub total_mass: u128,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
fn validate(model: &DiceModel, op: &'static str) -> Result<(), OperationError> {
    if model.dice.len() > 8 {
        return Err(resource(op, "dice", "at most 8 independent dice"));
    }
    if model.dice.iter().any(|d| d.is_empty() || d.len() > 64) {
        return Err(resource(op, "faces", "each die requires 1..64 faces"));
    }
    Ok(())
}
fn distribution(
    model: &DiceModel,
    b: &mut Budget,
) -> Result<(BTreeMap<i64, u128>, u128), OperationError> {
    validate(model, b.op)?;
    let mut pmf: BTreeMap<i64, u128> = BTreeMap::from([(0, 1)]);
    let mut total = 1;
    for die in &model.dice {
        let mut faces = BTreeMap::new();
        let mut mass = 0;
        for face in die {
            b.charge(1)?;
            mass = add(mass, face.weight.into(), b.op)?;
            if face.weight > 0 {
                let v = faces.entry(i64::from(face.value)).or_insert(0);
                *v = add(*v, face.weight.into(), b.op)?;
            }
        }
        if mass == 0 {
            return Err(invalid(
                b.op,
                "weights",
                "every die requires positive total weight",
            ));
        }
        total = mul(total, mass, b.op)?;
        let mut next = BTreeMap::new();
        for (&x, &a) in &pmf {
            for (&y, &c) in &faces {
                b.charge(1)?;
                let value = x
                    .checked_add(y)
                    .ok_or_else(|| resource(b.op, "value", "signed i64 sum overflow"))?;
                if next.len() == MAX_SUPPORT && !next.contains_key(&value) {
                    return Err(resource(
                        b.op,
                        "support",
                        "retained dice support exceeds 4096 points",
                    ));
                }
                let v = next.entry(value).or_insert(0);
                *v = add(*v, mul(a, c, b.op)?, b.op)?;
            }
        }
        pmf = next;
    }
    Ok((pmf, total))
}
/// Convolve independent supplied dice, retaining sorted positive mass support.
/// Zero dice yields the identity distribution {0:1}; repeated faces aggregate.
pub fn dice_sum(model: &DiceModel, limits: Limits) -> Result<SumReport, OperationError> {
    const OP: &str = "probability.dice.sum";
    validate(model, OP)?;
    let report = 4096 + MAX_SUPPORT * 256 + model.dice.iter().map(|d| d.len() * 128).sum::<usize>();
    let mut b = Budget::new(
        OP,
        limits,
        MAX_INPUT * 4 + MAX_SUPPORT * 512 + report * 2,
        report,
    )?;
    let (pmf, total) = distribution(model, &mut b)?;
    let (mut pos, mut neg) = (0, 0);
    let mut points = Vec::new();
    for (value, mass) in pmf {
        b.charge(1)?;
        let product = mul(u128::from(value.unsigned_abs()), mass, OP)?;
        if value < 0 {
            neg = add(neg, product, OP)?;
        } else {
            pos = add(pos, product, OP)?;
        }
        points.push(Point {
            value,
            mass,
            probability: fraction(mass, total),
        });
    }
    let e = fraction(pos.abs_diff(neg), total);
    Ok(SumReport {
        assumptions: model.clone(),
        independence: "supplied dice are independent",
        distribution: points,
        total_mass: total,
        expected_value: SignedFraction {
            negative: neg > pos,
            numerator: e.numerator,
            denominator: e.denominator,
        },
        work: b.work,
        memory_bytes_upper_bound: b.memory,
        report_bytes_upper_bound: b.report,
    })
}
/// Compare two independent sum distributions. Win means strictly left > right;
/// equal means and pairwise winning relations imply no transitivity claim.
pub fn dice_compare(model: &DicePair, limits: Limits) -> Result<Comparison, OperationError> {
    const OP: &str = "probability.dice.compare";
    validate(&model.left, OP)?;
    validate(&model.right, OP)?;
    let report = 4096
        + model
            .left
            .dice
            .iter()
            .chain(&model.right.dice)
            .map(|d| d.len() * 128)
            .sum::<usize>();
    let mut b = Budget::new(
        OP,
        limits,
        MAX_INPUT * 4 + MAX_SUPPORT * 768 + report * 2,
        report,
    )?;
    let (a, ma) = distribution(&model.left, &mut b)?;
    let (c, mc) = distribution(&model.right, &mut b)?;
    let total = mul(ma, mc, OP)?;
    let (mut win, mut tie, mut loss) = (0, 0, 0);
    for (x, a) in a {
        for (&y, &c) in &c {
            b.charge(1)?;
            let m = mul(a, c, OP)?;
            let target = if x > y {
                &mut win
            } else if x == y {
                &mut tie
            } else {
                &mut loss
            };
            *target = add(*target, m, OP)?;
        }
    }
    Ok(Comparison {
        assumptions: model.clone(),
        independence: "left and right dice collections are independent",
        win: fraction(win, total),
        tie: fraction(tie, total),
        loss: fraction(loss, total),
        win_mass: win,
        tie_mass: tie,
        loss_mass: loss,
        total_mass: total,
        work: b.work,
        memory_bytes_upper_bound: b.memory,
        report_bytes_upper_bound: b.report,
    })
}
