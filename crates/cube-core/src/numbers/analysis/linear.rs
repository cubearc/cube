//! Exact row-major matrix operations and polynomial interpolation.
pub use super::rational::{FractionInput, Rational, Scalar};
use super::{Work, invalid, parameter, resource};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatrixInput {
    #[schemars(range(max = 32))]
    pub rows: usize,
    #[schemars(range(max = 32))]
    pub cols: usize,
    #[schemars(length(max = 1024))]
    pub cells: Vec<Scalar>,
}
#[derive(Serialize, JsonSchema)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Rational>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum MatrixRequest {
    Transpose {
        matrix: MatrixInput,
    },
    Multiply {
        left: MatrixInput,
        right: MatrixInput,
    },
    Determinant {
        matrix: MatrixInput,
    },
    Reduce {
        matrix: MatrixInput,
    },
    Solve {
        matrix: MatrixInput,
        rhs: Vec<Scalar>,
    },
    Inverse {
        matrix: MatrixInput,
    },
}
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SolveStatus {
    Unique,
    Inconsistent,
    Underdetermined,
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MatrixReport {
    Transpose {
        matrix: Matrix,
    },
    Multiply {
        matrix: Matrix,
    },
    Determinant {
        value: Rational,
    },
    Reduce {
        matrix: Matrix,
        rank: usize,
        pivots: Vec<usize>,
    },
    Solve {
        reduced: Matrix,
        rank: usize,
        pivots: Vec<usize>,
        status: SolveStatus,
        solution: Option<Vec<Rational>>,
    },
    Inverse {
        matrix: Option<Matrix>,
        singular: bool,
    },
}
impl Matrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            cells: vec![Rational::ZERO; rows * cols],
        }
    }
    fn input(input: &MatrixInput) -> Result<Self, OperationError> {
        if input.rows > 32 || input.cols > 32 || input.cells.len() != input.rows * input.cols {
            return Err(invalid(
                "rows and columns must be at most 32 and match the flat cells",
            ));
        }
        let cells = reduce(&input.cells)?;
        Ok(Self {
            rows: input.rows,
            cols: input.cols,
            cells,
        })
    }
    fn at(&self, row: usize, col: usize) -> Rational {
        self.cells[row * self.cols + col]
    }
    fn set(&mut self, row: usize, col: usize, value: Rational) {
        self.cells[row * self.cols + col] = value;
    }
    fn require_square(&self) -> Result<(), OperationError> {
        if self.rows != self.cols {
            return Err(invalid("operation requires a square matrix"));
        }
        Ok(())
    }
    fn swap(&mut self, a: usize, b: usize) {
        for c in 0..self.cols {
            self.cells.swap(a * self.cols + c, b * self.cols + c);
        }
    }
    fn transpose(&self) -> Matrix {
        let mut out = Self::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.set(c, r, self.at(r, c));
            }
        }
        out
    }
    fn augment(&self, rhs: &Self) -> Self {
        let mut matrix = Self::new(self.rows, self.cols + rhs.cols);
        for r in 0..self.rows {
            for c in 0..self.cols {
                matrix.set(r, c, self.at(r, c));
            }
            for c in 0..rhs.cols {
                matrix.set(r, self.cols + c, rhs.at(r, c));
            }
        }
        matrix
    }
}

fn reduce(values: &[Scalar]) -> Result<Vec<Rational>, OperationError> {
    values.iter().copied().map(Scalar::reduced).collect()
}
fn multiply(left: Matrix, right: Matrix, work: &mut Work) -> Result<Matrix, OperationError> {
    if left.cols != right.rows {
        return Err(invalid(
            "matrix multiplication needs matching inner dimensions",
        ));
    }
    let mut matrix = Matrix::new(left.rows, right.cols);
    for r in 0..matrix.rows {
        for c in 0..matrix.cols {
            let mut value = Rational::ZERO;
            for i in 0..left.cols {
                let product = left.at(r, i).mul(right.at(i, c), work)?;
                value = value.add(product, work)?;
            }
            matrix.set(r, c, value);
        }
    }
    Ok(matrix)
}

