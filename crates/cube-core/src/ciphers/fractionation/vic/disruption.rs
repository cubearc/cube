//! Source triangles, one skipped row, repeated key order and a ragged last row.
use crate::ciphers::transposition::{apply, column_order};
pub(super) fn fill_order(n: usize, key: &[usize]) -> Vec<usize> {
    let width = key.len();
    let height = n.div_ceil(width);
    let mut disrupted = vec![false; n];
    let (mut row, mut rank) = (0, 0);
    while row < height {
        for boundary in key[rank]..width {
            if row >= height {
                break;
            }
            for col in boundary..width {
                if row * width + col < n {
                    disrupted[row * width + col] = true;
                }
            }
            row += 1;
        }
        row += 1;
        rank = (rank + 1) % width;
    }
    (0..n)
        .filter(|&i| !disrupted[i])
        .chain((0..n).filter(|&i| disrupted[i]))
        .collect()
}
pub(super) fn transform(text: &str, key: &[usize], decode: bool) -> String {
    let x: Vec<_> = text.chars().collect();
    let fill = fill_order(x.len(), key);
    let columns = column_order(x.len(), key);
    let (first, second) = if decode {
        (&columns, &fill)
    } else {
        (&fill, &columns)
    };
    let intermediate: Vec<_> = apply(&x, first, true).chars().collect();
    apply(&intermediate, second, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_triangles_skip_full_rows_cycle_and_preserve_ragged_domains() {
        let key = [4, 5, 2, 0, 1, 3, 6, 7];
        let fill = fill_order(88, &key);
        assert_eq!(&fill[..4], &[0, 1, 2, 3]);
        assert_eq!(&fill[22..30], &(32..40).collect::<Vec<_>>());
        for n in 0..=400 {
            let text: String = "0123456789".chars().cycle().take(n).collect();
            let encrypted = transform(&text, &key, false);
            assert_eq!(transform(&encrypted, &key, true), text);
            let mut positions = fill_order(n, &key);
            positions.sort_unstable();
            assert_eq!(positions, (0..n).collect::<Vec<_>>());
        }
    }
}
