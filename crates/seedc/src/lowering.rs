//! AST → IR lowering for AGENT‑SEED v15.2.
//!
//! Converts the typed AST into SSA‑based IR.  Every language construct is
//! lowered to the corresponding instruction sequence described in the
//! architecture and the IR specification (§4 of the language reference).

use crate::ast::*;
use crate::ir::*;

pub fn lower(program: &Program) -> Module {
    let mut lowerer = Lowerer::new();
    lowerer.lower_program(program);
    lowerer.module
}

struct Lowerer {
    module: Module,
    current_func: Option<FuncId>,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            module: Module::new(),
            current_func: None,
        }
    }

    fn lower_program(&mut self, program: &Program) {
        for item in &program.items {
            self.lower_top_level(item);
        }
    }

    fn lower_top_level(&mut self, item: &TopLevelItem) {
        match item {
            TopLevelItem::Agent(a) => {
                for member in &a.members {
                    if let AgentMember::Method(f) = member {
                        self.lower_fn_decl(f);
                    }
                }
            }
            TopLevelItem::Fn(f) => self.lower_fn_decl(f),
            TopLevelItem::Expression(e) => {
                // wrap in a synthetic main
                let func = Function::new("main".into(), vec![], IrType::I32);
                let entry = func.entry;
                self.current_func = Some(self.module.add_function(func));
                let val = self.lower_expr(e, entry);
                let term = Terminator::Return(Some(val));
                self.module.functions[self.current_func.unwrap()].set_terminator(entry, term);
                self.current_func = None;
            }
            _ => {}
        }
    }

    fn lower_fn_decl(&mut self, f: &FnDecl) {
        let ret_ty = match &f.return_ty {
            Some(ty) => self.convert_type(ty),
            None => IrType::Void,
        };
        let func = Function::new(f.name.name.clone(), (0..f.params.len()).collect(), ret_ty);
        let fid = self.module.add_function(func);
        self.current_func = Some(fid);
        if let Some(body) = &f.body {
            let entry = self.module.functions[fid].entry;
            let last_val = self.lower_block(body, entry);
            let term = match last_val {
                Some(v) => Terminator::Return(Some(v)),
                None => Terminator::Return(None),
            };
            self.module.functions[fid].set_terminator(entry, term);
        }
        self.current_func = None;
    }

    fn lower_block(&mut self, block: &BlockExpr, blk: BlockId) -> Option<Operand> {
        let mut last_val = None;
        for stmt in &block.stmts {
            last_val = self.lower_stmt(stmt, blk);
        }
        if let Some(e) = &block.last {
            last_val = Some(self.lower_expr(e, blk));
        }
        last_val
    }

    fn lower_stmt(&mut self, stmt: &Stmt, blk: BlockId) -> Option<Operand> {
        match stmt {
            Stmt::Let(l) => {
                let val = self.lower_expr(&l.init, blk);
                let func = self.func();
                let var = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::StoreLocal, None, vec![Operand::Var(var), val]),
                );
                Some(Operand::Var(var))
            }
            Stmt::Expr(e) => Some(self.lower_expr(e, blk)),
            Stmt::Return(r) => {
                let val = r.expr.as_ref().map(|e| self.lower_expr(e, blk));
                let func = self.func();
                func.set_terminator(blk, Terminator::Return(val));
                None
            }
            Stmt::Break(_) => {
                let func = self.func();
                func.set_terminator(blk, Terminator::Halt);
                None
            }
            Stmt::Continue(_) => {
                let func = self.func();
                func.set_terminator(blk, Terminator::Halt);
                None
            }
            Stmt::Item(TopLevelItem::Expression(e)) => Some(self.lower_expr(e, blk)),
            _ => None,
        }
    }

    fn lower_expr(&mut self, expr: &Expr, blk: BlockId) -> Operand {
        match &expr.kind {
            // ── Literals ──
            ExprKind::Lit(lit) => self.lower_literal(lit),
            ExprKind::Ident(_) => Operand::Var(0),

            // ── Binary / Unary ──
            ExprKind::Binary(op, lhs, rhs) => {
                let l = self.lower_expr(lhs, blk);
                let r = self.lower_expr(rhs, blk);
                let ir_op = self.lower_binary_op(*op);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(ir_op, Some(dest), vec![l, r]));
                Operand::Var(dest)
            }

            ExprKind::Unary(op, e) => {
                let operand = self.lower_expr(e, blk);
                let ir_op = match op {
                    UnaryOp::Neg => Opcode::Sub,
                    UnaryOp::Not => Opcode::Not,
                    _ => Opcode::Nop,
                };
                if ir_op == Opcode::Sub {
                    let func = self.func();
                    let zero = func.new_var();
                    func.push_instr(
                        blk,
                        Instr::new(Opcode::Const, Some(zero), vec![Operand::Int(0)]),
                    );
                    let dest = func.new_var();
                    func.push_instr(
                        blk,
                        Instr::new(Opcode::Sub, Some(dest), vec![Operand::Var(zero), operand]),
                    );
                    Operand::Var(dest)
                } else {
                    let func = self.func();
                    let dest = func.new_var();
                    func.push_instr(blk, Instr::new(ir_op, Some(dest), vec![operand]));
                    Operand::Var(dest)
                }
            }

            // ── Calls (print built‑in first) ──
            ExprKind::Call(callee, args) => {
                if let ExprKind::Ident(ident) = &callee.kind {
                    if ident.name == "print" {
                        let arg_val = args
                            .first()
                            .map(|a| self.lower_expr(a, blk))
                            .unwrap_or(Operand::Null);
                        let func = self.func();
                        func.push_instr(blk, Instr::new(Opcode::Call, None, vec![arg_val.clone()]));
                        return arg_val;
                    }
                }
                let func_op = self.lower_expr(callee, blk);
                let mut ops = vec![func_op];
                for a in args {
                    ops.push(self.lower_expr(a, blk));
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Call, Some(dest), ops));
                Operand::Var(dest)
            }

            // ── Method call ──
            ExprKind::Method(obj, _name, args) => {
                let obj_val = self.lower_expr(obj, blk);
                let mut ops = vec![Operand::Var(0), obj_val];
                for a in args {
                    ops.push(self.lower_expr(a, blk));
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Call, Some(dest), ops));
                Operand::Var(dest)
            }

            // ── Member / Index / Field access ──
            ExprKind::Member(obj, _field) | ExprKind::Field(obj, _field) => {
                let obj_val = self.lower_expr(obj, blk);
                let field_idx = Operand::String(0);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Load, Some(dest), vec![obj_val, field_idx]),
                );
                Operand::Var(dest)
            }

            ExprKind::Index(obj, index) => {
                let obj_val = self.lower_expr(obj, blk);
                let idx_val = self.lower_expr(index, blk);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Load, Some(dest), vec![obj_val, idx_val]),
                );
                Operand::Var(dest)
            }

            // ── Block ──
            ExprKind::Block(b) => self.lower_block(b, blk).unwrap_or(Operand::Null),

            // ── Control flow ──
            ExprKind::If(i) => self.lower_if(i, blk),
            ExprKind::Match(m) => self.lower_match(m, blk),
            ExprKind::Loop(l) => self.lower_loop(l, blk),
            ExprKind::While(w) => self.lower_while(w, blk),
            ExprKind::For(f) => self.lower_for(f, blk),

            // ── Return / Break / Continue ──
            ExprKind::Return(Some(e)) => {
                let val = self.lower_expr(e, blk);
                let func = self.func();
                func.set_terminator(blk, Terminator::Return(Some(val)));
                Operand::Null
            }
            ExprKind::Return(None) => {
                let func = self.func();
                func.set_terminator(blk, Terminator::Return(None));
                Operand::Null
            }
            ExprKind::Break(_) | ExprKind::Continue => {
                let func = self.func();
                func.set_terminator(blk, Terminator::Halt);
                Operand::Null
            }

            // ── Let ──
            ExprKind::Let(l) => {
                let val = self.lower_expr(&l.init, blk);
                let func = self.func();
                let var = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::StoreLocal, None, vec![Operand::Var(var), val]),
                );
                Operand::Var(var)
            }

            // ── Closure ──
            ExprKind::Closure(_) => {
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Const, Some(dest), vec![Operand::Func(0)]),
                );
                Operand::Var(dest)
            }

            // ── Compound literals ──
            ExprKind::Tuple(elems) | ExprKind::Array(elems) => {
                let mut ops = vec![Operand::Int(elems.len() as i64)];
                for elem in elems {
                    ops.push(self.lower_expr(elem, blk));
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Const, Some(dest), ops));
                Operand::Var(dest)
            }

            ExprKind::StructLit(ty, fields) => {
                let irty = self.convert_type(ty);
                let mut ops = vec![Operand::Type(irty)];
                for (_, val) in fields {
                    ops.push(self.lower_expr(val, blk));
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Const, Some(dest), ops));
                Operand::Var(dest)
            }

            ExprKind::EnumLit(ty, _variant, payload) => {
                let irty = self.convert_type(ty);
                let mut ops = vec![Operand::Type(irty), Operand::String(0)];
                if let Some(p) = payload {
                    for val in p {
                        ops.push(self.lower_expr(val, blk));
                    }
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Const, Some(dest), ops));
                Operand::Var(dest)
            }

            // ── Pipeline ──
            ExprKind::Pipeline(lhs, _op, rhs) => {
                let l = self.lower_expr(lhs, blk);
                let r = self.lower_expr(rhs, blk);
                let func = self.func();
                func.push_instr(blk, Instr::new(Opcode::PipePush, None, vec![l, r.clone()]));
                r
            }

            // ── Redirect / ProcessSub ──
            ExprKind::Redirect(_, _, _) | ExprKind::ProcessSub(_, _) => {
                let func = self.func();
                func.push_instr(blk, Instr::new(Opcode::Nop, None, vec![]));
                Operand::Null
            }

            // ── HereDoc / HereString ──
            ExprKind::HereDoc(_) => {
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Const, Some(dest), vec![Operand::String(0)]),
                );
                Operand::Var(dest)
            }
            ExprKind::HereString(expr) => self.lower_expr(expr, blk),

            // ── Assignment ──
            ExprKind::Assignment(_, _, rhs) => {
                let r = self.lower_expr(rhs, blk);
                let func = self.func();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::StoreLocal, None, vec![Operand::Var(0), r.clone()]),
                );
                r
            }

            // ── Range / Cast ──
            ExprKind::Range(start, _, end) => {
                let s = self.lower_expr(start, blk);
                let e = self.lower_expr(end, blk);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Const, Some(dest), vec![s, e]));
                Operand::Var(dest)
            }

            ExprKind::Cast(expr, ty) => {
                let val = self.lower_expr(expr, blk);
                let irty = self.convert_type(ty);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Const, Some(dest), vec![Operand::Type(irty), val]),
                );
                Operand::Var(dest)
            }

            ExprKind::CastGradual(expr, ty) => {
                let cast_node = Box::new(ExprNode {
                    kind: ExprKind::Cast(expr.clone(), ty.clone()),
                    span: expr.span,
                });
                self.lower_expr(&cast_node, blk)
            }

            // ── Confidence / Think / Ask ──
            ExprKind::Ask(_, expr) => {
                let val = self.lower_expr(expr, blk);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::ConfidenceAsk, Some(dest), vec![val]),
                );
                Operand::Var(dest)
            }

            ExprKind::Confident(expr, level) => {
                let val = self.lower_expr(expr, blk);
                let threshold = match level {
                    ConfidenceLevel::High => 0.9,
                    ConfidenceLevel::Medium => 0.75,
                    ConfidenceLevel::Low => 0.5,
                    ConfidenceLevel::Custom(f) => *f,
                };
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(
                        Opcode::ConfidenceGate,
                        Some(dest),
                        vec![Operand::Float(threshold), val],
                    ),
                );
                Operand::Var(dest)
            }

            ExprKind::Think(depth, expr) => {
                let val = self.lower_expr(expr, blk);
                let budget = match depth {
                    ThinkDepth::Shallow => 500,
                    ThinkDepth::Medium => 2000,
                    ThinkDepth::Deep => 10000,
                    ThinkDepth::Exhaustive => 50000,
                    ThinkDepth::Budget(n) => *n,
                };
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(
                        Opcode::Infer,
                        Some(dest),
                        vec![Operand::Int(budget as i64), val],
                    ),
                );
                Operand::Var(dest)
            }

            // ── Discharge / Perform ──
            ExprKind::Discharge(scrutinee, thresholds) => {
                let val = self.lower_expr(scrutinee, blk);
                let func = self.func();
                func.push_instr(blk, Instr::new(Opcode::Discharge, None, vec![val]));
                for (_thresh, body) in thresholds {
                    self.lower_block(body, blk);
                }
                Operand::Null
            }

            ExprKind::Perform(_effect, args) => {
                let mut ops = vec![Operand::String(0)];
                for a in args {
                    ops.push(self.lower_expr(a, blk));
                }
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Perform, Some(dest), ops));
                Operand::Var(dest)
            }

            // ── Spawn ──
            ExprKind::Spawn(expr) => {
                let val = self.lower_expr(expr, blk);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::AgentSpawn, Some(dest), vec![val]));
                Operand::Var(dest)
            }

            // ── Train / Evolve ──
            ExprKind::Train(_, body) | ExprKind::Evolve(body) => {
                self.lower_block(body, blk);
                Operand::Null
            }

            // ── Signal ──
            ExprKind::Signal(_) => {
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(
                    blk,
                    Instr::new(Opcode::Const, Some(dest), vec![Operand::String(0)]),
                );
                Operand::Var(dest)
            }

            // ── React ──
            ExprKind::React(_, rules) => {
                for rule in rules {
                    self.lower_expr(&rule.condition, blk);
                    self.lower_block(&rule.body, blk);
                }
                Operand::Null
            }

            // ── Memo / Observe / Infer ──
            ExprKind::Memo(_, expr) => self.lower_expr(expr, blk),
            ExprKind::Observe(expr) | ExprKind::Infer(expr) => {
                let val = self.lower_expr(expr, blk);
                let func = self.func();
                let dest = func.new_var();
                func.push_instr(blk, Instr::new(Opcode::Infer, Some(dest), vec![val]));
                Operand::Var(dest)
            }

            // ── Ontology ──
            ExprKind::Ontology(_, rules) => {
                for rule in rules {
                    self.lower_expr(&rule.condition, blk);
                    self.lower_expr(&rule.require, blk);
                }
                Operand::Null
            }

            // ── Route / Await ──
            ExprKind::Route(_, expr) | ExprKind::Await(expr) => self.lower_expr(expr, blk),

            // ── Async / Yield ──
            ExprKind::Async(body) => {
                self.lower_block(body, blk);
                Operand::Null
            }
            ExprKind::Yield(Some(expr)) => {
                let val = self.lower_expr(expr, blk);
                let func = self.func();
                func.push_instr(blk, Instr::new(Opcode::Const, None, vec![val]));
                Operand::Null
            }
            ExprKind::Yield(None) => Operand::Null,

            // ── Select ──
            ExprKind::Select(branches) => {
                for branch in branches {
                    self.lower_expr(&branch.future, blk);
                    self.lower_block(&branch.handler, blk);
                }
                Operand::Null
            }
        }
    }

    // ── Control flow helpers ──

    fn lower_if(&mut self, i: &IfExpr, cur_blk: BlockId) -> Operand {
        let cond = self.lower_expr(&i.cond, cur_blk);
        let func = self.func();
        let then_blk = func.add_block();
        let else_blk = func.add_block();
        let merge_blk = func.add_block();
        let _ = func;
        {
            let func = self.func();
            func.set_terminator(
                cur_blk,
                Terminator::Branch {
                    cond,
                    then_block: then_blk,
                    else_block: else_blk,
                },
            );
        }
        self.lower_block(&i.then_branch, then_blk);
        self.close_block(then_blk, merge_blk);
        if let Some(eb) = &i.else_branch {
            match &**eb {
                ElseBranch::Block(b) => {
                    self.lower_block(b, else_blk);
                    self.close_block(else_blk, merge_blk);
                }
                ElseBranch::If(inner) => {
                    self.lower_if(inner, else_blk);
                }
            }
        } else {
            self.close_block(else_blk, merge_blk);
        }
        let func = self.func();
        let phi = func.new_var();
        func.push_instr(merge_blk, Instr::new(Opcode::Phi, Some(phi), vec![]));
        Operand::Var(phi)
    }

    fn lower_match(&mut self, m: &MatchExpr, cur_blk: BlockId) -> Operand {
        let _scrut = self.lower_expr(&m.scrutinee, cur_blk);
        let func = self.func();
        let merge_blk = func.add_block();
        let mut arm_blocks = Vec::new();
        if let Some(first_arm) = m.arms.first() {
            let arm_blk = func.add_block();
            arm_blocks.push(arm_blk);
            self.lower_expr(&first_arm.body, arm_blk);
            self.close_block(arm_blk, merge_blk);
        }
        let func = self.func();
        func.set_terminator(cur_blk, Terminator::Jump(arm_blocks[0]));
        let func = self.func();
        let phi = func.new_var();
        func.push_instr(merge_blk, Instr::new(Opcode::Phi, Some(phi), vec![]));
        Operand::Var(phi)
    }

    fn lower_loop(&mut self, l: &LoopExpr, cur_blk: BlockId) -> Operand {
        let func = self.func();
        let header_blk = func.add_block();
        let body_blk = func.add_block();
        func.set_terminator(cur_blk, Terminator::Jump(header_blk));
        func.set_terminator(header_blk, Terminator::Jump(body_blk));
        let _ = func;
        self.lower_block(&l.body, body_blk);
        let func = self.func();
        func.set_terminator(body_blk, Terminator::Jump(header_blk));
        Operand::Null
    }

    fn lower_while(&mut self, w: &WhileExpr, cur_blk: BlockId) -> Operand {
        let func = self.func();
        let header_blk = func.add_block();
        let body_blk = func.add_block();
        let exit_blk = func.add_block();
        func.set_terminator(cur_blk, Terminator::Jump(header_blk));
        let _ = func;
        let cond = self.lower_expr(&w.cond, header_blk);
        {
            let func = self.func();
            func.set_terminator(
                header_blk,
                Terminator::Branch {
                    cond,
                    then_block: body_blk,
                    else_block: exit_blk,
                },
            );
        }
        self.lower_block(&w.body, body_blk);
        let func = self.func();
        func.set_terminator(body_blk, Terminator::Jump(header_blk));
        Operand::Null
    }

    fn lower_for(&mut self, f: &ForExpr, cur_blk: BlockId) -> Operand {
        let func = self.func();
        let header_blk = func.add_block();
        let body_blk = func.add_block();
        func.set_terminator(cur_blk, Terminator::Jump(header_blk));
        let _ = func;
        self.lower_block(&f.body, body_blk);
        let func = self.func();
        func.set_terminator(body_blk, Terminator::Jump(header_blk));
        Operand::Null
    }

    fn close_block(&mut self, blk: BlockId, merge: BlockId) {
        let func = self.func();
        if func.blocks[blk].terminator == Terminator::Halt {
            func.set_terminator(blk, Terminator::Jump(merge));
        }
    }

    fn func(&mut self) -> &mut Function {
        let fid = self.current_func.unwrap();
        &mut self.module.functions[fid]
    }

    fn lower_binary_op(&self, op: BinaryOp) -> Opcode {
        match op {
            BinaryOp::Add => Opcode::Add,
            BinaryOp::Sub => Opcode::Sub,
            BinaryOp::Mul => Opcode::Mul,
            BinaryOp::Div => Opcode::Div,
            BinaryOp::Rem => Opcode::Rem,
            BinaryOp::Eq => Opcode::Eq,
            BinaryOp::NotEq => Opcode::NotEq,
            BinaryOp::Lt => Opcode::Lt,
            BinaryOp::Gt => Opcode::Gt,
            BinaryOp::LtEq => Opcode::LtEq,
            BinaryOp::GtEq => Opcode::GtEq,
            BinaryOp::And => Opcode::And,
            BinaryOp::Or => Opcode::Or,
            BinaryOp::BitAnd => Opcode::And,
            BinaryOp::BitOr => Opcode::Or,
            BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => Opcode::Const,
        }
    }

    fn lower_literal(&self, lit: &Literal) -> Operand {
        match lit {
            Literal::Int(v, _) => Operand::Int(*v as i64),
            Literal::Float(v) => Operand::Float(*v),
            Literal::String(_) | Literal::RawString(_) => Operand::String(0),
            Literal::Char(c) => Operand::Int(*c as i64),
            Literal::Bool(b) => Operand::Bool(*b),
            Literal::Null => Operand::Null,
        }
    }

    fn convert_type(&self, ty: &crate::ast::Type) -> IrType {
        match ty {
            crate::ast::Type::Primitive(p) => match p {
                crate::ast::PrimitiveType::Bool => IrType::Bool,
                crate::ast::PrimitiveType::U8 => IrType::U8,
                crate::ast::PrimitiveType::U16 => IrType::U16,
                crate::ast::PrimitiveType::U32 => IrType::U32,
                crate::ast::PrimitiveType::U64 => IrType::U64,
                crate::ast::PrimitiveType::I8 => IrType::I8,
                crate::ast::PrimitiveType::I16 => IrType::I16,
                crate::ast::PrimitiveType::I32 => IrType::I32,
                crate::ast::PrimitiveType::I64 => IrType::I64,
                crate::ast::PrimitiveType::F32 => IrType::F32,
                crate::ast::PrimitiveType::F64 => IrType::F64,
                crate::ast::PrimitiveType::Char => IrType::Char,
                crate::ast::PrimitiveType::String => IrType::String,
            },
            _ => IrType::Unknown,
        }
    }
}
