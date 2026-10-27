//! Independently inspect module declarations so a new OPERATIONS/PROFILED/ROWS
//! declaration cannot disappear merely because catalog registration was omitted.
use cube_core::operations;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use syn::{
    Expr, Item,
    visit::{self, Visit},
};

fn operation_id(value: &str) -> bool {
    value.contains('.')
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}
struct Ids<'a> {
    constants: &'a BTreeMap<String, String>,
    ids: BTreeSet<String>,
}
impl Ids<'_> {
    fn value(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Lit(value) => {
                if let syn::Lit::Str(value) = &value.lit {
                    Some(value.value())
                } else {
                    None
                }
            }
            Expr::Path(path) => path
                .path
                .segments
                .last()
                .and_then(|s| self.constants.get(&s.ident.to_string()).cloned()),
            _ => None,
        }
    }
}
impl<'ast> Visit<'ast> for Ids<'_> {
    fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
        if expr
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "Descriptor")
        {
            for field in &expr.fields {
                if matches!(&field.member,syn::Member::Named(name) if name=="id")
                    && let Some(value) = self.value(&field.expr).filter(|s| operation_id(s))
                {
                    self.ids.insert(value);
                }
            }
        }
        visit::visit_expr_struct(self, expr);
    }
    fn visit_expr_call(&mut self, expr: &'ast syn::ExprCall) {
        if let Some(value) = expr
            .args
            .first()
            .and_then(|e| self.value(e))
            .filter(|s| operation_id(s))
        {
            self.ids.insert(value);
        }
        visit::visit_expr_call(self, expr);
    }
}
fn sources(path: &Path, root: &Path, result: &mut Vec<(String, syn::File)>) {
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().unwrap() != "operations" {
                sources(&path, root, result);
            }
            continue;
        }
        if path.extension().is_none_or(|s| s != "rs")
            || path.file_name().unwrap() == "operations.rs"
        {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().with_extension("");
        let mut parts = relative
            .iter()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        if parts.last().is_some_and(|p| p == "mod" || p == "lib") {
            parts.pop();
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let file =
            syn::parse_file(&source).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        result.push((parts.join("::"), file));
    }
}
fn imports(tree: &syn::UseTree, prefix: Vec<String>, result: &mut Vec<(Vec<String>, String)>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            imports(&path.tree, prefix, result);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                imports(item, prefix.clone(), result);
            }
        }
        syn::UseTree::Name(name) => {
            let mut prefix = prefix;
            prefix.push(name.ident.to_string());
            result.push((prefix, name.ident.to_string()));
        }
        syn::UseTree::Rename(name) => {
            let mut prefix = prefix;
            prefix.push(name.ident.to_string());
            result.push((prefix, name.rename.to_string()));
        }
        syn::UseTree::Glob(_) => result.push((prefix, "*".into())),
    }
}
fn inspect(path: &Path, all: &mut BTreeSet<String>) {
    let mut files = Vec::new();
    sources(path, path, &mut files);
    let mut qualified = BTreeMap::new();
    for (module, file) in &files {
        for item in &file.items {
            if let Item::Const(item) = item
                && let Expr::Lit(value) = item.expr.as_ref()
                && let syn::Lit::Str(value) = &value.lit
            {
                qualified.insert(format!("{module}::{}", item.ident), value.value());
            }
        }
    }
    for (module, file) in &files {
        let prefix = format!("{module}::");
        let mut constants = qualified
            .iter()
            .filter_map(|(name, value)| {
                name.strip_prefix(&prefix)
                    .filter(|tail| !tail.contains("::"))
                    .map(|tail| (tail.into(), value.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        let mut declarations = Vec::new();
        for item in &file.items {
            if let Item::Use(item) = item {
                imports(&item.tree, Vec::new(), &mut declarations);
            }
        }
        for (segments, alias) in declarations {
            let mut target = module.split("::").map(str::to_owned).collect::<Vec<_>>();
            for (index, segment) in segments.iter().enumerate() {
                match segment.as_str() {
                    "crate" => target.clear(),
                    "super" => {
                        target.pop();
                    }
                    "self" => (),
                    _ => {
                        if index == 0 {
                            target.clear();
                        }
                        target.push(segment.clone());
                    }
                }
            }
            let key = target.join("::");
            if alias == "*" {
                let prefix = format!("{key}::");
                for (name, value) in &qualified {
                    if let Some(name) = name.strip_prefix(&prefix).filter(|s| !s.contains("::")) {
                        constants.insert(name.into(), value.clone());
                    }
                }
            } else if let Some(value) = qualified.get(&key) {
                constants.insert(alias, value.clone());
            }
        }
        let mut ids = Ids {
            constants: &constants,
            ids: BTreeSet::new(),
        };
        for item in &file.items {
            if let Item::Const(item) = item
                && matches!(
                    item.ident.to_string().as_str(),
                    "OPERATIONS" | "PROFILED" | "ROWS"
                )
            {
                ids.visit_expr(&item.expr);
            }
        }
        all.extend(ids.ids);
    }
}
#[test]
fn every_module_owned_operation_is_registered() {
    let mut declared = BTreeSet::new();
    inspect(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut declared,
    );
    let catalog = operations::catalog()
        .into_iter()
        .map(|d| d.id.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        declared, catalog,
        "module declarations and catalog must have exact ID parity"
    );
}
