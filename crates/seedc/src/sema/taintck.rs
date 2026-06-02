//! Taint analysis for AGENT‑SEED v15.2.

use crate::ast::*;
use crate::sema::types::*;
use crate::sema::TypeError;
use std::collections::HashMap;

pub struct TaintChecker {
    pc_level: TaintLevel,
    var_taint: HashMap<String, TaintLevel>,
    errors: Vec<TypeError>,
}

impl TaintChecker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            pc_level: TaintLevel::Clean,
            var_taint: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> TaintLevel {
        match &expr.kind {
            ExprKind::Lit(_) => self.pc_level,
            ExprKind::Ident(id) => self
                .var_taint
                .get(&id.name)
                .copied()
                .unwrap_or(self.pc_level),
            ExprKind::Binary(_, lhs, rhs) => {
                let lt = self.check_expr(lhs);
                let rt = self.check_expr(rhs);
                lt.join(rt)
            }
            ExprKind::Unary(_, e) => self.check_expr(e),
            ExprKind::Call(f, args) => {
                let ft = self.check_expr(f);
                let mut t = ft;
                for a in args {
                    t = t.join(self.check_expr(a));
                }
                t
            }
            ExprKind::If(i) => {
                let cond_taint = self.check_expr(&i.cond);
                let prev_pc = self.pc_level;
                self.pc_level = self.pc_level.join(cond_taint);
                let then_taint = self.check_expr(&Box::new(ExprNode {
                    kind: ExprKind::Block(i.then_branch.clone()),
                    span: i.span,
                }));
                let else_taint = if let Some(eb) = &i.else_branch {
                    match &**eb {
                        ElseBranch::Block(b) => self.check_expr(&Box::new(ExprNode {
                            kind: ExprKind::Block(b.clone()),
                            span: i.span,
                        })),
                        ElseBranch::If(inner) => self.check_expr(&Box::new(ExprNode {
                            kind: ExprKind::If(*inner.clone()),
                            span: inner.span,
                        })),
                    }
                } else {
                    TaintLevel::Clean
                };
                self.pc_level = prev_pc;
                then_taint.join(else_taint)
            }
            ExprKind::Match(m) => {
                let scrut_taint = self.check_expr(&m.scrutinee);
                let prev_pc = self.pc_level;
                self.pc_level = self.pc_level.join(scrut_taint);
                let mut t = TaintLevel::Clean;
                for arm in &m.arms {
                    t = t.join(self.check_expr(&arm.body));
                }
                self.pc_level = prev_pc;
                t
            }
            ExprKind::Block(b) => {
                let mut t = self.pc_level;
                for stmt in &b.stmts {
                    t = t.join(self.check_stmt(stmt));
                }
                if let Some(last) = &b.last {
                    t = t.join(self.check_expr(last));
                }
                t
            }
            ExprKind::Let(l) => {
                let init_taint = self.check_expr(&l.init);
                self.var_taint.insert("".into(), init_taint);
                init_taint
            }
            ExprKind::Assignment(_, AssignOp::Eq, rhs) => {
                let rhs_taint = self.check_expr(rhs);
                let target_taint = self.pc_level;
                let source_taint = self.pc_level.join(rhs_taint);
                if !source_taint.can_flow_into(target_taint) {
                    self.errors.push(TypeError::TaintViolation {
                        message: format!(
                            "Cannot assign {:?} value to {:?} target",
                            source_taint, target_taint
                        ),
                        span: expr.span,
                    });
                }
                source_taint
            }
            ExprKind::Perform(_, args) => {
                let mut t = self.pc_level;
                for a in args {
                    t = t.join(self.check_expr(a));
                }
                t
            }
            ExprKind::Discharge(s, thresholds) => {
                let st = self.check_expr(s);
                for (_, body) in thresholds {
                    let _ = self.check_expr(&Box::new(ExprNode {
                        kind: ExprKind::Block(body.clone()),
                        span: expr.span,
                    }));
                }
                st
            }
            _ => self.pc_level,
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> TaintLevel {
        match stmt {
            Stmt::Let(l) => self.check_expr(&Box::new(ExprNode {
                kind: ExprKind::Let(l.clone()),
                span: l.span,
            })),
            Stmt::Expr(e) => self.check_expr(e),
            Stmt::Return(r) => {
                if let Some(e) = &r.expr {
                    self.check_expr(e)
                } else {
                    self.pc_level
                }
            }
            _ => self.pc_level,
        }
    }
}

impl Default for TaintChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_taint(program: Program) -> Result<Program, TypeError> {
    let mut checker = TaintChecker::new();
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
