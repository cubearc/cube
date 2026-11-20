//! Iterative search and atomic capture-avoiding rewrites over flat term arenas.
use super::{Arena, Atom, Id, Kind, MAX_LIVE, Mode, Options, Reason, Rule, Strategy};
use crate::formal_work::Work;

#[derive(Clone, Copy)]
pub(super) enum Redex {
    Beta { body: Id, argument: Id },
    Eta(Id),
    I(Id),
    K(Id),
    S { x: Id, y: Id, z: Id },
}
impl Redex {
    pub(super) fn rule(self) -> Rule {
        match self {
            Self::Beta { .. } => Rule::Beta,
            Self::Eta(_) => Rule::Eta,
            Self::I(_) => Rule::I,
            Self::K(_) => Rule::K,
            Self::S { .. } => Rule::S,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Found {
    at: Id,
    pub(super) preorder: usize,
    pub(super) redex: Redex,
}

enum Scan {
    Enter(Id),
    Exit(Id, usize),
}

pub(super) fn find(
    arena: &Arena,
    root: Id,
    options: &Options<'_>,
    work: &mut Work<'_>,
) -> Result<Option<Found>, Reason> {
    let innermost = matches!(options.strategy, Strategy::ApplicativeOrder);
    let mut stack = vec![Scan::Enter(root)];
    let mut serial = 0;
    while let Some(item) = stack.pop() {
        work.charge(1)?;
        let (id, preorder) = match item {
            Scan::Enter(id) => (id, serial),
            Scan::Exit(id, preorder) => {
                if let Some(redex) = redex(arena, id, options, work)? {
                    return Ok(Some(Found {
                        at: id,
                        preorder,
                        redex,
                    }));
                }
                continue;
            }
        };
        serial += 1;
        if innermost {
            stack.push(Scan::Exit(id, preorder));
        } else if let Some(redex) = redex(arena, id, options, work)? {
            return Ok(Some(Found {
                at: id,
                preorder,
                redex,
            }));
        }
        match arena.get(id).kind {
            Kind::Atom(_) => (),
            Kind::Abs(body) => stack.push(Scan::Enter(body)),
            Kind::App(left, right) => stack.extend([Scan::Enter(right), Scan::Enter(left)]),
        }
    }
    Ok(None)
}

fn redex(
    arena: &Arena,
    at: Id,
    options: &Options<'_>,
    work: &mut Work<'_>,
) -> Result<Option<Redex>, Reason> {
    let kind = arena.get(at).kind;
    if matches!(options.mode, Mode::Ski { .. }) {
        return Ok(ski_redex(arena, kind));
    }
    if let Kind::App(left, argument) = kind
        && let Kind::Abs(body) = arena.get(left).kind
    {
        return Ok(Some(Redex::Beta { body, argument }));
    }
    if options.eta
        && let Kind::Abs(body) = kind
        && let Kind::App(candidate, argument) = arena.get(body).kind
        && matches!(arena.get(argument).kind, Kind::Atom(Atom::Bound(0)))
        && !uses_binder(arena, candidate, work)?
    {
        return Ok(Some(Redex::Eta(candidate)));
    }
    Ok(None)
}

fn ski_redex(arena: &Arena, kind: Kind) -> Option<Redex> {
    let Kind::App(left, argument) = kind else {
        return None;
    };
    let head = arena.get(left).kind;
    if matches!(head, Kind::Atom(Atom::I)) {
        return Some(Redex::I(argument));
    }
    let Kind::App(left, b) = head else {
        return None;
    };
    let head = arena.get(left).kind;
    if matches!(head, Kind::Atom(Atom::K)) {
        return Some(Redex::K(b));
    }
    let Kind::App(left, a) = head else {
        return None;
    };
    matches!(arena.get(left).kind, Kind::Atom(Atom::S)).then_some(Redex::S {
        x: a,
        y: b,
        z: argument,
    })
}

fn uses_binder(arena: &Arena, id: Id, work: &mut Work<'_>) -> Result<bool, Reason> {
    let mut pending = vec![(id, 0)];
    while let Some((id, depth)) = pending.pop() {
        work.charge(1)?;
        match arena.get(id).kind {
            Kind::Atom(Atom::Bound(index)) if index == depth => return Ok(true),
            Kind::Atom(_) => (),
            Kind::Abs(body) => pending.push((body, depth + 1)),
            Kind::App(left, right) => pending.extend([(right, depth), (left, depth)]),
        }
    }
    Ok(false)
}

#[derive(Clone, Copy)]
enum Transform {
    Copy,
    Substitute { argument: Id, depth: u32 },
    Shift { cutoff: u32, delta: i32 },
}

impl Transform {
    fn under_binder(self) -> Self {
        match self {
            Self::Copy => Self::Copy,
            Self::Substitute { argument, depth } => Self::Substitute {
                argument,
                depth: depth + 1,
            },
            Self::Shift { cutoff, delta } => Self::Shift {
                cutoff: cutoff + 1,
                delta,
            },
        }
    }
}

enum Job {
    Visit(Id, Transform),
    FinishAbs,
    FinishApp,
}

fn replace(tasks: &mut Vec<Job>, redex: Redex) {
    let copy = |id| Job::Visit(id, Transform::Copy);
    match redex {
        Redex::Beta { body, argument } => tasks.push(Job::Visit(
            body,
            Transform::Substitute { argument, depth: 0 },
        )),
        Redex::Eta(term) => tasks.push(Job::Visit(
            term,
            Transform::Shift {
                cutoff: 0,
                delta: -1,
            },
        )),
        Redex::K(term) | Redex::I(term) => tasks.push(copy(term)),
        Redex::S { x, y, z } => tasks.extend([
            Job::FinishApp,
            Job::FinishApp,
            copy(z),
            copy(y),
            Job::FinishApp,
            copy(z),
            copy(x),
        ]),
    }
}

pub(super) fn rewrite(
    old: &Arena,
    root: Id,
    found: Found,
    work: &mut Work<'_>,
) -> Result<(Arena, Id), Reason> {
    let mut arena = Arena::new(MAX_LIVE);
    let mut tasks = vec![Job::Visit(root, Transform::Copy)];
    let mut values = Vec::new();
    while let Some(job) = tasks.pop() {
        work.charge(1)?;
        let kind = match job {
            Job::FinishApp => {
                let right = values.pop().expect("application right");
                let left = values.pop().expect("application left");
                Kind::App(left, right)
            }
            Job::FinishAbs => Kind::Abs(values.pop().expect("abstraction body")),
            Job::Visit(id, _) if id == found.at => {
                replace(&mut tasks, found.redex);
                continue;
            }
            Job::Visit(id, transform) => match old.get(id).kind {
                Kind::Atom(Atom::Bound(index)) => match transform {
                    Transform::Substitute { argument, depth } if index == depth => {
                        tasks.push(Job::Visit(
                            argument,
                            Transform::Shift {
                                cutoff: 0,
                                delta: depth as i32,
                            },
                        ));
                        continue;
                    }
                    Transform::Substitute { depth, .. } if index > depth => {
                        Kind::Atom(Atom::Bound(index - 1))
                    }
                    Transform::Shift { cutoff, delta } if index >= cutoff => {
                        Kind::Atom(Atom::Bound(
                            index
                                .checked_add_signed(delta)
                                .expect("scope-preserving shift"),
                        ))
                    }
                    _ => Kind::Atom(Atom::Bound(index)),
                },
                atom @ Kind::Atom(_) => atom,
                Kind::Abs(body) => {
                    tasks.extend([Job::FinishAbs, Job::Visit(body, transform.under_binder())]);
                    continue;
                }
                Kind::App(left, right) => {
                    tasks.extend([
                        Job::FinishApp,
                        Job::Visit(right, transform),
                        Job::Visit(left, transform),
                    ]);
                    continue;
                }
            },
        };
        values.push(arena.push(kind)?);
    }
    let result = values.pop().expect("one rewritten term");
    Ok((arena, result))
}
