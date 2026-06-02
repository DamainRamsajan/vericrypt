//! Effect checking for AGENT‑SEED v15.2.

use crate::ast::*;
use crate::sema::types::*;
use crate::sema::TypeError;
use std::collections::HashSet;

pub struct EffectChecker {
    inside_discharge: bool,
    accumulated: HashSet<Effect>,
    errors: Vec<TypeError>,
}

impl EffectChecker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inside_discharge: false,
            accumulated: HashSet::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> HashSet<Effect> {
        match &expr.kind {
            ExprKind::Perform(op, args) => {
                let mut effects = HashSet::new();
                effects.insert(Effect::Named(op.name.clone()));
                for a in args {
                    self.check_expr(a);
                }
                if !self.inside_discharge {
                    self.errors.push(TypeError::UndischargedEffect {
                        effect: op.name.clone(),
                        span: op.span,
                    });
                }
                effects
            }
            ExprKind::Discharge(scrutinee, thresholds) => {
                self.check_expr(scrutinee);
                let prev = self.inside_discharge;
                self.inside_discharge = true;
                for (_, body) in thresholds {
                    self.check_expr(&Box::new(ExprNode {
                        kind: ExprKind::Block(body.clone()),
                        span: expr.span,
                    }));
                }
                self.inside_discharge = prev;
                HashSet::new()
            }
            ExprKind::Call(f, args) => {
                self.check_expr(f);
                for a in args {
                    self.check_expr(a);
                }
                HashSet::new()
            }
            ExprKind::Binary(_, l, r) => {
                let mut e = self.check_expr(l);
                e.extend(self.check_expr(r));
                e
            }
            ExprKind::Unary(_, e) => self.check_expr(e),
            ExprKind::If(i) => {
                self.check_expr(&i.cond);
                self.check_expr(&Box::new(ExprNode {
                    kind: ExprKind::Block(i.then_branch.clone()),
                    span: i.span,
                }));
                if let Some(eb) = &i.else_branch {
                    match &**eb {
                        ElseBranch::Block(b) => {
                            self.check_expr(&Box::new(ExprNode {
                                kind: ExprKind::Block(b.clone()),
                                span: i.span,
                            }));
                        }
                        ElseBranch::If(inner) => {
                            self.check_expr(&Box::new(ExprNode {
                                kind: ExprKind::If(*inner.clone()),
                                span: inner.span,
                            }));
                        }
                    }
                }
                HashSet::new()
            }
            ExprKind::Block(b) => {
                let mut effects = HashSet::new();
                for stmt in &b.stmts {
                    effects.extend(self.check_stmt(stmt));
                }
                if let Some(last) = &b.last {
                    effects.extend(self.check_expr(last));
                }
                effects
            }
            ExprKind::Spawn(e) => {
                let mut effects = self.check_expr(e);
                effects.insert(Effect::AgentSpawn);
                effects
            }
            ExprKind::Let(l) => {
                self.check_expr(&l.init);
                HashSet::new()
            }
            // fix: `r` is the Option<Expr>, not a struct with `.expr`
            ExprKind::Return(r) => {
                if let Some(e) = r {
                    self.check_expr(e);
                }
                HashSet::new()
            }
            _ => HashSet::new(),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> HashSet<Effect> {
        match stmt {
            Stmt::Let(l) => self.check_expr(&Box::new(ExprNode {
                kind: ExprKind::Let(l.clone()),
                span: l.span,
            })),
            Stmt::Expr(e) => self.check_expr(e),
            Stmt::Return(r) => {
                if let Some(e) = &r.expr {
                    self.check_expr(e);
                }
                HashSet::new()
            }
            _ => HashSet::new(),
        }
    }
}

impl Default for EffectChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_effects(program: Program) -> Result<Program, TypeError> {
    let mut checker = EffectChecker::new();
    for item in &program.items {
        if let TopLevelItem::Fn(f) = item {
            if let Some(body) = &f.body {
                checker.check_expr(&Box::new(ExprNode {
                    kind: ExprKind::Block(body.clone()),
                    span: f.span,
                }));
            }
        }
    }
    if !checker.errors.is_empty() {
        return Err(checker.errors.remove(0));
    }
    Ok(program)
}