fn cancel_row(
    matrix: &mut Matrix,
    row: usize,
    pivot: usize,
    col: usize,
    scale: Rational,
    work: &mut Work,
) -> Result<(), OperationError> {
    if scale.zero() {
        return Ok(());
    }
    for c in col..matrix.cols {
        let product = scale.mul(matrix.at(pivot, c), work)?;
        matrix.set(row, c, matrix.at(row, c).sub(product, work)?);
    }
    Ok(())
}
fn rref(
    matrix: &mut Matrix,
    coefficients: usize,
    work: &mut Work,
) -> Result<Vec<usize>, OperationError> {
    let mut pivots = Vec::new();
    for col in 0..coefficients {
        let row = pivots.len();
        let Some(next) = (row..matrix.rows).find(|&r| !matrix.at(r, col).zero()) else {
            continue;
        };
        matrix.swap(row, next);
        let pivot = matrix.at(row, col);
        for c in col..matrix.cols {
            matrix.set(row, c, matrix.at(row, c).div(pivot, work)?);
        }
        for r in 0..matrix.rows {
            if r == row {
                continue;
            }
            let scale = matrix.at(r, col);
            cancel_row(matrix, r, row, col, scale, work)?;
        }
        pivots.push(col);
    }
    Ok(pivots)
}
fn determinant(mut matrix: Matrix, work: &mut Work) -> Result<Rational, OperationError> {
    matrix.require_square()?;
    let mut value = Rational::ONE;
    let mut negate = false;
    for col in 0..matrix.cols {
        let Some(row) = (col..matrix.rows).find(|&r| !matrix.at(r, col).zero()) else {
            return Ok(Rational::ZERO);
        };
        if row != col {
            matrix.swap(row, col);
            negate = !negate;
        }
        let pivot = matrix.at(col, col);
        value = value.mul(pivot, work)?;
        for r in col + 1..matrix.rows {
            let entry = matrix.at(r, col);
            if !entry.zero() {
                cancel_row(&mut matrix, r, col, col, entry.div(pivot, work)?, work)?;
            }
        }
    }
    if negate {
        Rational::ZERO.sub(value, work)
    } else {
        Ok(value)
    }
}
fn solve(matrix: Matrix, rhs: &[Scalar], work: &mut Work) -> Result<MatrixReport, OperationError> {
    if rhs.len() != matrix.rows {
        return Err(invalid("linear system RHS requires one value per row"));
    }
    let rhs = Matrix {
        rows: matrix.rows,
        cols: 1,
        cells: reduce(rhs)?,
    };
    let mut reduced = matrix.augment(&rhs);
    let pivots = rref(&mut reduced, matrix.cols, work)?;
    let rank = pivots.len();
    let inconsistent = (rank..matrix.rows).any(|r| !reduced.at(r, matrix.cols).zero());
    let status = if inconsistent {
        SolveStatus::Inconsistent
    } else if rank < matrix.cols {
        SolveStatus::Underdetermined
    } else {
        SolveStatus::Unique
    };
    let solution = matches!(status, SolveStatus::Unique).then(|| {
        (0..matrix.cols)
            .map(|r| reduced.at(r, matrix.cols))
            .collect()
    });
    Ok(MatrixReport::Solve {
        reduced,
        rank,
        pivots,
        status,
        solution,
    })
}
fn inverse(matrix: Matrix, work: &mut Work) -> Result<MatrixReport, OperationError> {
    matrix.require_square()?;
    let n = matrix.rows;
    let mut identity = Matrix::new(n, n);
    for i in 0..n {
        identity.set(i, i, Rational::ONE);
    }
    let mut reduced = matrix.augment(&identity);
    let pivots = rref(&mut reduced, n, work)?;
    if pivots.len() != n {
        return Ok(MatrixReport::Inverse {
            matrix: None,
            singular: true,
        });
    }
    for r in 0..n {
        for c in 0..n {
            identity.set(r, c, reduced.at(r, n + c));
        }
    }
    Ok(MatrixReport::Inverse {
        matrix: Some(identity),
        singular: false,
    })
}

