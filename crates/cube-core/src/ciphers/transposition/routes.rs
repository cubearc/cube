//! Full rectangles only. Every route visits each cell once.
use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RouteKind {
    Rows,
    Columns,
    Clockwise,
    Counterclockwise,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub kind: RouteKind,
    pub corner: Corner,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Routes {
    #[schemars(range(min = 1, max = 65536))]
    #[serde(deserialize_with = "wire::integer")]
    pub rows: usize,
    #[schemars(range(min = 1, max = 65536))]
    #[serde(deserialize_with = "wire::integer")]
    pub columns: usize,
    pub fill: Route,
    pub read: Route,
}
fn path(rows: usize, cols: usize, spec: Route) -> Vec<usize> {
    let (bottom, right) = match spec.corner {
        Corner::TopLeft => (false, false),
        Corner::TopRight => (false, true),
        Corner::BottomRight => (true, true),
        Corner::BottomLeft => (true, false),
    };
    let at = |r: usize, c: usize| {
        let r = if bottom { rows - 1 - r } else { r };
        let c = if right { cols - 1 - c } else { c };
        r * cols + c
    };
    match spec.kind {
        RouteKind::Rows => (0..rows)
            .flat_map(|r| (0..cols).map(move |c| at(r, c)))
            .collect(),
        RouteKind::Columns => (0..cols)
            .flat_map(|c| (0..rows).map(move |r| at(r, c)))
            .collect(),
        _ => {
            let clockwise = matches!(spec.kind, RouteKind::Clockwise);
            let mut dir = match (spec.corner, clockwise) {
                (Corner::TopLeft, true) | (Corner::TopRight, false) => 0,
                (Corner::TopRight, true) | (Corner::BottomRight, false) => 1,
                (Corner::BottomRight, true) | (Corner::BottomLeft, false) => 2,
                _ => 3,
            };
            let mut r = if bottom { rows - 1 } else { 0 };
            let mut c = if right { cols - 1 } else { 0 };
            let mut seen = vec![false; rows * cols];
            let mut out = Vec::with_capacity(rows * cols);
            for i in 0..rows * cols {
                out.push(r * cols + c);
                seen[r * cols + c] = true;
                if i + 1 == rows * cols {
                    break;
                }
                for _ in 0..4 {
                    let next = match dir {
                        0 => c.checked_add(1).filter(|&v| v < cols).map(|v| (r, v)),
                        1 => r.checked_add(1).filter(|&v| v < rows).map(|v| (v, c)),
                        2 => c.checked_sub(1).map(|v| (r, v)),
                        _ => r.checked_sub(1).map(|v| (v, c)),
                    };
                    if let Some((nr, nc)) = next
                        && !seen[nr * cols + nc]
                    {
                        r = nr;
                        c = nc;
                        break;
                    }
                    dir = (dir + if clockwise { 1 } else { 3 }) % 4;
                }
            }
            out
        }
    }
}
fn order(n: usize, p: &Routes, op: &'static str) -> Result<Vec<usize>, OperationError> {
    if p.rows == 0 || p.columns == 0 || p.rows.checked_mul(p.columns) != Some(n) {
        return Err(invalid(
            op,
            "shape",
            "positive full rectangle must equal scalar length; caller padding is required for incomplete grids",
        ));
    }
    let fill = path(p.rows, p.columns, p.fill);
    let read = path(p.rows, p.columns, p.read);
    let mut indices = vec![0; n];
    for (i, &j) in fill.iter().enumerate() {
        indices[j] = i;
    }
    Ok(read.iter().map(|&i| indices[i]).collect())
}
/// Fill by one route and read by another. No filler, ragged mask or normalization.
pub fn route(text: &str, p: &Routes, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.route.decode"
    } else {
        "cipher.route.encode"
    };
    let x = scalars(text, op)?;
    Ok(apply(&x, &order(x.len(), p, op)?, decode))
}
/// A route profile requiring a clockwise/counterclockwise inward spiral readout.
pub fn spiral(text: &str, p: &Routes, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.spiral.decode"
    } else {
        "cipher.spiral.encode"
    };
    if !matches!(
        p.read.kind,
        RouteKind::Clockwise | RouteKind::Counterclockwise
    ) {
        return Err(invalid(
            op,
            "read",
            "spiral profile requires a spiral read route",
        ));
    }
    let x = scalars(text, op)?;
    Ok(apply(&x, &order(x.len(), p, op)?, decode))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaesarBox {
    #[schemars(range(min = 1, max = 256))]
    #[serde(deserialize_with = "wire::integer")]
    pub side: usize,
}
/// Explicit square, top-left row fill/column read. Caller supplies any padding.
pub fn caesar_box(text: &str, p: &CaesarBox, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.caesar_box.decode"
    } else {
        "cipher.caesar_box.encode"
    };
    let x = scalars(text, op)?;
    let p = Routes {
        rows: p.side,
        columns: p.side,
        fill: Route {
            kind: RouteKind::Rows,
            corner: Corner::TopLeft,
        },
        read: Route {
            kind: RouteKind::Columns,
            corner: Corner::TopLeft,
        },
    };
    Ok(apply(&x, &order(x.len(), &p, op)?, decode))
}
