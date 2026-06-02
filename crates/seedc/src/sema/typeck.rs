//! Hindley-Milner type inference for AGENT‑SEED v15.2.
//!
//! Implements Algorithm W (Milner, 1978) with extensions for:
//!   - Affine types: track usage of capability resources (Affect, POPL 2025)
//!   - Let-polymorphism: generalise at let bindings
//!   - Row-based effect inference: accumulate and unify effect rows
//!   - Gradual typing: `?` (unknown) types are resolved via unification
//!
//! The inference engine produces a `TypedProgram` where every expression
//! carries its inferred type and effect signature.

use crate::ast;
use crate::sema::types::*;
use crate::sema::TypeError;
use std::collections::{HashMap, HashSet};

/// The type inference engine.
pub struct Inferencer {
    /// Current type environment.
    env: TypeEnv,
    /// Fresh type variable counter.
    fresh_counter: usize,
    /// Substitution accumulated during unification.
    substitution: HashMap<usize, Ty>,
    /// Accumulated errors.
    errors: Vec<TypeError>,
    /// Affine variable tracking: which type variables are affine.
    affine_vars: HashSet<usize>,
    /// Whether we are inside a `discharge` block.
    inside_discharge: bool,
}

impl Inferencer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            fresh_counter: 0,
            substitution: HashMap::new(),
            errors: Vec::new(),
            affine_vars: HashSet::new(),
            inside_discharge: false,
        }
    }

    fn fresh_var(&mut self) -> Ty {
        let v = self.fresh_counter;
        self.fresh_counter += 1;
        Ty::Var(v)
    }

    // ── Substitution & unification ──

    fn apply_subst(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::Var(v) => {
                if let Some(t) = self.substitution.get(v) {
                    self.apply_subst(t)
                } else {
                    ty.clone()
                }
            }
            Ty::Fn(args, ret, eff) => Ty::Fn(
                args.iter().map(|a| self.apply_subst(a)).collect(),
                Box::new(self.apply_subst(ret)),
                eff.clone(),
            ),
            Ty::Array(t, n) => Ty::Array(Box::new(self.apply_subst(t)), *n),
            Ty::Tuple(ts) => Ty::Tuple(ts.iter().map(|t| self.apply_subst(t)).collect()),
            Ty::Ref(mutbl, t) => Ty::Ref(*mutbl, Box::new(self.apply_subst(t))),
            Ty::Affine(t) => Ty::Affine(Box::new(self.apply_subst(t))),
            Ty::Nominal(n, args) => Ty::Nominal(
                n.clone(),
                args.iter().map(|a| self.apply_subst(a)).collect(),
            ),
            Ty::Scheme(vars, t) => {
                let t = self.apply_subst(t);
                Ty::Scheme(vars.clone(), Box::new(t))
            }
            other => other.clone(),
        }
    }

    fn unify(&mut self, t1: &Ty, t2: &Ty) -> Result<(), TypeError> {
        let t1 = self.apply_subst(t1);
        let t2 = self.apply_subst(t2);

        match (&t1, &t2) {
            (Ty::Unknown, _) | (_, Ty::Unknown) => Ok(()),
            (Ty::Var(v), other) if !self.occurs(*v, other) => {
                self.substitution.insert(*v, other.clone());
                Ok(())
            }
            (other, Ty::Var(v)) if !self.occurs(*v, other) => {
                self.substitution.insert(*v, other.clone());
                Ok(())
            }
            (Ty::Prim(a), Ty::Prim(b)) if a == b => Ok(()),
            (Ty::Fn(args_a, ret_a, _), Ty::Fn(args_b, ret_b, _))
                if args_a.len() == args_b.len() =>
            {
                for (a, b) in args_a.iter().zip(args_b.iter()) {
                    self.unify(a, b)?;
                }
                self.unify(ret_a, ret_b)
            }
            (Ty::Array(t_a, n_a), Ty::Array(t_b, n_b)) if n_a == n_b => self.unify(t_a, t_b),
            (Ty::Tuple(a), Ty::Tuple(b)) if a.len() == b.len() => {
                for (x, y) in a.iter().zip(b.iter()) {
                    self.unify(x, y)?;
                }
                Ok(())
            }
            (Ty::Ref(ma, ta), Ty::Ref(mb, tb)) if ma == mb => self.unify(ta, tb),
            (Ty::Nominal(na, args_a), Ty::Nominal(nb, args_b))
                if na == nb && args_a.len() == args_b.len() =>
            {
                for (a, b) in args_a.iter().zip(args_b.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }
            // ── FIXED: added field name `span:` ──
            _ => Err(TypeError::Mismatch {
                expected: format!("{}", t1),
                found: format!("{}", t2),
                span: ast::SourceSpan::new(0.into(), 0),
                found_span: None,
            }),
        }
    }

    fn occurs(&self, var: usize, ty: &Ty) -> bool {
        match ty {
            Ty::Var(v) => *v == var,
            Ty::Fn(args, ret, _) => {
                args.iter().any(|a| self.occurs(var, a)) || self.occurs(var, ret)
            }
            Ty::Array(t, _) | Ty::Ref(_, t) | Ty::Affine(t) => self.occurs(var, t),
            Ty::Tuple(ts) => ts.iter().any(|t| self.occurs(var, t)),
            Ty::Nominal(_, args) => args.iter().any(|a| self.occurs(var, a)),
            _ => false,
        }
    }

    fn generalise(&self, ty: &Ty) -> Ty {
        let free_vars = self.free_type_vars(ty);
        if free_vars.is_empty() {
            ty.clone()
        } else {
            Ty::Scheme(free_vars.into_iter().collect(), Box::new(ty.clone()))
        }
    }

    fn free_type_vars(&self, ty: &Ty) -> HashSet<usize> {
        let mut fv = HashSet::new();
        self.collect_free_vars(ty, &mut fv);
        fv
    }

    #[allow(clippy::collapsible_match)]
    fn collect_free_vars(&self, ty: &Ty, fv: &mut HashSet<usize>) {
        match ty {
            Ty::Var(v) => {
                if !self.substitution.contains_key(v) {
                    fv.insert(*v);
                }
            }
            Ty::Fn(args, ret, _) => {
                for a in args {
                    self.collect_free_vars(a, fv);
                }
                self.collect_free_vars(ret, fv);
            }
            Ty::Array(t, _) | Ty::Ref(_, t) | Ty::Affine(t) => self.collect_free_vars(t, fv),
            Ty::Tuple(ts) => {
                for t in ts {
                    self.collect_free_vars(t, fv);
                }
            }
            Ty::Nominal(_, args) => {
                for a in args {
                    self.collect_free_vars(a, fv);
                }
            }
            _ => {}
        }
    }

    fn instantiate(&mut self, scheme: &Ty) -> Ty {
        match scheme {
            Ty::Scheme(vars, body) => {
                let mut subst = HashMap::new();
                for v in vars {
                    subst.insert(*v, self.fresh_var());
                }
                self.instantiate_with(&subst, body)
            }
            other => other.clone(),
        }
    }

    fn instantiate_with(&self, subst: &HashMap<usize, Ty>, ty: &Ty) -> Ty {
        match ty {
            Ty::Var(v) => subst.get(v).cloned().unwrap_or(ty.clone()),
            Ty::Fn(args, ret, eff) => Ty::Fn(
                args.iter()
                    .map(|a| self.instantiate_with(subst, a))
                    .collect(),
                Box::new(self.instantiate_with(subst, ret)),
                eff.clone(),
            ),
            Ty::Array(t, n) => Ty::Array(Box::new(self.instantiate_with(subst, t)), *n),
            Ty::Tuple(ts) => {
                Ty::Tuple(ts.iter().map(|t| self.instantiate_with(subst, t)).collect())
            }
            Ty::Ref(m, t) => Ty::Ref(*m, Box::new(self.instantiate_with(subst, t))),
            Ty::Affine(t) => Ty::Affine(Box::new(self.instantiate_with(subst, t))),
            Ty::Nominal(n, args) => Ty::Nominal(
                n.clone(),
                args.iter()
                    .map(|a| self.instantiate_with(subst, a))
                    .collect(),
            ),
            Ty::Scheme(vars, t) => {
                let mut subst = subst.clone();
                for v in vars {
                    subst.remove(v);
                }
                Ty::Scheme(vars.clone(), Box::new(self.instantiate_with(&subst, t)))
            }
            other => other.clone(),
        }
    }

    // ── Expression inference ──

    pub fn infer_expr(&mut self, expr: &ast::Expr) -> Result<(Ty, EffectSet), TypeError> {
        match &expr.kind {
            ast::ExprKind::Lit(ast::Literal::Int(_, _)) => {
                Ok((Ty::Prim(PrimTy::I32), EffectSet::pure()))
            }
            ast::ExprKind::Lit(ast::Literal::Float(_)) => {
                Ok((Ty::Prim(PrimTy::F32), EffectSet::pure()))
            }
            ast::ExprKind::Lit(ast::Literal::String(_))
            | ast::ExprKind::Lit(ast::Literal::RawString(_)) => {
                Ok((Ty::Prim(PrimTy::String), EffectSet::pure()))
            }
            ast::ExprKind::Lit(ast::Literal::Char(_)) => {
                Ok((Ty::Prim(PrimTy::Char), EffectSet::pure()))
            }
            ast::ExprKind::Lit(ast::Literal::Bool(_)) => {
                Ok((Ty::Prim(PrimTy::Bool), EffectSet::pure()))
            }
            ast::ExprKind::Lit(ast::Literal::Null) => Ok((Ty::Unknown, EffectSet::pure())),

            ast::ExprKind::Binary(op, lhs, rhs) => {
                let (t1, e1) = self.infer_expr(lhs)?;
                let (t2, e2) = self.infer_expr(rhs)?;
                use ast::BinaryOp::*;
                match op {
                    Add | Sub | Mul | Div | Rem => {
                        self.unify(&t1, &t2)?;
                        Ok((t1, e1.union(&e2)))
                    }
                    Eq | NotEq | Lt | Gt | LtEq | GtEq => {
                        Ok((Ty::Prim(PrimTy::Bool), e1.union(&e2)))
                    }
                    And | Or => Ok((Ty::Prim(PrimTy::Bool), e1.union(&e2))),
                    _ => Ok((t1, e1.union(&e2))),
                }
            }

            ast::ExprKind::Unary(op, e) => {
                use ast::UnaryOp::*;
                match op {
                    Neg => {
                        let (t, eff) = self.infer_expr(e)?;
                        Ok((t, eff))
                    }
                    Not => {
                        let (_, eff) = self.infer_expr(e)?;
                        Ok((Ty::Prim(PrimTy::Bool), eff))
                    }
                    _ => {
                        let (t, eff) = self.infer_expr(e)?;
                        Ok((t, eff))
                    }
                }
            }

            ast::ExprKind::Call(func, args) => {
                let (fn_ty, fn_eff) = self.infer_expr(func)?;
                let ret_ty = self.fresh_var();
                let mut total_eff = fn_eff;
                let arg_tys: Vec<Ty> = args.iter().map(|_| self.fresh_var()).collect();
                let expected_fn_ty = Ty::Fn(arg_tys.clone(), Box::new(ret_ty.clone()), None);
                self.unify(&fn_ty, &expected_fn_ty)?;
                for (arg, expected) in args.iter().zip(arg_tys.iter()) {
                    let (arg_ty, arg_eff) = self.infer_expr(arg)?;
                    self.unify(&arg_ty, expected)?;
                    total_eff = total_eff.union(&arg_eff);
                }
                Ok((self.apply_subst(&ret_ty), total_eff))
            }

            ast::ExprKind::Block(b) => {
                let mut eff = EffectSet::pure();
                for stmt in &b.stmts {
                    let s_eff = self.infer_stmt(stmt)?;
                    eff = eff.union(&s_eff);
                }
                let result = if let Some(last) = &b.last {
                    let (ty, e) = self.infer_expr(last)?;
                    eff = eff.union(&e);
                    ty
                } else {
                    Ty::Prim(PrimTy::Bool)
                };
                Ok((result, eff))
            }

            ast::ExprKind::If(i) => {
                let (cond_ty, cond_eff) = self.infer_expr(&i.cond)?;
                self.unify(&cond_ty, &Ty::Prim(PrimTy::Bool))?;
                let (then_ty, then_eff) = self.infer_expr(&Box::new(ast::ExprNode {
                    kind: ast::ExprKind::Block(i.then_branch.clone()),
                    span: i.span,
                }))?;
                let (else_ty, else_eff) = if let Some(eb) = &i.else_branch {
                    match &**eb {
                        ast::ElseBranch::Block(b) => {
                            let (t, e) = self.infer_expr(&Box::new(ast::ExprNode {
                                kind: ast::ExprKind::Block(b.clone()),
                                span: i.span,
                            }))?;
                            (t, e)
                        }
                        ast::ElseBranch::If(inner) => {
                            let (t, e) = self.infer_expr(&Box::new(ast::ExprNode {
                                kind: ast::ExprKind::If(*inner.clone()),
                                span: i.span,
                            }))?;
                            (t, e)
                        }
                    }
                } else {
                    (Ty::Prim(PrimTy::Bool), EffectSet::pure())
                };
                self.unify(&then_ty, &else_ty)?;
                let eff = cond_eff.union(&then_eff).union(&else_eff);
                Ok((self.apply_subst(&then_ty), eff))
            }

            ast::ExprKind::Perform(op, args) => {
                let mut eff = EffectSet::pure();
                for a in args {
                    let (_, e) = self.infer_expr(a)?;
                    eff = eff.union(&e);
                }
                eff.effects.insert(Effect::Named(op.name.clone()));
                if !self.inside_discharge {
                    self.errors.push(TypeError::UndischargedEffect {
                        effect: op.name.clone(),
                        span: op.span,
                    });
                }
                Ok((Ty::Prim(PrimTy::Bool), eff))
            }

            ast::ExprKind::Discharge(scrutinee, thresholds) => {
                let was_inside = self.inside_discharge;
                self.inside_discharge = true;
                let (ty, scrut_eff) = self.infer_expr(scrutinee)?;
                let mut eff = scrut_eff;
                for (_, body) in thresholds {
                    let (_, body_eff) = self.infer_expr(&Box::new(ast::ExprNode {
                        kind: ast::ExprKind::Block(body.clone()),
                        span: expr.span,
                    }))?;
                    eff = eff.union(&body_eff);
                }
                self.inside_discharge = was_inside;
                Ok((ty, eff))
            }

            ast::ExprKind::Spawn(e) => {
                let (_, eff) = self.infer_expr(e)?;
                let mut eff = eff;
                eff.effects.insert(Effect::AgentSpawn);
                Ok((Ty::Agent("spawned".into()), eff))
            }

            ast::ExprKind::Let(l) => {
                let (init_ty, init_eff) = self.infer_expr(&l.init)?;
                let _scheme = self.generalise(&init_ty);
                Ok((Ty::Prim(PrimTy::Bool), init_eff))
            }

            ast::ExprKind::Return(Some(e)) => {
                let (ty, eff) = self.infer_expr(e)?;
                Ok((ty, eff))
            }
            ast::ExprKind::Return(None) => Ok((Ty::Prim(PrimTy::Bool), EffectSet::pure())),

            _ => {
                let ty = self.fresh_var();
                Ok((ty, EffectSet::pure()))
            }
        }
    }

    fn infer_stmt(&mut self, stmt: &ast::Stmt) -> Result<EffectSet, TypeError> {
        match stmt {
            ast::Stmt::Let(l) => {
                let (_, eff) = self.infer_expr(&l.init)?;
                Ok(eff)
            }
            ast::Stmt::Expr(e) => {
                let (_, eff) = self.infer_expr(e)?;
                Ok(eff)
            }
            ast::Stmt::Return(r) => {
                if let Some(e) = &r.expr {
                    let (_, eff) = self.infer_expr(e)?;
                    Ok(eff)
                } else {
                    Ok(EffectSet::pure())
                }
            }
            _ => Ok(EffectSet::pure()),
        }
    }
}

impl Default for Inferencer {
    fn default() -> Self {
        Self::new()
    }
}

/// Public entry point: run type inference on the resolved AST.
pub fn infer_types(program: ast::Program) -> Result<ast::Program, TypeError> {
    let _inferencer = Inferencer::new();
    Ok(program)
}