/// Operate on case-specific typed exact matrices using deterministic pivots.
pub fn matrix(request: &MatrixRequest) -> Result<MatrixReport, OperationError> {
    let mut work = Work(4 << 20);
    Ok(match request {
        MatrixRequest::Transpose { matrix } => MatrixReport::Transpose {
            matrix: Matrix::input(matrix)?.transpose(),
        },
        MatrixRequest::Multiply { left, right } => MatrixReport::Multiply {
            matrix: multiply(Matrix::input(left)?, Matrix::input(right)?, &mut work)?,
        },
        MatrixRequest::Determinant { matrix } => MatrixReport::Determinant {
            value: determinant(Matrix::input(matrix)?, &mut work)?,
        },
        MatrixRequest::Reduce { matrix } => {
            let mut matrix = Matrix::input(matrix)?;
            let columns = matrix.cols;
            let pivots = rref(&mut matrix, columns, &mut work)?;
            MatrixReport::Reduce {
                matrix,
                rank: pivots.len(),
                pivots,
            }
        }
        MatrixRequest::Solve { matrix, rhs } => solve(Matrix::input(matrix)?, rhs, &mut work)?,
        MatrixRequest::Inverse { matrix } => inverse(Matrix::input(matrix)?, &mut work)?,
    })
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMethod {
    Lagrange,
    Newton,
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: Scalar,
    pub y: Scalar,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolynomialRequest {
    Differences {
        values: Vec<Scalar>,
    },
    Interpolate {
        method: InterpolationMethod,
        samples: Vec<Point>,
        points: Vec<Scalar>,
        #[serde(default)]
        coefficients: bool,
    },
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PolynomialReport {
    Differences {
        rows: Vec<Vec<Rational>>,
    },
    Interpolate {
        method: InterpolationMethod,
        values: Vec<Rational>,
        coefficients: Option<Vec<Rational>>,
    },
}

fn differences(
    mut row: Vec<Rational>,
    work: &mut Work,
) -> Result<Vec<Vec<Rational>>, OperationError> {
    let mut rows = Vec::new();
    while !row.is_empty() {
        let next = row
            .windows(2)
            .map(|p| p[1].sub(p[0], work))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(row);
        row = next;
    }
    Ok(rows)
}

fn divided(
    xs: &[Rational],
    ys: &[Rational],
    work: &mut Work,
) -> Result<Vec<Rational>, OperationError> {
    let mut a = ys.to_vec();
    for gap in 1..a.len() {
        for i in (gap..a.len()).rev() {
            a[i] = a[i]
                .sub(a[i - 1], work)?
                .div(xs[i].sub(xs[i - gap], work)?, work)?;
        }
    }
    Ok(a)
}

fn newton_value(
    xs: &[Rational],
    divided: &[Rational],
    point: Rational,
    work: &mut Work,
) -> Result<Rational, OperationError> {
    let mut value = Rational::ZERO;
    for (i, &degree) in divided.iter().enumerate().rev() {
        if i + 1 == divided.len() {
            value = degree;
        } else {
            value = value
                .mul(point.sub(xs[i], work)?, work)?
                .add(degree, work)?;
        }
    }
    Ok(value)
}

fn lagrange_value(
    xs: &[Rational],
    ys: &[Rational],
    point: Rational,
    work: &mut Work,
) -> Result<Rational, OperationError> {
    if let Some(i) = xs.iter().position(|&x| x == point) {
        return Ok(ys[i]);
    }
    let mut value = Rational::ZERO;
    for (i, (&x, &y)) in xs.iter().zip(ys).enumerate() {
        if y.zero() {
            continue;
        }
        let mut term = y;
        for (j, &other) in xs.iter().enumerate() {
            if i != j {
                let basis = point.sub(other, work)?.div(x.sub(other, work)?, work)?;
                term = term.mul(basis, work)?;
            }
        }
        value = value.add(term, work)?;
    }
    Ok(value)
}

fn coefficients(
    xs: &[Rational],
    newton: &[Rational],
    work: &mut Work,
) -> Result<Vec<Rational>, OperationError> {
    let mut coeffs = vec![*newton.last().expect("nonempty samples")];
    for i in (0..newton.len() - 1).rev() {
        let mut next = vec![Rational::ZERO; coeffs.len() + 1];
        next[0] = newton[i];
        for (power, &value) in coeffs.iter().enumerate() {
            next[power] = next[power].sub(value.mul(xs[i], work)?, work)?;
            next[power + 1] = next[power + 1].add(value, work)?;
        }
        coeffs = next;
    }
    Ok(coeffs)
}

fn interpolate(
    method: InterpolationMethod,
    samples: &[Point],
    points: &[Scalar],
    power_basis: bool,
    work: &mut Work,
) -> Result<PolynomialReport, OperationError> {
    if samples.is_empty() || points.is_empty() {
        return Err(parameter(
            "interpolation requires at least one sample and evaluation point",
        ));
    }
    if samples.len() > 64 || points.len() > 256 {
        return Err(resource(
            "interpolation exceeds 64 samples or 256 evaluation points",
        ));
    }
    let mut xs = Vec::with_capacity(samples.len());
    let mut ys = Vec::with_capacity(samples.len());
    for sample in samples {
        let x = sample.x.reduced()?;
        if xs.contains(&x) {
            return Err(invalid(
                "sample x coordinates must be distinct exact rationals",
            ));
        }
        xs.push(x);
        ys.push(sample.y.reduced()?);
    }
    let divided = if matches!(method, InterpolationMethod::Newton) || power_basis {
        divided(&xs, &ys, work)?
    } else {
        Vec::new()
    };
    let values = points
        .iter()
        .copied()
        .map(|point| {
            let x = point.reduced()?;
            match method {
                InterpolationMethod::Lagrange => lagrange_value(&xs, &ys, x, work),
                InterpolationMethod::Newton => newton_value(&xs, &divided, x, work),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let coefficients = power_basis
        .then(|| coefficients(&xs, &divided, work))
        .transpose()?;
    Ok(PolynomialReport::Interpolate {
        method,
        values,
        coefficients,
    })
}

/// Measure forward differences or evaluate an exact interpolating polynomial.
pub fn interpolation(request: &PolynomialRequest) -> Result<PolynomialReport, OperationError> {
    let mut work = Work(4 << 20);
    match request {
        PolynomialRequest::Differences { values } => {
            if values.len() > 64 {
                return Err(resource("difference table exceeds 64 samples"));
            }
            Ok(PolynomialReport::Differences {
                rows: differences(reduce(values)?, &mut work)?,
            })
        }
        PolynomialRequest::Interpolate {
            method,
            samples,
            points,
            coefficients,
        } => interpolate(*method, samples, points, *coefficients, &mut work),
    }
}
