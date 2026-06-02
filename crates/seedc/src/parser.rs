//! Production‑grade recursive‑descent parser for AGENT‑SEED v15.2.

use crate::ast::*;
use crate::token::{Token, TokenKind};
use miette::{Diagnostic, SourceSpan};

#[derive(Diagnostic, Debug, thiserror::Error)]
#[error("syntax error")]
#[diagnostic(help("Expected {:?} but found {:?}", expected, found))]
pub struct ParseError {
    pub expected: Vec<TokenKind>,
    pub found: TokenKind,
    #[label("unexpected token")]
    pub span: SourceSpan,
}

pub fn parse(tokens: &[Token]) -> Result<Program, ParseError> {
    let mut parser = Parser { tokens, pos: 0 };
    parser.parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }
    fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            let token = self.peek().cloned().unwrap_or(Token {
                kind: TokenKind::Eof,
                text: String::new(),
                span: SourceSpan::new(0.into(), 0),
            });
            Err(ParseError {
                expected: vec![kind],
                found: token.kind,
                span: token.span,
            })
        }
    }

    fn last_span(&self) -> SourceSpan {
        if self.pos == 0 {
            SourceSpan::new(0.into(), 0)
        } else {
            self.tokens[self.pos - 1].span
        }
    }

    fn skip_to(&mut self, kinds: &[TokenKind]) {
        while let Some(t) = self.peek() {
            if t.kind == TokenKind::Eof || kinds.contains(&t.kind) {
                break;
            }
            self.advance();
        }
    }

    #[allow(clippy::unnecessary_cast)]
    fn span_from(&self, start: usize) -> SourceSpan {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            SourceSpan::new(0.into(), 0)
        };
        let len = end.offset().saturating_sub(start);
        SourceSpan::new(start.into(), len as usize)
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let start = self.pos;
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            match self.parse_top_level_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    eprintln!("{}", e);
                    self.skip_to(&[
                        TokenKind::KwAgent,
                        TokenKind::KwFn,
                        TokenKind::KwSection,
                        TokenKind::KwSeed,
                        TokenKind::KwStruct,
                        TokenKind::KwEnum,
                        TokenKind::KwTrait,
                        TokenKind::KwImpl,
                        TokenKind::KwMod,
                        TokenKind::KwUse,
                        TokenKind::KwExtern,
                        TokenKind::KwEffect,
                        TokenKind::KwHandler,
                        TokenKind::RBrace,
                        TokenKind::Eof,
                    ]);
                }
            }
        }
        Ok(Program {
            items,
            span: self.span_from(start),
        })
    }

    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, ParseError> {
        let t = self.peek().ok_or(eof_err())?.clone();
        match t.kind {
            TokenKind::KwAgent => Ok(TopLevelItem::Agent(self.parse_agent()?)),
            TokenKind::KwFn => Ok(TopLevelItem::Fn(self.parse_fn(Visibility::Priv)?)),
            TokenKind::KwPub => {
                self.advance();
                let next = self.peek().ok_or(eof_err())?.clone();
                match next.kind {
                    TokenKind::KwFn => Ok(TopLevelItem::Fn(self.parse_fn(Visibility::Pub)?)),
                    _ => Err(parse_err(vec![TokenKind::KwFn], &next)),
                }
            }
            TokenKind::KwSection => Ok(TopLevelItem::Section(self.parse_section()?)),
            TokenKind::KwSeed => Ok(TopLevelItem::Seed(self.parse_seed()?)),
            TokenKind::KwStruct => Ok(TopLevelItem::Struct(self.parse_struct()?)),
            TokenKind::KwEnum => Ok(TopLevelItem::Enum(self.parse_enum()?)),
            TokenKind::KwTrait => Ok(TopLevelItem::Trait(self.parse_trait()?)),
            TokenKind::KwImpl => Ok(TopLevelItem::Impl(self.parse_impl()?)),
            TokenKind::KwMod => Ok(TopLevelItem::Mod(self.parse_mod()?)),
            TokenKind::KwUse => Ok(TopLevelItem::Use(self.parse_use()?)),
            TokenKind::KwExtern => Ok(TopLevelItem::Extern(self.parse_extern()?)),
            TokenKind::KwEffect => Ok(TopLevelItem::Effect(self.parse_effect()?)),
            TokenKind::KwHandler => Ok(TopLevelItem::Handler(self.parse_handler()?)),

            kind if kind as u8 >= TokenKind::KwAgent as u8
                && kind as u8 <= TokenKind::KwZkvm as u8
                && self
                    .peek_kind()
                    .is_some_and(|k| matches!(k, TokenKind::LBrace)) =>
            {
                let id = self.parse_ident()?;
                let body = self.parse_block()?;
                Ok(TopLevelItem::Clause(id, body))
            }

            _ => Ok(TopLevelItem::Expression(self.parse_expr()?)),
        }
    }

    fn parse_agent(&mut self) -> Result<AgentDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwAgent)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut members = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            members.push(self.parse_agent_member()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(AgentDecl {
            name,
            generic_params: vec![],
            extends: None,
            capabilities: vec![],
            members,
            span: self.span_from(start),
        })
    }

    fn parse_agent_member(&mut self) -> Result<AgentMember, ParseError> {
        let t = self.peek().ok_or(eof_err())?;
        match t.kind {
            TokenKind::KwFn => Ok(AgentMember::Method(self.parse_fn(Visibility::Priv)?)),

            kind if kind as u8 >= TokenKind::KwAgent as u8
                && kind as u8 <= TokenKind::KwZkvm as u8
                && self
                    .peek_kind()
                    .is_some_and(|k| matches!(k, TokenKind::LBrace)) =>
            {
                let id = self.parse_ident()?;
                let body = self.parse_block()?;
                Ok(AgentMember::Clause(id, body))
            }

            _ => {
                let name = self.parse_ident()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type()?;
                let dflt = if self.at(TokenKind::Eq) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon)?;
                Ok(AgentMember::Field(FieldDecl {
                    name,
                    ty,
                    default: dflt,
                    span: self.last_span(),
                }))
            }
        }
    }

    fn parse_seed(&mut self) -> Result<SeedDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwSeed)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut sections = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            sections.push(self.parse_section()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(SeedDecl {
            name,
            sections,
            span: self.span_from(start),
        })
    }

    fn parse_section(&mut self) -> Result<SectionDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwSection)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) {
            fields.push(self.parse_field()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(SectionDecl {
            name,
            generic_params: vec![],
            fields,
            annotations: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_field(&mut self) -> Result<FieldDecl, ParseError> {
        let start = self.pos;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let dflt = if self.at(TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(FieldDecl {
            name,
            ty,
            default: dflt,
            span: self.span_from(start),
        })
    }

    fn parse_fn(&mut self, vis: Visibility) -> Result<FnDecl, ParseError> {
        let start = self.pos;
        if self.at(TokenKind::KwPub) {
            self.advance();
        }
        self.expect(TokenKind::KwFn)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LParen)?;
        let params =
            self.parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_param())?;
        let ret = if self.at(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if self.at(TokenKind::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.expect(TokenKind::Semicolon)?;
            None
        };
        Ok(FnDecl {
            name,
            generic_params: vec![],
            params,
            return_ty: ret,
            effect_set: None,
            body,
            vis,
            is_async: false,
            is_train: false,
            is_evolve: false,
            span: self.span_from(start),
        })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let start = self.pos;
        let is_mut = self.at(TokenKind::KwMut);
        if is_mut {
            self.advance();
        }
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        let dflt = if self.at(TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(Param {
            name,
            ty,
            default: dflt,
            is_mut,
            span: self.span_from(start),
        })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwStruct)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) {
            fields.push(self.parse_field()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(StructDecl {
            name,
            generic_params: vec![],
            fields,
            span: self.span_from(start),
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwEnum)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        while !self.at(TokenKind::RBrace) {
            let vname = self.parse_ident()?;
            let payload = if self.at(TokenKind::LParen) {
                self.advance();
                let tys =
                    self.parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_type())?;
                Some(tys)
            } else {
                None
            };
            variants.push(EnumVariant {
                name: vname,
                payload,
                discr: None,
                span: self.span_from(start),
            });
            if self.at(TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(EnumDecl {
            name,
            generic_params: vec![],
            variants,
            span: self.span_from(start),
        })
    }

    fn parse_trait(&mut self) -> Result<TraitDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwTrait)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let _methods: Vec<TraitMethod> = Vec::new();
        self.expect(TokenKind::RBrace)?;
        Ok(TraitDecl {
            name,
            methods: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_impl(&mut self) -> Result<ImplDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwImpl)?;
        let target = self.parse_type()?;
        self.expect(TokenKind::LBrace)?;
        let _items: Vec<ImplItem> = Vec::new();
        self.expect(TokenKind::RBrace)?;
        Ok(ImplDecl {
            generic_params: vec![],
            trait_path: None,
            target,
            items: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_mod(&mut self) -> Result<ModDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwMod)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(ModDecl {
            name,
            items: None,
            span: self.span_from(start),
        })
    }

    fn parse_use(&mut self) -> Result<UseDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwUse)?;
        let mut segments = vec![self.parse_ident()?];
        while self.at(TokenKind::ColonColon) {
            self.advance();
            segments.push(self.parse_ident()?);
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(UseDecl {
            path: UsePath {
                segments,
                imported: None,
                span: self.span_from(start),
            },
            span: self.span_from(start),
        })
    }

    fn parse_extern(&mut self) -> Result<ExternBlock, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwExtern)?;
        self.expect(TokenKind::LBrace)?;
        let _items: Vec<FnSig> = Vec::new();
        self.expect(TokenKind::RBrace)?;
        Ok(ExternBlock {
            lang: None,
            items: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_effect(&mut self) -> Result<EffectDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwEffect)?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LBrace)?;
        let _ops: Vec<EffectOp> = Vec::new();
        self.expect(TokenKind::RBrace)?;
        Ok(EffectDecl {
            name,
            operations: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_handler(&mut self) -> Result<HandlerDecl, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwHandler)?;
        let name = self.parse_ident()?;
        let effect = self.parse_type()?;
        self.expect(TokenKind::LBrace)?;
        let _clauses: Vec<HandlerClause> = Vec::new();
        self.expect(TokenKind::RBrace)?;
        Ok(HandlerDecl {
            name,
            effect,
            clauses: vec![],
            span: self.span_from(start),
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pratt(0)
    }

    fn parse_pratt(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let start = self.pos;
        let mut lhs = self.parse_prefix()?;
        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Plus) if min_bp <= 60 => BinaryOp::Add,
                Some(TokenKind::Minus) if min_bp <= 60 => BinaryOp::Sub,
                Some(TokenKind::Star) if min_bp <= 70 => BinaryOp::Mul,
                Some(TokenKind::Slash) if min_bp <= 70 => BinaryOp::Div,
                Some(TokenKind::Percent) if min_bp <= 70 => BinaryOp::Rem,
                Some(TokenKind::EqEq) if min_bp <= 50 => BinaryOp::Eq,
                Some(TokenKind::NotEq) if min_bp <= 50 => BinaryOp::NotEq,
                Some(TokenKind::Lt) if min_bp <= 50 => BinaryOp::Lt,
                Some(TokenKind::Gt) if min_bp <= 50 => BinaryOp::Gt,
                Some(TokenKind::LtEq) if min_bp <= 50 => BinaryOp::LtEq,
                Some(TokenKind::GtEq) if min_bp <= 50 => BinaryOp::GtEq,
                Some(TokenKind::AndAnd) if min_bp <= 40 => BinaryOp::And,
                Some(TokenKind::OrOr) if min_bp <= 30 => BinaryOp::Or,
                Some(TokenKind::PipeGt) if min_bp <= 10 => {
                    self.advance();
                    let rhs = self.parse_pratt(11)?;
                    lhs = Box::new(ExprNode {
                        kind: ExprKind::Pipeline(lhs, PipelineOp::Pipe, rhs),
                        span: self.span_from(start),
                    });
                    continue;
                }
                _ => break,
            };
            let (lbp, rbp) = Self::bp_and_prec(op);
            if lbp < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_pratt(rbp)?;
            lhs = Box::new(ExprNode {
                kind: ExprKind::Binary(op, lhs, rhs),
                span: self.span_from(start),
            });
        }
        Ok(lhs)
    }

    fn bp_and_prec(op: BinaryOp) -> (u8, u8) {
        match op {
            BinaryOp::Add | BinaryOp::Sub => (60, 61),
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => (70, 71),
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => (50, 51),
            BinaryOp::And => (40, 41),
            BinaryOp::Or => (30, 31),
            _ => (0, 0),
        }
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        let t = self.peek().ok_or(eof_err()).cloned()?;
        match t.kind {
            TokenKind::Minus => {
                self.advance();
                let rhs = self.parse_pratt(90)?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Unary(UnaryOp::Neg, rhs),
                    span: self.span_from(start),
                }))
            }
            TokenKind::Not => {
                self.advance();
                let rhs = self.parse_pratt(90)?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Unary(UnaryOp::Not, rhs),
                    span: self.span_from(start),
                }))
            }
            TokenKind::Star => {
                self.advance();
                let rhs = self.parse_pratt(90)?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Unary(UnaryOp::Deref, rhs),
                    span: self.span_from(start),
                }))
            }
            TokenKind::And => {
                self.advance();
                let rhs = self.parse_pratt(90)?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Unary(UnaryOp::Ref, rhs),
                    span: self.span_from(start),
                }))
            }
            TokenKind::IntLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::Int(t.text.parse().unwrap_or(0), IntBase::Dec)),
                    span: t.span,
                }))
            }
            TokenKind::FloatLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::Float(t.text.parse().unwrap_or(0.0))),
                    span: t.span,
                }))
            }
            TokenKind::StringLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::String(t.text.clone())),
                    span: t.span,
                }))
            }
            TokenKind::TrueLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::Bool(true)),
                    span: t.span,
                }))
            }
            TokenKind::FalseLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::Bool(false)),
                    span: t.span,
                }))
            }
            TokenKind::NullLiteral => {
                let t = self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Lit(Literal::Null),
                    span: t.span,
                }))
            }
            TokenKind::Ident => {
                let t = self.advance().clone();
                let id = Ident {
                    name: t.text.clone(),
                    span: t.span,
                };
                self.parse_postfix(Box::new(ExprNode {
                    kind: ExprKind::Ident(id),
                    span: t.span,
                }))
            }
            TokenKind::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::LBrace => Ok(Box::new(ExprNode {
                kind: ExprKind::Block(self.parse_block()?),
                span: self.span_from(start),
            })),
            TokenKind::KwIf => Ok(Box::new(ExprNode {
                kind: ExprKind::If(self.parse_if()?),
                span: self.span_from(start),
            })),
            TokenKind::KwMatch => Ok(Box::new(ExprNode {
                kind: ExprKind::Match(self.parse_match()?),
                span: self.span_from(start),
            })),
            TokenKind::KwLoop => Ok(Box::new(ExprNode {
                kind: ExprKind::Loop(self.parse_loop()?),
                span: self.span_from(start),
            })),
            TokenKind::KwWhile => Ok(Box::new(ExprNode {
                kind: ExprKind::While(self.parse_while()?),
                span: self.span_from(start),
            })),
            TokenKind::KwFor => Ok(Box::new(ExprNode {
                kind: ExprKind::For(self.parse_for()?),
                span: self.span_from(start),
            })),
            TokenKind::KwReturn => {
                self.advance();
                let e = if self.at(TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Return(e),
                    span: self.span_from(start),
                }))
            }
            TokenKind::KwBreak => {
                self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Break(None),
                    span: self.span_from(start),
                }))
            }
            TokenKind::KwLet => {
                let l = self.parse_let_stmt()?;
                let s = l.span;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Let(l),
                    span: s,
                }))
            }
            TokenKind::KwAsync => {
                self.advance();
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Async(self.parse_block()?),
                    span: self.span_from(start),
                }))
            }
            TokenKind::KwDischarge => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let mut thresholds = Vec::new();
                while !self.at(TokenKind::RBrace) {
                    let _thresh = self.parse_expr()?;
                    self.expect(TokenKind::FatArrow)?;
                    let body = self.parse_block()?;
                    thresholds.push((1.0, body));
                    if self.at(TokenKind::Comma) {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Discharge(e, thresholds),
                    span: self.span_from(start),
                }))
            }
            TokenKind::KwPerform => {
                self.advance();
                let op = self.parse_ident()?;
                self.expect(TokenKind::LParen)?;
                let args =
                    self.parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_expr())?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Perform(op, args),
                    span: self.span_from(start),
                }))
            }
            TokenKind::KwSpawn => {
                self.advance();
                let e = self.parse_expr()?;
                Ok(Box::new(ExprNode {
                    kind: ExprKind::Spawn(e),
                    span: self.span_from(start),
                }))
            }
            _ => Err(parse_err(
                vec![TokenKind::Ident, TokenKind::LParen, TokenKind::LBrace],
                &t,
            )),
        }
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            let span = expr.span;
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    self.advance();
                    let args = self
                        .parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_expr())?;
                    expr = Box::new(ExprNode {
                        kind: ExprKind::Call(expr, args),
                        span,
                    });
                }
                Some(TokenKind::Dot) => {
                    self.advance();
                    let f = self.parse_ident()?;
                    expr = Box::new(ExprNode {
                        kind: ExprKind::Member(expr, f),
                        span,
                    });
                }
                Some(TokenKind::LBracket) => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Box::new(ExprNode {
                        kind: ExprKind::Index(expr, idx),
                        span,
                    });
                }
                Some(TokenKind::Question) => {
                    self.advance();
                    expr = Box::new(ExprNode {
                        kind: ExprKind::CastGradual(expr, Type::Unknown),
                        span,
                    });
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_block(&mut self) -> Result<BlockExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut last: Option<Expr> = None;
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            if let Ok(item) = self.parse_top_level_item() {
                stmts.push(Stmt::Item(item));
            } else if self.at(TokenKind::KwLet) {
                stmts.push(Stmt::Let(self.parse_let_stmt()?));
            } else if self.at(TokenKind::KwReturn) {
                stmts.push(self.parse_return()?);
            } else {
                let e = self.parse_expr()?;
                if self.at(TokenKind::Semicolon) {
                    self.advance();
                    stmts.push(Stmt::Expr(e));
                } else {
                    last = Some(e);
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(BlockExpr {
            stmts,
            last,
            span: self.span_from(start),
        })
    }

    fn parse_if(&mut self) -> Result<IfExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwIf)?;
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.at(TokenKind::KwElse) {
            self.advance();
            Some(if self.at(TokenKind::KwIf) {
                Box::new(ElseBranch::If(Box::new(self.parse_if()?)))
            } else {
                Box::new(ElseBranch::Block(self.parse_block()?))
            })
        } else {
            None
        };
        Ok(IfExpr {
            cond,
            then_branch,
            else_branch,
            span: self.span_from(start),
        })
    }

    fn parse_match(&mut self) -> Result<MatchExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwMatch)?;
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.at(TokenKind::RBrace) {
            let pattern = self.parse_pattern()?;
            let guard = if self.at(TokenKind::KwIf) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expr()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: self.span_from(start),
            });
            if self.at(TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(MatchExpr {
            scrutinee,
            arms,
            span: self.span_from(start),
        })
    }

    fn parse_loop(&mut self) -> Result<LoopExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwLoop)?;
        let body = self.parse_block()?;
        Ok(LoopExpr {
            label: None,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_while(&mut self) -> Result<WhileExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwWhile)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(WhileExpr {
            label: None,
            cond,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_for(&mut self) -> Result<ForExpr, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwFor)?;
        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::KwIn)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(ForExpr {
            label: None,
            pattern,
            iter,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_let_stmt(&mut self) -> Result<LetStmt, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwLet)?;
        let is_mut = self.at(TokenKind::KwMut);
        if is_mut {
            self.advance();
        }
        let pattern = self.parse_pattern()?;
        let ty = if self.at(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Eq)?;
        let init = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(LetStmt {
            pattern,
            ty,
            init,
            is_mut,
            span: self.span_from(start),
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let start = self.pos;
        self.expect(TokenKind::KwReturn)?;
        let e = if self.at(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(ReturnStmt {
            expr: e,
            span: self.span_from(start),
        }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start = self.pos;
        let t = self.peek().ok_or(eof_err()).cloned()?;
        let kind = match t.kind {
            TokenKind::Minus => {
                self.advance();
                let lit = self.parse_literal()?;
                PatternKind::Lit(lit)
            }
            TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::CharLiteral
            | TokenKind::TrueLiteral
            | TokenKind::FalseLiteral
            | TokenKind::NullLiteral => PatternKind::Lit(self.parse_literal()?),
            TokenKind::Ident => {
                let id = self.parse_ident()?;
                if self.at(TokenKind::ColonColon)
                    || self.at(TokenKind::LBrace)
                    || self.at(TokenKind::LParen)
                {
                    let ty = Type::Named(id.clone());
                    if self.at(TokenKind::LBrace) {
                        self.advance();
                        let mut fields = Vec::new();
                        while !self.at(TokenKind::RBrace) {
                            let fname = self.parse_ident()?;
                            let fpat = if self.at(TokenKind::Colon) {
                                self.advance();
                                self.parse_pattern()?
                            } else {
                                Box::new(PatternNode {
                                    kind: PatternKind::Binding(fname.clone(), None),
                                    span: fname.span,
                                })
                            };
                            fields.push((fname, fpat));
                            if self.at(TokenKind::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        PatternKind::Struct(ty, fields)
                    } else {
                        PatternKind::Binding(id, None)
                    }
                } else {
                    PatternKind::Binding(id, None)
                }
            }
            TokenKind::LParen => {
                self.advance();
                let pats = self
                    .parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_pattern())?;
                PatternKind::Tuple(pats)
            }
            _ => return Err(parse_err(vec![TokenKind::Ident, TokenKind::LParen], &t)),
        };
        Ok(Box::new(PatternNode {
            kind,
            span: self.span_from(start),
        }))
    }

    fn parse_literal(&mut self) -> Result<Literal, ParseError> {
        let t = self.advance();
        match t.kind {
            TokenKind::IntLiteral => Ok(Literal::Int(t.text.parse().unwrap_or(0), IntBase::Dec)),
            TokenKind::FloatLiteral => Ok(Literal::Float(t.text.parse().unwrap_or(0.0))),
            TokenKind::StringLiteral => Ok(Literal::String(t.text.clone())),
            TokenKind::CharLiteral => Ok(Literal::Char(t.text.chars().next().unwrap_or('?'))),
            TokenKind::TrueLiteral => Ok(Literal::Bool(true)),
            TokenKind::FalseLiteral => Ok(Literal::Bool(false)),
            TokenKind::NullLiteral => Ok(Literal::Null),
            _ => Err(parse_err(
                vec![TokenKind::IntLiteral, TokenKind::StringLiteral],
                t,
            )),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let t = self.peek().ok_or(eof_err()).cloned()?;
        match t.kind {
            TokenKind::Ident => {
                let id = self.parse_ident()?;
                Ok(Type::Named(id))
            }
            TokenKind::And => {
                self.advance();
                Ok(Type::Ref(false, Box::new(self.parse_type()?), None))
            }
            TokenKind::Star => {
                self.advance();
                Ok(Type::Ptr(false, Box::new(self.parse_type()?)))
            }
            TokenKind::LParen => {
                self.advance();
                let tys =
                    self.parse_delimited(TokenKind::Comma, TokenKind::RParen, |p| p.parse_type())?;
                Ok(if tys.len() == 1 {
                    tys.into_iter().next().unwrap()
                } else {
                    Type::Tuple(tys)
                })
            }
            _ => Err(parse_err(vec![TokenKind::Ident], &t)),
        }
    }

    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        let t = self.advance();
        if t.kind == TokenKind::Ident
            || t.kind == TokenKind::KwSelf
            || (t.kind as u8 >= TokenKind::KwAgent as u8
                && t.kind as u8 <= TokenKind::KwWhile as u8)
        {
            Ok(Ident {
                name: t.text.clone(),
                span: t.span,
            })
        } else {
            Err(ParseError {
                expected: vec![TokenKind::Ident],
                found: t.kind,
                span: t.span,
            })
        }
    }

    fn parse_delimited<T>(
        &mut self,
        sep: TokenKind,
        end: TokenKind,
        f: impl Fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let mut items = Vec::new();
        if self.at(end) {
            self.advance();
            return Ok(items);
        }
        loop {
            items.push(f(self)?);
            if self.at(end) {
                self.advance();
                break;
            }
            self.expect(sep)?;
        }
        Ok(items)
    }
}

fn eof_err() -> ParseError {
    ParseError {
        expected: vec![TokenKind::Ident],
        found: TokenKind::Eof,
        span: SourceSpan::new(0.into(), 0),
    }
}

#[allow(clippy::unnecessary_cast)]
fn parse_err(expected: Vec<TokenKind>, found: &Token) -> ParseError {
    ParseError {
        expected,
        found: found.kind,
        span: found.span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_literal_int() {
        let e = parse_expr("42");
        match e.kind {
            ExprKind::Lit(Literal::Int(42, _)) => {}
            _ => panic!("expected int literal"),
        }
    }

    #[test]
    fn test_binary_expr() {
        let e = parse_expr("1 + 2 * 3");
        match e.kind {
            ExprKind::Binary(BinaryOp::Add, _, _) => {}
            _ => panic!("expected binary add at top"),
        }
    }

    fn parse_expr(source: &str) -> Expr {
        let tokens = tokenize(source).unwrap();
        let mut parser = Parser {
            tokens: &tokens,
            pos: 0,
        };
        parser.parse_expr().unwrap()
    }
}
