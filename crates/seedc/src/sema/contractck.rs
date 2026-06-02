//! Contract verification for AGENT‑SEED v15.2.

use crate::ast::*;
use crate::sema::TypeError;

pub struct ContractChecker {
    errors: Vec<TypeError>,
}

impl ContractChecker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn check_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Discharge(scrutinee, thresholds) => {
                if thresholds.is_empty() {
                    self.errors.push(TypeError::ContractViolation {
                        message: "Discharge block must have at least one threshold arm.".into(),
                        span: expr.span,
                    });
                }
                self.check_expr(scrutinee);
                for (_, body) in thresholds {
                    self.check_expr(&Box::new(ExprNode {
                        kind: ExprKind::Block(body.clone()),
                        span: expr.span,
                    }));
                }
            }
            ExprKind::Call(f, args) => {
                self.check_expr(f);
                for a in args {
                    self.check_expr(a);
                }
            }
            ExprKind::Binary(_, l, r) => {
                self.check_expr(l);
                self.check_expr(r);
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
                        ElseBranch::Block(b) => self.check_expr(&Box::new(ExprNode {
                            kind: ExprKind::Block(b.clone()),
                            span: i.span,
                        })),
                        ElseBranch::If(inner) => self.check_expr(&Box::new(ExprNode {
                            kind: ExprKind::If(*inner.clone()),
                            span: inner.span,
                        })),
                    }
                }
            }
            ExprKind::Block(b) => {
                for stmt in &b.stmts {
                    self.check_stmt(stmt);
                }
                if let Some(last) = &b.last {
                    self.check_expr(last);
                }
            }
            ExprKind::Let(l) => {
                self.check_expr(&l.init);
            }
            _ => {}
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
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
            }
            _ => {}
        }
    }
}

impl Default for ContractChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_contracts(program: Program) -> Result<Program, TypeError> {
    let mut checker = ContractChecker::new();
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
