use crate::ast::*;
use crate::sema::types::Ty;
use crate::sema::TypeError;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub usize);

#[derive(Debug, Clone, Default)]
struct Scope {
    values: HashMap<String, DefId>,
    types: HashMap<String, DefId>,
    effects: HashMap<String, DefId>,
}

#[derive(Debug, Clone)]
struct Definition {
    name: String,
    kind: DefKind,
    span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DefKind {
    Variable,
    Function,
    Type,
    Effect,
    Agent,
    Section,
    Import,
}

pub struct Resolver {
    scopes: Vec<Scope>,
    definitions: Vec<Definition>,
    next_id: usize,
    affine_usage: HashMap<DefId, usize>,
    errors: Vec<TypeError>,
}

impl Resolver {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut resolver = Self {
            scopes: vec![Scope::default()],
            definitions: Vec::new(),
            next_id: 0,
            affine_usage: HashMap::new(),
            errors: Vec::new(),
        };
        for name in &[
            "bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64", "char",
            "string", "Self",
        ] {
            resolver.define_type(name, DefKind::Type, SourceSpan::new(0.into(), 0));
        }
        for name in &["network", "fileio", "inference", "spawn", "decision"] {
            resolver.define_effect(name, SourceSpan::new(0.into(), 0));
        }
        resolver
    }

