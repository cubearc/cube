//! Small ASCII grammar with explicit binder scope and bounded parser recursion.
use super::{Arena, Atom, Id, Kind, Mode};
use crate::{
    errors::OperationError,
    formal_work::{invalid, resource},
};
use std::collections::BTreeMap;

const MAX_NODES: usize = 4096;
const MAX_DEPTH: usize = 128;
const MAX_IDENTIFIER: usize = 64;

pub(super) struct Parsed {
    pub arena: Arena,
    pub root: Id,
    pub names: Vec<String>,
}

pub(super) fn parse(source: &str, mode: Mode) -> Result<Parsed, OperationError> {
    if source.len() > 512 << 10 {
        return Err(resource("term text exceeds 512 KiB"));
    }
    if !source.is_ascii() {
        return Err(invalid(
            "term grammar uses exact ASCII identifiers and punctuation",
        ));
    }
    let mut parser = Parser {
        input: source.as_bytes(),
        cursor: 0,
        mode,
        arena: Arena::new(MAX_NODES),
        binders: Vec::new(),
        names: Vec::new(),
        free: BTreeMap::new(),
    };
    let root = parser.expression(1)?;
    if parser.peek().is_some() {
        return Err(parser.error("unexpected trailing term token"));
    }
    Ok(Parsed {
        arena: parser.arena,
        root,
        names: parser.names,
    })
}

struct Parser<'a> {
    input: &'a [u8],
    cursor: usize,
    mode: Mode,
    arena: Arena,
    binders: Vec<&'a str>,
    names: Vec<String>,
    free: BTreeMap<&'a str, u32>,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &str) -> OperationError {
        invalid(message).offset(self.cursor as u64)
    }

    fn peek(&mut self) -> Option<u8> {
        while self
            .input
            .get(self.cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.cursor += 1;
        }
        self.input.get(self.cursor).copied()
    }

    fn take(&mut self, token: u8) -> bool {
        if self.peek() == Some(token) {
            self.cursor += 1;
            return true;
        }
        false
    }

    fn checked(identifier: u8) -> bool {
        identifier == b'_' || identifier.is_ascii_alphabetic()
    }

    fn name(&mut self) -> Result<&'a str, OperationError> {
        if !self.peek().is_some_and(Self::checked) {
            return Err(self.error("expected an ASCII identifier"));
        }
        let start = self.cursor;
        self.cursor += 1;
        while self
            .input
            .get(self.cursor)
            .is_some_and(|&x| Self::checked(x) || x.is_ascii_digit())
        {
            self.cursor += 1;
        }
        if self.cursor - start > MAX_IDENTIFIER {
            return Err(resource("term identifier exceeds 64 bytes"));
        }
        Ok(std::str::from_utf8(&self.input[start..self.cursor]).expect("checked ASCII"))
    }

    fn push(&mut self, kind: Kind) -> Result<Id, OperationError> {
        let id = self
            .arena
            .push(kind)
            .map_err(|_| resource("input term exceeds 4096 nodes"))?;
        if self.arena.get(id).height > MAX_DEPTH {
            return Err(resource("input term exceeds depth 128"));
        }
        Ok(id)
    }

    fn expression(&mut self, depth: usize) -> Result<Id, OperationError> {
        if depth > MAX_DEPTH {
            return Err(resource("input term syntax exceeds depth 128"));
        }
        if matches!(self.mode, Mode::Lambda) && self.take(b'\\') {
            let name = self.name()?;
            if !self.take(b'.') {
                return Err(self.error("abstraction binder must be followed by a dot"));
            }
            self.binders.push(name);
            let body = self.expression(depth + 1)?;
            self.binders.pop();
            return self.push(Kind::Abs(body));
        }
        let mut head = self.atom(depth)?;
        while self.peek().is_some_and(|x| x == b'(' || Self::checked(x)) {
            let right = self.atom(depth)?;
            head = self.push(Kind::App(head, right))?;
        }
        Ok(head)
    }

    fn atom(&mut self, depth: usize) -> Result<Id, OperationError> {
        if self.take(b'(') {
            let term = self.expression(depth + 1)?;
            if !self.take(b')') {
                return Err(self.error("expected closing parenthesis"));
            }
            return Ok(term);
        }
        let name = self.name()?;
        let kind = match self.mode {
            Mode::Lambda => self
                .binders
                .iter()
                .rev()
                .position(|&binder| binder == name)
                .map_or_else(
                    || self.free_atom(name),
                    |position| Kind::Atom(Atom::Bound(position as u32)),
                ),
            Mode::Ski { free_symbols } => match name {
                "S" => Kind::Atom(Atom::S),
                "K" => Kind::Atom(Atom::K),
                "I" => Kind::Atom(Atom::I),
                _ if free_symbols => self.free_atom(name),
                _ => return Err(self.error("SKI requires S, K, I or explicit free_symbols")),
            },
        };
        self.push(kind)
    }

    fn free_atom(&mut self, name: &'a str) -> Kind {
        let index = *self.free.entry(name).or_insert_with(|| {
            self.names.push(name.to_owned());
            (self.names.len() - 1) as u32
        });
        Kind::Atom(Atom::Free(index))
    }
}
