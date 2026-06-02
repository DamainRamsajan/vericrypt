//! Complete Abstract Syntax Tree for AGENT‑SEED v15.2.

pub use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
    pub span: SourceSpan,
}

impl Default for Program {
    fn default() -> Self {
        Program {
            items: vec![],
            span: SourceSpan::new(0.into(), 0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Seed(SeedDecl),
    Agent(AgentDecl),
    Section(SectionDecl),
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Mod(ModDecl),
    Use(UseDecl),
    Extern(ExternBlock),
    Effect(EffectDecl),
    Handler(HandlerDecl),
    Expression(Expr),
    Clause(Ident, BlockExpr), // ← NEW
}

// ── Declarations ──

#[derive(Debug, Clone)]
pub struct SeedDecl {
    pub name: Ident,
    pub sections: Vec<SectionDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct AgentDecl {
    pub name: Ident,
    pub generic_params: Vec<GenericParam>,
    pub extends: Option<Type>,
    pub capabilities: Vec<Capability>,
    pub members: Vec<AgentMember>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum AgentMember {
    Field(FieldDecl),
    Method(FnDecl),
    Lifecycle(LifecycleBlock),
    StateMachine(StateMachineDecl),
    SignalHandler(SignalHandlerDecl),
    Clause(Ident, BlockExpr), // ← NEW
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LifecycleBlock {
    pub handlers: Vec<LifecycleHandler>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LifecycleHandler {
    pub event: LifecycleEvent,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    OnBoot,
    OnSessionStart,
    OnSessionEnd,
    OnError,
    OnDispose,
    OnInterrupt,
    OnDream,
    OnDawn,
    OnTick,
}

#[derive(Debug, Clone)]
pub struct StateMachineDecl {
    pub initial: Ident,
    pub states: Vec<StateDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: Ident,
    pub transitions: Vec<Transition>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub target: Ident,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SignalHandlerDecl {
    pub signal: Ident,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub name: Ident,
    pub ty: Option<Type>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: Ident,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub effect_set: Option<EffectSet>,
    pub body: Option<BlockExpr>,
    pub vis: Visibility,
    pub is_async: bool,
    pub is_train: bool,
    pub is_evolve: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub is_mut: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EffectSet {
    pub effects: Vec<Ident>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Pub,
    Priv,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: Ident,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<FieldDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: Ident,
    pub generic_params: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub payload: Option<Vec<Type>>,
    pub discr: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: Ident,
    pub methods: Vec<TraitMethod>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub sig: FnSig,
    pub default: Option<BlockExpr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ImplDecl {
    pub generic_params: Vec<GenericParam>,
    pub trait_path: Option<Type>,
    pub target: Type,
    pub items: Vec<ImplItem>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum ImplItem {
    Fn(FnDecl),
    Type(TypeDecl),
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: Ident,
    pub ty: Type,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub name: Ident,
    pub items: Option<Vec<TopLevelItem>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: UsePath,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct UsePath {
    pub segments: Vec<Ident>,
    pub imported: Option<Vec<Ident>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub lang: Option<Ident>,
    pub items: Vec<FnSig>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_ty: Option<Type>,
    pub effect_set: Option<EffectSet>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: Ident,
    pub bounds: Vec<Type>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct SectionDecl {
    pub name: Ident,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<FieldDecl>,
    pub annotations: Vec<Annotation>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: Ident,
    pub args: Option<Vec<Expr>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EffectDecl {
    pub name: Ident,
    pub operations: Vec<EffectOp>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct EffectOp {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct HandlerDecl {
    pub name: Ident,
    pub effect: Type,
    pub clauses: Vec<HandlerClause>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct HandlerClause {
    pub op_name: Ident,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

// ── Statements ──

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Expr(Expr),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Item(TopLevelItem),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Expr,
    pub is_mut: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub expr: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub span: SourceSpan,
}

// ── Expressions ──

#[derive(Debug, Clone)]
pub struct ExprNode {
    pub kind: ExprKind,
    pub span: SourceSpan,
}
pub type Expr = Box<ExprNode>;

#[derive(Debug, Clone)]
pub enum ExprKind {
    Lit(Literal),
    Ident(Ident),
    Binary(BinaryOp, Expr, Expr),
    Unary(UnaryOp, Expr),
    Call(Expr, Vec<Expr>),
    Method(Expr, Ident, Vec<Expr>),
    Member(Expr, Ident),
    Index(Expr, Expr),
    Field(Expr, Ident),
    Block(BlockExpr),
    If(IfExpr),
    Match(MatchExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    For(ForExpr),
    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,
    Let(LetStmt),
    Closure(ClosureExpr),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),
    StructLit(Type, Vec<(Ident, Expr)>),
    EnumLit(Type, Ident, Option<Vec<Expr>>),
    Pipeline(Expr, PipelineOp, Expr),
    Redirect(Expr, RedirectOp, Expr),
    ProcessSub(ProcessSubKind, Expr),
    HereDoc(String),
    HereString(Expr),
    Assignment(Expr, AssignOp, Expr),
    Range(Expr, RangeKind, Expr),
    Cast(Expr, Type),
    CastGradual(Expr, Type),
    Ask(Option<Type>, Expr),
    Confident(Expr, ConfidenceLevel),
    Think(ThinkDepth, Expr),
    Discharge(Expr, Vec<(f64, BlockExpr)>),
    Perform(Ident, Vec<Expr>),
    Spawn(Expr),
    Train(TrainConfig, BlockExpr),
    Evolve(BlockExpr),
    Signal(SignalDecl),
    React(Ident, Vec<ReactRule>),
    Memo(String, Expr),
    Observe(Expr),
    Infer(Expr),
    Ontology(Ident, Vec<OntologyRule>),
    Route(ModelTier, Expr),
    Await(Expr),
    Async(BlockExpr),
    Yield(Option<Expr>),
    Select(Vec<SelectBranch>),
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub stmts: Vec<Stmt>,
    pub last: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Expr,
    pub then_branch: BlockExpr,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    Block(BlockExpr),
    If(Box<IfExpr>),
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Expr,
    pub arms: Vec<MatchArm>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub label: Option<Ident>,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub label: Option<Ident>,
    pub cond: Expr,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ForExpr {
    pub label: Option<Ident>,
    pub pattern: Pattern,
    pub iter: Expr,
    pub body: BlockExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub body: BlockExpr,
    pub captures: CaptureMode,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Move,
    Borrow,
}

#[derive(Debug, Clone)]
pub enum PipelineOp {
    Pipe,
    PipeAppend,
    PipeErr,
}

#[derive(Debug, Clone)]
pub enum RedirectOp {
    Stdout,
    StdoutAppend,
    Stdin,
    Stderr,
    StderrToStdout,
    StdoutToStderr,
    DupInput(u32),
    DupOutput(u32),
    CloseFd(u32),
}

#[derive(Debug, Clone)]
pub enum ProcessSubKind {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct TrainConfig {
    pub algorithm: Ident,
    pub reward: Expr,
    pub episodes: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct SignalDecl {
    pub name: Ident,
    pub ty: Type,
    pub frequency: Option<SignalFrequency>,
    pub init: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum SignalFrequency {
    High,
    Medium,
    Low,
    Periodic(u64),
}

#[derive(Debug, Clone)]
pub struct ReactRule {
    pub condition: Expr,
    pub body: BlockExpr,
}

#[derive(Debug, Clone)]
pub enum ThinkDepth {
    Shallow,
    Medium,
    Deep,
    Exhaustive,
    Budget(u64),
}

#[derive(Debug, Clone)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    Custom(f64),
}

#[derive(Debug, Clone)]
pub enum ModelTier {
    LocalSlm(Ident),
    CloudMid(Ident),
    Frontier(Ident),
}

#[derive(Debug, Clone)]
pub struct SelectBranch {
    pub pattern: Pattern,
    pub future: Expr,
    pub handler: BlockExpr,
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AndEq,
    OrEq,
    XorEq,
    ShlEq,
    ShrEq,
}

#[derive(Debug, Clone)]
pub enum RangeKind {
    Exclusive,
    Inclusive,
}

#[derive(Debug, Clone)]
pub struct OntologyRule {
    pub name: Ident,
    pub condition: Expr,
    pub require: Expr,
    pub violation: Ident,
}

// ── Patterns ──

#[derive(Debug, Clone)]
pub struct PatternNode {
    pub kind: PatternKind,
    pub span: SourceSpan,
}
pub type Pattern = Box<PatternNode>;

#[derive(Debug, Clone)]
pub enum PatternKind {
    Wildcard,
    Binding(Ident, Option<Pattern>),
    Lit(Literal),
    Tuple(Vec<Pattern>),
    Struct(Type, Vec<(Ident, Pattern)>),
    EnumVariant(Type, Ident, Option<Vec<Pattern>>),
    Or(Vec<Pattern>),
    Range(Expr, RangeKind, Expr),
    Rest,
}

// ── Literals ──

#[derive(Debug, Clone)]
pub enum Literal {
    Int(u64, IntBase),
    Float(f64),
    String(String),
    RawString(String),
    Char(char),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntBase {
    Dec,
    Hex,
    Oct,
    Bin,
}

// ── Operators ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    RefMut,
    Try,
}

// ── Types ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(PrimitiveType),
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Fn(Vec<Type>, Box<Type>),
    Ref(bool, Box<Type>, Option<Lifetime>),
    Ptr(bool, Box<Type>),
    Agent(Ident),
    Section(Ident),
    Named(Ident),
    Generic(Ident),
    Effectful(Box<Type>, Vec<Ident>),
    Dynamic(Box<Type>),
    Union(Box<Type>, Box<Type>),
    Intersection(Box<Type>, Box<Type>),
    Unknown,
}