    fn alloc_id(&mut self) -> DefId {
        let id = DefId(self.next_id);
        self.next_id += 1;
        id
    }
    fn define_value(&mut self, name: &str, kind: DefKind, span: SourceSpan) -> DefId {
        let id = self.alloc_id();
        self.scopes
            .last_mut()
            .unwrap()
            .values
            .insert(name.to_string(), id);
        self.definitions.push(Definition {
            name: name.to_string(),
            kind,
            span,
        });
        id
    }
    fn define_type(&mut self, name: &str, kind: DefKind, span: SourceSpan) -> DefId {
        let id = self.alloc_id();
        self.scopes
            .last_mut()
            .unwrap()
            .types
            .insert(name.to_string(), id);
        self.definitions.push(Definition {
            name: name.to_string(),
            kind,
            span,
        });
        id
    }
    fn define_effect(&mut self, name: &str, span: SourceSpan) -> DefId {
        let id = self.alloc_id();
        self.scopes
            .last_mut()
            .unwrap()
            .effects
            .insert(name.to_string(), id);
        self.definitions.push(Definition {
            name: name.to_string(),
            kind: DefKind::Effect,
            span,
        });
        id
    }
    fn lookup_value(&self, name: &str) -> Option<DefId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.values.get(name).copied())
    }
    fn lookup_type(&self, name: &str) -> Option<DefId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.types.get(name).copied())
    }
    fn lookup_effect(&self, name: &str) -> Option<DefId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.effects.get(name).copied())
    }
    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn mark_affine_used(&mut self, id: DefId) -> Result<(), TypeError> {
        let count = self.affine_usage.entry(id).or_insert(0);
        if *count >= 2 {
            let def = &self.definitions[id.0];
            return Err(TypeError::AffineViolation {
                name: def.name.clone(),
                span: def.span,
            });
        }
        *count += 1;
        Ok(())
    }

    pub fn resolve(mut self, program: Program) -> Result<Program, TypeError> {
        for item in &program.items {
            self.resolve_top_level(item)?;
        }
        if !self.errors.is_empty() {
            return Err(self.errors.remove(0));
        }
        Ok(program)
    }

    fn resolve_top_level(&mut self, item: &TopLevelItem) -> Result<(), TypeError> {
        match item {
            TopLevelItem::Agent(a) => {
                self.define_type(&a.name.name, DefKind::Agent, a.name.span);
                Ok(())
            }
            TopLevelItem::Section(s) => {
                self.define_type(&s.name.name, DefKind::Section, s.name.span);
                Ok(())
            }
            TopLevelItem::Fn(f) => {
                self.define_value(&f.name.name, DefKind::Function, f.name.span);
                self.resolve_fn(f)
            }
            TopLevelItem::Seed(s) => {
                for sec in &s.sections {
                    self.define_type(&sec.name.name, DefKind::Section, sec.name.span);
                }
                Ok(())
            }
            TopLevelItem::Struct(s) => {
                self.define_type(&s.name.name, DefKind::Type, s.name.span);
                Ok(())
            }
            TopLevelItem::Enum(e) => {
                self.define_type(&e.name.name, DefKind::Type, e.name.span);
                Ok(())
            }
            TopLevelItem::Mod(m) => {
                self.push_scope();
                if let Some(items) = &m.items {
                    for i in items {
                        self.resolve_top_level(i)?;
                    }
                }
                self.pop_scope();
                Ok(())
            }
            TopLevelItem::Use(u) => {
                let _ = self.resolve_use(u);
                Ok(())
            }
            TopLevelItem::Effect(e) => {
                self.define_effect(&e.name.name, e.name.span);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_fn(&mut self, f: &FnDecl) -> Result<(), TypeError> {
        self.push_scope();
        for p in &f.params {
            self.define_value(&p.name.name, DefKind::Variable, p.name.span);
        }
        if let Some(body) = &f.body {
            self.resolve_block(body)?;
        }
        self.pop_scope();
        Ok(())
    }

    fn resolve_block(&mut self, block: &BlockExpr) -> Result<(), TypeError> {
        self.push_scope();
        for stmt in &block.stmts {
            self.resolve_stmt(stmt)?;
        }
        if let Some(last) = &block.last {
            let _ = self.resolve_expr(last);
        }
        self.pop_scope();
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::Let(l) => {
                let _ = self.resolve_expr(&l.init);
                self.define_value("", DefKind::Variable, l.span);
                Ok(())
            }
            Stmt::Expr(e) => {
                let _ = self.resolve_expr(e);
                Ok(())
            }
            Stmt::Return(r) => {
                if let Some(e) = &r.expr {
                    let _ = self.resolve_expr(e);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Option<Ty> {
        match &expr.kind {
            ExprKind::Ident(id) => {
                if let Some(def_id) = self.lookup_value(&id.name) {
                    let _ = self.mark_affine_used(def_id);
                } else {
                    self.errors.push(TypeError::UnresolvedName {
                        name: id.name.clone(),
                        span: id.span,
                    });
                }
                None
            }
            ExprKind::Call(f, args) => {
                let _ = self.resolve_expr(f);
                for a in args {
                    let _ = self.resolve_expr(a);
                }
                None
            }
            ExprKind::Block(b) => {
                let _ = self.resolve_block(b);
                None
            }
            ExprKind::If(i) => {
                let _ = self.resolve_expr(&i.cond);
                let _ = self.resolve_block(&i.then_branch);
                if let Some(eb) = &i.else_branch {
                    match &**eb {
                        ElseBranch::Block(b) => {
                            let _ = self.resolve_block(b);
                        }
                        ElseBranch::If(inner) => {
                            let _ = self.resolve_expr(&Box::new(ExprNode {
                                kind: ExprKind::If(*inner.clone()),
                                span: i.span,
                            }));
                        }
                    }
                }
                None
            }
            ExprKind::Match(m) => {
                let _ = self.resolve_expr(&m.scrutinee);
                for arm in &m.arms {
                    self.push_scope();
                    let _ = self.resolve_expr(&arm.body);
                    self.pop_scope();
                }
                None
            }
            ExprKind::Binary(_, l, r) => {
                let _ = self.resolve_expr(l);
                let _ = self.resolve_expr(r);
                None
            }
            ExprKind::Unary(_, e) => {
                let _ = self.resolve_expr(e);
                None
            }
            ExprKind::Let(l) => {
                let _ = self.resolve_expr(&l.init);
                self.define_value("", DefKind::Variable, l.span);
                None
            }
            _ => None,
        }
    }

    fn resolve_use(&mut self, _use: &UseDecl) -> Result<(), TypeError> {
        Ok(())
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resolve(program: Program) -> Result<Program, TypeError> {
    Resolver::new().resolve(program)
}
