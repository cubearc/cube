//! Line anchors for Patience and Histogram with exact Myers fallback.
use super::*;
use std::collections::BTreeMap;
use std::ops::Range;

#[derive(Clone, Copy)]
struct Anchor {
    left: usize,
    right: usize,
    len: usize,
}

type Slots = BTreeMap<u32, Vec<usize>>;
fn slots(lines: &[u32], work: &mut Work) -> Result<Slots, OperationError> {
    let mut slots = Slots::new();
    for (i, &line) in lines.iter().enumerate() {
        work.charge(1)?;
        slots.entry(line).or_default().push(i);
    }
    Ok(slots)
}

fn patience(a: &[u32], b: &[u32], work: &mut Work) -> Result<Vec<Anchor>, OperationError> {
    let (left, right) = (slots(a, work)?, slots(b, work)?);
    let mut anchors = Vec::new();
    for (i, line) in a.iter().enumerate() {
        work.charge(2)?;
        if left[line].len() != 1 {
            continue;
        }
        if let Some(&[j]) = right.get(line).map(Vec::as_slice) {
            anchors.push(Anchor {
                left: i,
                right: j,
                len: 1,
            });
        }
    }
    let mut tree = vec![0; b.len() + 1];
    let mut suffix = vec![0; anchors.len()];
    let mut remaining = 0;
    for (i, anchor) in anchors.iter().enumerate().rev() {
        let index = b.len() - anchor.right;
        let mut at = index - 1;
        while at > 0 {
            work.charge(1)?;
            suffix[i] = suffix[i].max(tree[at]);
            at &= at - 1;
        }
        suffix[i] += 1;
        remaining = remaining.max(suffix[i]);
        let mut at = index;
        while at < tree.len() {
            work.charge(1)?;
            tree[at] = tree[at].max(suffix[i]);
            at += at.isolate_lowest_one();
        }
    }
    let mut start = 0;
    Ok(anchors
        .into_iter()
        .zip(suffix)
        .filter_map(|(anchor, len)| {
            if remaining > 0 && len >= remaining && anchor.right >= start {
                remaining -= 1;
                start = anchor.right + 1;
                Some(anchor)
            } else {
                None
            }
        })
        .collect())
}

fn extend(
    a: &[u32],
    b: &[u32],
    i: usize,
    j: usize,
    slots: &Slots,
    work: &mut Work,
) -> Result<(Anchor, usize), OperationError> {
    let (mut l, mut r) = (i, j);
    let (mut x, mut y) = (i + 1, j + 1);
    let mut rarity = slots[&a[i]].len();
    while l > 0 && r > 0 {
        work.charge(1)?;
        if a[l - 1] != b[r - 1] {
            break;
        }
        l -= 1;
        r -= 1;
        rarity = rarity.min(slots[&a[l]].len());
    }
    while x < a.len() && y < b.len() {
        work.charge(1)?;
        if a[x] != b[y] {
            break;
        }
        rarity = rarity.min(slots[&a[x]].len());
        x += 1;
        y += 1;
    }
    Ok((
        Anchor {
            left: l,
            right: r,
            len: x - l,
        },
        rarity,
    ))
}

fn histogram(
    a: &[u32],
    b: &[u32],
    cap: usize,
    work: &mut Work,
) -> Result<Vec<Anchor>, OperationError> {
    let slots = slots(a, work)?;
    let mut best: Option<Anchor> = None;
    let mut rare = cap;
    for (j, line) in b.iter().enumerate() {
        work.charge(1)?;
        let Some(positions) = slots.get(line).filter(|positions| positions.len() <= rare) else {
            continue;
        };
        for &i in positions {
            work.charge(1)?;
            let (candidate, rarity) = extend(a, b, i, j, &slots, work)?;
            let choose = best.is_none_or(|old| {
                rarity < rare
                    || candidate.len > old.len
                    || rarity == rare
                        && candidate.len == old.len
                        && (candidate.left, candidate.right) < (old.left, old.right)
            });
            if choose {
                best = Some(candidate);
                rare = rarity;
            }
        }
    }
    Ok(best.into_iter().collect())
}

enum Job {
    Compare(Range<usize>, Range<usize>),
    Equal(usize),
}

pub(super) fn script(
    a: &[u32],
    b: &[u32],
    method: Method,
    cap: usize,
    work: &mut Work,
    out: &mut Vec<EditKind>,
) -> Result<(), OperationError> {
    let mut jobs = vec![Job::Compare(0..a.len(), 0..b.len())];
    while let Some(job) = jobs.pop() {
        let Job::Compare(left, right) = job else {
            if let Job::Equal(n) = job {
                out.extend(std::iter::repeat_n(EditKind::Equal, n));
            }
            continue;
        };
        let (l, r) = (&a[left.clone()], &b[right.clone()]);
        // Identical ends naturally match and avoid work for repetitive files.
        let mut prefix = 0;
        while prefix < l.len().min(r.len()) {
            work.charge(1)?;
            if l[prefix] != r[prefix] {
                break;
            }
            prefix += 1;
        }
        out.extend(std::iter::repeat_n(EditKind::Equal, prefix));
        let left = left.start + prefix..left.end;
        let right = right.start + prefix..right.end;
        let (l, r) = (&a[left.clone()], &b[right.clone()]);
        let mut suffix = 0;
        while suffix < l.len().min(r.len()) {
            work.charge(1)?;
            if l[l.len() - 1 - suffix] != r[r.len() - 1 - suffix] {
                break;
            }
            suffix += 1;
        }
        jobs.push(Job::Equal(suffix));
        let left = left.start..left.end - suffix;
        let right = right.start..right.end - suffix;
        let (l, r) = (&a[left.clone()], &b[right.clone()]);
        if l.is_empty() || r.is_empty() {
            myers::script(l, r, work, out)?;
            continue;
        }
        let anchors = match method {
            Method::Patience => patience(l, r, work)?,
            Method::Histogram => histogram(l, r, cap, work)?,
            Method::Myers => unreachable!("line anchor method"),
        };
        if anchors.is_empty() {
            myers::script(l, r, work, out)?;
            continue;
        }
        let (mut x, mut y) = (left.end, right.end);
        for anchor in anchors.into_iter().rev() {
            let (start_x, start_y) = (left.start + anchor.left, right.start + anchor.right);
            jobs.push(Job::Compare(
                start_x + anchor.len..x,
                start_y + anchor.len..y,
            ));
            jobs.push(Job::Equal(anchor.len));
            (x, y) = (start_x, start_y);
        }
        jobs.push(Job::Compare(left.start..x, right.start..y));
    }
    Ok(())
}
