#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Method(MethodDecl),
    Function(FunctionDecl),
    Predicate(PredicateDecl),
    Datatype(DatatypeDecl),
    Class(ClassDecl),
    // Lemma等其他声明类型可在此补充
}

/* 方法声明 */
#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub id: String,
    pub params: Vec<Param>,
    pub returns: Vec<ReturnVar>,
    pub return_type: Option<Type>,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub modifies: Vec<Expr>,
    pub decreases: Vec<Expr>,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct ReturnVar {
    pub id: String,
    pub type_: Type,
}

/* 函数声明 */
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub id: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub reads: Vec<Expr>,
    pub decreases: Vec<Expr>,
    pub body: Expr,
}

/* 谓词声明 */
#[derive(Debug, Clone)]
pub struct PredicateDecl {
    pub id: String,
    pub params: Vec<Param>,
    pub requires: Vec<Expr>,
    pub body: Expr,
}

/* 数据类型声明 */
#[derive(Debug, Clone)]
pub struct DatatypeDecl {
    pub id: String,
    pub constructors: Vec<ConstructorDecl>,
}

#[derive(Debug, Clone)]
pub struct ConstructorDecl {
    pub params: Vec<Param>,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub modifies: Vec<Expr>,
    pub block: Block,
}

/* 类声明 */
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub id: String,
    pub extends: Option<Type>,
    pub fields: Vec<FieldDecl>,
    pub constructor: Option<ConstructorDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub id: String,
    pub type_: Type,
    pub init: Option<Expr>,
}

/* 参数和类型系统 */
#[derive(Debug, Clone)]
pub struct Param {
    pub id: String,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Bool,
    Real,
    Bv(u32),
    Set(Box<Type>),
    Seq(Box<Type>),
    Array(Box<Type>),
    Named(String),
    Function(Vec<Type>, Box<Type>),
}

/* 表达式系统 */
#[derive(Debug, Clone)]
pub enum Expr {
    LogicalOr(Box<Expr>, Box<Expr>),
    LogicalAnd(Box<Expr>, Box<Expr>),
    Equality(EqualityOp, Box<Expr>, Box<Expr>),
    Comparison(ComparisonOp, Box<Expr>, Box<Expr>),
    BitwiseOr(Box<Expr>, Box<Expr>),
    BitwiseXor(Box<Expr>, Box<Expr>),
    BitwiseAnd(Box<Expr>, Box<Expr>),
    Shift(ShiftOp, Box<Expr>, Box<Expr>),
    Additive(AdditiveOp, Box<Expr>, Box<Expr>),
    Mult(MultOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Primary(PrimaryExpr),
    Forall(Quantifier, Box<Expr>),
    Exists(Quantifier, Box<Expr>),
    IfThenElse(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum EqualityOp { Eq, Ne }
#[derive(Debug, Clone)]
pub enum ComparisonOp { Lt, Gt, Le, Ge }
#[derive(Debug, Clone)]
pub enum ShiftOp { Shl, Shr }
#[derive(Debug, Clone)]
pub enum AdditiveOp { Add, Sub }
#[derive(Debug, Clone)]
pub enum MultOp { Mul, Div, Mod }
#[derive(Debug, Clone)]
pub enum UnaryOp { Not, Neg, Old }

#[derive(Debug, Clone)]
pub enum PrimaryExpr {
    Literal(Literal),
    Identifier(String),
    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    MemberAccess(Box<Expr>, String),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(String),
    Boolean(bool),
    Null,
    Sequence(Vec<Expr>),
    Star, // represents any value
    This, // represents the current object
}

#[derive(Debug, Clone)]
pub struct Quantifier {
    pub variables: Vec<QuantifierVar>,
    pub condition: Option<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct QuantifierVar {
    pub id: String,
    pub type_: Type,
}

/* 语句系统 */
#[derive(Debug, Clone)]
pub enum Stmt {
    Assign(Assign),
    IfElse(IfElse),
    WhileLoop(WhileLoop),
    ForLoop(ForLoop),
    Match(Match),
    Assert(Expr),
    Print(Expr),
    Return(Option<Expr>),
    DeclVar(Var),
}

#[derive(Debug, Clone)]
pub struct Var {
    pub id: String,
    pub type_: Type,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub lhs: Lhs,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Lhs {
    Identifier(String),
    MemberAccess(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct IfElse {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub cond: Expr,
    pub invariants: Vec<Expr>,
    pub decreases: Vec<Expr>,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub id: String,
    pub start: Expr,
    pub end: Expr,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub expr: Expr,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub pattern: Pattern,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(String),
    Constructor(String, Vec<Pattern>),
}

/* 块和作用域 */
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}