//! Furthest-reaching edit frontiers with bounded exact traceback.
use super::*;

const TRACE_BYTES: usize = 48 << 20;

fn at(layer: &[i32], i: usize) -> Option<usize> {
    layer
        .get(i)
        .copied()
        .filter(|&x| x >= 0)
        .map(|x| x as usize)
}

fn advance(
    previous: &[i32],
    d: usize,
    i: usize,
    n: usize,
    m: usize,
) -> Option<(usize, usize, EditKind)> {
    let diagonal = 2 * i as isize - d as isize;
    let after = |x: usize, kind| {
        let y = x as isize - diagonal;
        (x <= n && y >= 0 && y as usize <= m).then_some((x, y as usize, kind))
    };
    let deletion = i
        .checked_sub(1)
        .and_then(|j| at(previous, j))
        .and_then(|x| after(x + 1, EditKind::Delete));
    let insertion = at(previous, i).and_then(|x| after(x, EditKind::Insert));
    match (deletion, insertion) {
        (Some(a), Some(b)) => Some(if a.0 >= b.0 { a } else { b }),
        (a, b) => a.or(b),
    }
}

fn snake(
    a: &[u32],
    b: &[u32],
    mut x: usize,
    mut y: usize,
    work: &mut Work,
) -> Result<(usize, usize), OperationError> {
    while x < a.len() && y < b.len() {
        work.charge(1)?;
        if a[x] != b[y] {
            break;
        }
        x += 1;
        y += 1;
    }
    Ok((x, y))
}

fn traceback(a: &[u32], b: &[u32], layers: &[Vec<i32>], out: &mut Vec<EditKind>) {
    let from = out.len();
    let (mut x, mut y) = (a.len(), b.len());
    for d in (1..layers.len()).rev() {
        let diagonal = x as isize - y as isize;
        let i = ((diagonal + d as isize) / 2) as usize;
        let (start_x, start_y, kind) =
            advance(&layers[d - 1], d, i, a.len(), b.len()).expect("valid stored frontier");
        out.extend(std::iter::repeat_n(EditKind::Equal, x - start_x));
        out.push(kind);
        x = start_x - usize::from(kind == EditKind::Delete);
        y = start_y - usize::from(kind == EditKind::Insert);
    }
    out.extend(std::iter::repeat_n(EditKind::Equal, x));
    out[from..].reverse();
}

pub(super) fn script(
    a: &[u32],
    b: &[u32],
    work: &mut Work,
    out: &mut Vec<EditKind>,
) -> Result<(), OperationError> {
    if a.is_empty() || b.is_empty() {
        work.charge(a.len() + b.len())?;
        out.extend(std::iter::repeat_n(EditKind::Delete, a.len()));
        out.extend(std::iter::repeat_n(EditKind::Insert, b.len()));
        return Ok(());
    }
    let (start_x, start_y) = snake(a, b, 0, 0, work)?;
    if start_x == a.len() && start_y == b.len() {
        out.extend(std::iter::repeat_n(EditKind::Equal, start_x));
        return Ok(());
    }
    let mut layers = vec![vec![start_x as i32]];
    let mut retained = 1;
    for d in 1..=a.len() + b.len() {
        retained += d + 1;
        if retained * size_of::<i32>() > TRACE_BYTES {
            return Err(resource("Myers traceback exceeds its scratch budget"));
        }
        let mut layer = vec![-1; d + 1];
        for (i, cell) in layer.iter_mut().enumerate() {
            work.charge(1)?;
            let Some((x, y, _)) = advance(&layers[d - 1], d, i, a.len(), b.len()) else {
                continue;
            };
            let (x, y) = snake(a, b, x, y, work)?;
            *cell = x as i32;
            if x == a.len() && y == b.len() {
                // Traceback needs only the earlier frontiers and the final distance.
                layers.push(layer);
                work.charge(a.len() + b.len())?;
                traceback(a, b, &layers, out);
                return Ok(());
            }
        }
        layers.push(layer);
    }
    Err(invalid("no exact edit path"))
}
