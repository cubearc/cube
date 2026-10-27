//! Standalone research instrument for #145, not a production solver or Cube API.
//! Compile with rustc --edition=2024 -D warnings; no dependencies or host inputs.
use std::{collections::VecDeque, mem::size_of, time::Instant};
#[derive(Clone)]
struct Constraint {
    a: usize,
    b: usize,
    rows: [u32; 32],
}
#[derive(Clone)]
struct Model {
    domains: Vec<u32>,
    constraints: Vec<Constraint>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Trace {
    kind: u8,
    constraint: u16,
    variable: u8,
    value: u8,
    depth: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    Work,
    SolutionCap,
    Cancelled,
}
struct Run {
    work: u64,
    max_work: u64,
    max_solutions: usize,
    trace_cap: usize,
    trace: Vec<Trace>,
    trace_truncated: bool,
    solutions: Vec<Vec<u8>>,
    cancel_after: Option<u64>,
}
impl Run {
    fn charge(&mut self, n: u64) -> Result<(), Stop> {
        for _ in 0..n {
            if self.cancel_after.is_some_and(|n| self.work == n) {
                return Err(Stop::Cancelled);
            }
            if self.work == self.max_work {
                return Err(Stop::Work);
            }
            self.work += 1;
        }
        Ok(())
    }
    fn record(&mut self, kind: u8, c: usize, v: usize, value: u8, depth: u8) {
        if self.trace.len() < self.trace_cap {
            self.trace.push(Trace {
                kind,
                constraint: c as u16,
                variable: v as u8,
                value,
                depth,
            });
        } else {
            self.trace_truncated = true;
        }
    }
    fn ac3(&mut self, m: &Model, d: &mut [u32], depth: u8) -> Result<bool, Stop> {
        if d.contains(&0) {
            return Ok(false);
        }
        let arcs = 2 * m.constraints.len();
        let mut queued = vec![true; arcs];
        let mut queue: VecDeque<_> = (0..arcs).collect();
        self.charge(arcs as u64)?;
        while let Some(arc) = queue.pop_front() {
            self.charge(1)?;
            queued[arc] = false;
            let index = arc / 2;
            let forward = arc % 2 == 0;
            let c = &m.constraints[index];
            let (a, b) = if forward { (c.a, c.b) } else { (c.b, c.a) };
            let mut changed = false;
            let mut candidates = d[a];
            while candidates != 0 {
                let x = candidates.trailing_zeros() as u8;
                candidates &= candidates - 1;
                let mut supports = d[b];
                let mut supported = false;
                while supports != 0 {
                    let y = supports.trailing_zeros() as u8;
                    supports &= supports - 1;
                    self.charge(1)?;
                    let allowed = if forward {
                        c.rows[x as usize] & (1 << y) != 0
                    } else {
                        c.rows[y as usize] & (1 << x) != 0
                    };
                    if allowed {
                        supported = true;
                        break;
                    }
                }
                if !supported {
                    self.charge(1)?;
                    d[a] &= !(1 << x);
                    changed = true;
                    self.record(0, index, a, x, depth);
                    if d[a] == 0 {
                        return Ok(false);
                    }
                }
            }
            if changed {
                for (i, in_queue) in queued.iter_mut().enumerate() {
                    self.charge(1)?;
                    let other = &m.constraints[i / 2];
                    let to = if i % 2 == 0 { other.b } else { other.a };
                    if to == a && i / 2 != index && !*in_queue {
                        self.charge(1)?;
                        *in_queue = true;
                        queue.push_back(i);
                    }
                }
            }
        }
        Ok(true)
    }
    fn search(&mut self, m: &Model, mut d: Vec<u32>, depth: u8) -> Result<(), Stop> {
        self.charge(1)?;
        if !self.ac3(m, &mut d, depth)? {
            return Ok(());
        }
        if let Some(v) = d.iter().position(|d| d.count_ones() > 1) {
            let mut choices = d[v];
            while choices != 0 {
                let value = choices.trailing_zeros() as u8;
                choices &= choices - 1;
                self.charge(d.len() as u64 + 1)?;
                let mut child = d.clone();
                child[v] = 1 << value;
                self.record(1, usize::from(u16::MAX), v, value, depth);
                self.search(m, child, depth + 1)?;
                self.record(2, usize::from(u16::MAX), v, value, depth);
            }
        } else {
            if self.solutions.len() == self.max_solutions {
                return Err(Stop::SolutionCap);
            }
            self.solutions
                .push(d.iter().map(|d| d.trailing_zeros() as u8).collect());
        }
        Ok(())
    }
}
fn run(
    m: &Model,
    max_work: u64,
    max_solutions: usize,
    trace_cap: usize,
    cancel_after: Option<u64>,
) -> (Run, Option<Stop>) {
    assert!(
        m.domains.len() <= 32
            && m.constraints.len() <= 256
            && max_work <= 1_000_000
            && max_solutions <= 100
            && trace_cap <= 10000
    );
    assert!(
        m.constraints
            .iter()
            .all(|c| c.a < m.domains.len() && c.b < m.domains.len() && c.a != c.b)
    );
    let mut r = Run {
        work: 0,
        max_work,
        max_solutions,
        trace_cap,
        trace: Vec::with_capacity(trace_cap),
        trace_truncated: false,
        solutions: Vec::with_capacity(max_solutions),
        cancel_after,
    };
    let stop = r.search(m, m.domains.clone(), 0).err();
    (r, stop)
}
fn mask(n: u8) -> u32 {
    if n == 32 { u32::MAX } else { (1 << n) - 1 }
}
fn relation(m: &mut Model, a: usize, b: usize, kind: u8, parameter: i32) {
    let mut rows = [0u32; 32];
    for (x, row) in rows.iter_mut().enumerate() {
        for y in 0..32 {
            let (x, y) = (x as i32, y as i32);
            let allowed = match kind {
                0 => x != y,
                1 => x == y,
                2 => y - x == parameter,
                3 => (x - y).abs() == 1,
                4 => x / 3 == y,
                5 => x % 3 == y,
                _ => false,
            };
            if allowed {
                *row |= 1 << y;
            }
        }
    }
    m.constraints.push(Constraint { a, b, rows });
}
fn different(m: &mut Model, vars: &[usize]) {
    for (i, &a) in vars.iter().enumerate() {
        for &b in &vars[i + 1..] {
            relation(m, a, b, 0, 0);
        }
    }
}
fn einstein() -> Model {
    let mut m = Model {
        domains: vec![mask(3); 6],
        constraints: vec![],
    };
    different(&mut m, &[0, 1, 2]);
    different(&mut m, &[3, 4, 5]);
    m.domains[0] = 1;
    m.domains[5] = 4;
    relation(&mut m, 0, 3, 1, 0);
    relation(&mut m, 1, 4, 1, 0);
    relation(&mut m, 0, 1, 2, 1);
    relation(&mut m, 0, 1, 3, 0);
    m
}
fn latin(n: usize, reduced: bool) -> Model {
    let mut m = Model {
        domains: vec![mask(n as u8); n * n],
        constraints: vec![],
    };
    for r in 0..n {
        different(&mut m, &(0..n).map(|c| r * n + c).collect::<Vec<_>>());
    }
    for c in 0..n {
        different(&mut m, &(0..n).map(|r| r * n + c).collect::<Vec<_>>());
    }
    if reduced {
        for i in 0..n {
            m.domains[i] = 1 << i;
            m.domains[i * n] = 1 << i;
        }
    }
    m
}
fn orthogonal(valid: bool) -> Model {
    let mut m = Model {
        domains: vec![mask(3); 27],
        constraints: vec![],
    };
    for offset in [0, 9] {
        for r in 0..3 {
            different(
                &mut m,
                &(0..3).map(|c| offset + r * 3 + c).collect::<Vec<_>>(),
            );
        }
        for c in 0..3 {
            different(
                &mut m,
                &(0..3).map(|r| offset + r * 3 + c).collect::<Vec<_>>(),
            );
        }
    }
    for r in 0..3 {
        for c in 0..3 {
            let i = r * 3 + c;
            m.domains[i] = 1 << ((r + c) % 3);
            m.domains[9 + i] = 1 << ((r + if valid { 2 * c } else { c }) % 3);
            m.domains[18 + i] = mask(9);
            relation(&mut m, 18 + i, i, 4, 0);
            relation(&mut m, 18 + i, 9 + i, 5, 0);
        }
    }
    different(&mut m, &(18..27).collect::<Vec<_>>());
    m
}
fn brute(m: &Model) -> Vec<Vec<u8>> {
    fn visit(m: &Model, a: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
        if a.len() == m.domains.len() {
            if m.constraints
                .iter()
                .all(|c| c.rows[a[c.a] as usize] & (1 << a[c.b]) != 0)
            {
                out.push(a.clone());
            }
            return;
        }
        let mut values = m.domains[a.len()];
        while values != 0 {
            let x = values.trailing_zeros() as u8;
            values &= values - 1;
            a.push(x);
            visit(m, a, out);
            a.pop();
        }
    }
    let mut out = Vec::new();
    visit(m, &mut Vec::new(), &mut out);
    out
}
fn main() {
    // Tiny exhaustive oracle is independent of propagation and backtracking pruning.
    let mut checks = 0;
    for table in 0u32..512 {
        let mut rows = [0; 32];
        for (x, row) in rows.iter_mut().take(3).enumerate() {
            *row = (table >> (x * 3)) & 7;
        }
        let m = Model {
            domains: vec![7; 2],
            constraints: vec![Constraint { a: 0, b: 1, rows }],
        };
        let expected = brute(&m);
        let (r, stop) = run(&m, 1_000_000, 100, 10000, None);
        assert_eq!(stop, None);
        assert_eq!(r.solutions, expected);
        let mut reduced = m.domains.clone();
        let mut p = run(
            &Model {
                domains: vec![],
                constraints: vec![],
            },
            1_000_000,
            100,
            0,
            None,
        )
        .0;
        let stable = p.ac3(&m, &mut reduced, 0).unwrap();
        assert_eq!(stable, !expected.is_empty());
        for a in &expected {
            assert!(reduced[0] & (1 << a[0]) != 0 && reduced[1] & (1 << a[1]) != 0);
        }
        checks += 1;
    }
    let families = [
        ("einstein3", einstein()),
        ("latin3", latin(3, false)),
        ("latin3_reduced", latin(3, true)),
        ("latin4_reduced", latin(4, true)),
        ("orthogonal3", orthogonal(true)),
        ("nonorthogonal3", orthogonal(false)),
    ];
    for (name, m) in families {
        let now = Instant::now();
        let (r, stop) = run(&m, 1_000_000, 100, 10000, None);
        let solve_us = now.elapsed().as_micros();
        assert_eq!(stop, None);
        let expected = match name {
            "latin3" => 12,
            "latin4_reduced" => 4,
            "nonorthogonal3" => 0,
            _ => 1,
        };
        assert_eq!(r.solutions.len(), expected);
        if m.domains
            .iter()
            .map(|d| u64::from(d.count_ones()))
            .product::<u64>()
            <= 300_000
        {
            assert_eq!(r.solutions, brute(&m));
        }
        let (again, again_stop) = run(&m, 1_000_000, 100, 10000, None);
        assert_eq!(again_stop, stop);
        assert_eq!(again.trace, r.trace);
        assert_eq!(again.work, r.work);
        let used_trace_fields: usize = r
            .trace
            .iter()
            .map(|t| {
                usize::from(t.kind)
                    + usize::from(t.constraint)
                    + usize::from(t.variable)
                    + usize::from(t.value)
                    + usize::from(t.depth)
            })
            .sum();
        let memory_bound = m.constraints.len() * size_of::<Constraint>()
            + 33 * 32 * 4
            + 2 * m.constraints.len() * 32
            + 10000 * size_of::<Trace>()
            + 100 * 32
            + 8 * 1024 * 1024;
        let first = r
            .solutions
            .first()
            .map(|v| format!("{v:?}"))
            .unwrap_or_else(|| "null".into());
        println!(
            "{{\"case\":\"{name}\",\"variables\":{},\"relations\":{},\"solutions\":{},\"work\":{},\"trace_records\":{},\"trace_truncated\":{},\"elapsed_us\":{},\"memory_bound_bytes\":{},\"trace_checksum\":{},\"first\":{first}}}",
            m.domains.len(),
            m.constraints.len(),
            r.solutions.len(),
            r.work,
            r.trace.len(),
            r.trace_truncated,
            solve_us,
            memory_bound,
            used_trace_fields,
        );
        if r.work > 0 {
            assert_eq!(run(&m, r.work - 1, 100, 10000, None).1, Some(Stop::Work));
            assert_eq!(run(&m, r.work, 100, 10000, None).1, None);
        }
        let (a, _) = run(&m, 1_000_000, 100, 1, None);
        if r.trace.len() > 1 {
            assert!(a.trace_truncated);
        }
        assert_eq!(a.solutions, r.solutions);
    }
    let m = einstein();
    let (r, stop) = run(&m, 1_000_000, 100, 10000, None);
    assert_eq!(stop, None);
    assert_eq!(r.solutions[0], vec![0, 1, 2, 0, 1, 2]);
    let (r, stop) = run(&latin(3, true), 1_000_000, 100, 10000, None);
    assert_eq!(stop, None);
    assert_eq!(r.solutions[0], vec![0, 1, 2, 1, 2, 0, 2, 0, 1]);
    assert_eq!(
        run(&latin(3, false), 1_000_000, 1, 10000, None).1,
        Some(Stop::SolutionCap)
    );
    let (cancelled, stop) = run(&latin(3, false), 1_000_000, 100, 10000, Some(10));
    assert_eq!(stop, Some(Stop::Cancelled));
    assert_eq!(cancelled.work, 10);
    let mut empty = einstein();
    empty.domains[1] = 0;
    let (r, stop) = run(&empty, 1_000_000, 100, 10000, None);
    assert_eq!(stop, None);
    assert!(r.solutions.is_empty());
    let mut triangle = Model {
        domains: vec![3; 3],
        constraints: vec![],
    };
    different(&mut triangle, &[0, 1, 2]);
    let mut stable = triangle.domains.clone();
    let (mut p, _) = run(
        &Model {
            domains: vec![],
            constraints: vec![],
        },
        1_000_000,
        100,
        0,
        None,
    );
    assert!(p.ac3(&triangle, &mut stable, 0).unwrap());
    assert!(brute(&triangle).is_empty());
    let (r, stop) = run(&triangle, 1_000_000, 100, 10000, None);
    assert_eq!(stop, None);
    assert!(r.solutions.is_empty());
    let max = Model {
        domains: vec![u32::MAX; 32],
        constraints: vec![],
    };
    assert_eq!(run(&max, 1_000_000, 1, 1, None).1, Some(Stop::SolutionCap));
    println!(
        "oracle_checks={checks}; exact/one-over work, capped traces, solution cap, empty domain, 32x32 and cancellation checks passed"
    );
}
