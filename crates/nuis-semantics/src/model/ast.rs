#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirIntent {
    pub op: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstVisibility {
    Private,
    Public,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirVisibility {
    Private,
    Public,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstUse {
    pub domain: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstModule {
    pub attributes: Vec<AstAttribute>,
    pub uses: Vec<AstUse>,
    pub domain: String,
    pub unit: String,
    pub externs: Vec<AstExternFunction>,
    pub extern_interfaces: Vec<AstExternInterface>,
    pub consts: Vec<AstConstItem>,
    pub type_aliases: Vec<AstTypeAlias>,
    pub structs: Vec<AstStructDef>,
    pub enums: Vec<AstEnumDef>,
    pub traits: Vec<AstTraitDef>,
    pub impls: Vec<AstImplDef>,
    pub functions: Vec<AstFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstConstItem {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub ty: Option<AstTypeRef>,
    pub value: AstExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTypeAlias {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub generic_params: Vec<AstGenericParam>,
    pub where_bounds: Vec<AstWherePredicate>,
    pub target: AstTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExternFunction {
    pub visibility: AstVisibility,
    pub abi: String,
    pub interface: Option<String>,
    pub name: String,
    pub host_symbol: Option<String>,
    pub params: Vec<AstParam>,
    pub return_type: AstTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExternInterface {
    pub visibility: AstVisibility,
    pub abi: String,
    pub name: String,
    pub methods: Vec<AstExternFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructField {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub ty: AstTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructDef {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub generic_params: Vec<AstGenericParam>,
    pub where_bounds: Vec<AstWherePredicate>,
    pub fields: Vec<AstStructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstEnumDef {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub generic_params: Vec<AstGenericParam>,
    pub where_bounds: Vec<AstWherePredicate>,
    pub variants: Vec<AstEnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstEnumVariant {
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub kind: AstEnumVariantKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstEnumVariantKind {
    Unit,
    Tuple(Vec<AstTypeRef>),
    Struct(Vec<AstStructField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstAttributeValue {
    Bool(bool),
    Int(i64),
    String(String),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstAttributeArg {
    pub name: Option<String>,
    pub value: AstAttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstAttribute {
    pub name: String,
    pub args: Vec<AstAttributeArg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGenericParam {
    pub name: String,
    pub bounds: Vec<AstTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstWherePredicate {
    pub param_name: String,
    pub bounds: Vec<AstTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTraitMethodSig {
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
    pub default_body: Option<Vec<AstStmt>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTraitDef {
    pub visibility: AstVisibility,
    pub attributes: Vec<AstAttribute>,
    pub name: String,
    pub methods: Vec<AstTraitMethodSig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstImplMethod {
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstImplDef {
    pub generic_params: Vec<AstGenericParam>,
    pub where_bounds: Vec<AstWherePredicate>,
    pub trait_name: String,
    pub for_type: AstTypeRef,
    pub methods: Vec<AstImplMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTypeRef {
    pub name: String,
    pub generic_args: Vec<AstTypeRef>,
    pub is_optional: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstParam {
    pub name: String,
    pub ty: AstTypeRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestClockDomain {
    Monotonic,
    Wall,
    Global,
}

impl TestClockDomain {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "monotonic" => Some(Self::Monotonic),
            "wall" => Some(Self::Wall),
            "global" => Some(Self::Global),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Monotonic => "monotonic",
            Self::Wall => "wall",
            Self::Global => "global",
        }
    }

    pub fn code(self) -> i64 {
        match self {
            Self::Monotonic => 0,
            Self::Wall => 1,
            Self::Global => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestClockPolicy {
    Bridge,
}

impl TestClockPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "bridge" => Some(Self::Bridge),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bridge => "bridge",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFunction {
    pub visibility: AstVisibility,
    pub name: String,
    pub attributes: Vec<AstAttribute>,
    pub test_name: Option<String>,
    pub test_ignored: bool,
    pub test_should_fail: bool,
    pub test_reason: Option<String>,
    pub test_timeout_ms: Option<i64>,
    pub test_clock_domain: Option<TestClockDomain>,
    pub test_clock_policy: Option<TestClockPolicy>,
    pub benchmark_name: Option<String>,
    pub benchmark_warmup_iters: Option<i64>,
    pub benchmark_measure_iters: Option<i64>,
    pub benchmark_timeout_ms: Option<i64>,
    pub benchmark_clock_domain: Option<TestClockDomain>,
    pub benchmark_clock_policy: Option<TestClockPolicy>,
    pub is_async: bool,
    pub generic_params: Vec<AstGenericParam>,
    pub where_bounds: Vec<AstWherePredicate>,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstMatchPattern {
    Wildcard,
    Bind(String),
    Bool(bool),
    Int(i64),
    IntRangeInclusive(i64, i64),
    Or(Vec<AstMatchPattern>),
    PayloadStruct {
        type_ref: AstTypeRef,
        payload: Box<AstMatchPattern>,
    },
    StructFields {
        type_ref: Option<AstTypeRef>,
        fields: Vec<(String, AstMatchPattern)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstMatchArm {
    pub pattern: AstMatchPattern,
    pub guard: Option<AstExpr>,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstDestructureBinding {
    Bind(String),
    Ignore,
    Nested {
        type_ref: Option<AstTypeRef>,
        fields: Vec<AstDestructureField>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub struct AstDestructureField { pub field: String, pub binding: AstDestructureBinding }

#[derive(Debug, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub enum AstStmt {
    Let { mutable: bool, name: String, ty: Option<AstTypeRef>, value: AstExpr },
    AssignLocal { name: String, value: AstExpr },
    DestructureLet { type_ref: Option<AstTypeRef>, fields: Vec<AstDestructureField>, value: AstExpr },
    Const { name: String, ty: Option<AstTypeRef>, value: AstExpr },
    Print(AstExpr), Await(AstExpr),
    If { condition: AstExpr, then_body: Vec<AstStmt>, else_body: Vec<AstStmt> },
    Match { value: AstExpr, arms: Vec<AstMatchArm> },
    While { condition: AstExpr, body: Vec<AstStmt> },
    Break, Continue, Expr(AstExpr), Return(Option<AstExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpr {
    Bool(bool),
    Text(String),
    Int(i64),
    Float(String),
    Var(String),
    If {
        condition: Box<AstExpr>,
        then_body: Vec<AstStmt>,
        else_body: Vec<AstStmt>,
    },
    Match {
        value: Box<AstExpr>,
        arms: Vec<AstMatchArm>,
    },
    Lambda {
        params: Vec<AstParam>,
        return_type: Option<AstTypeRef>,
        body: Vec<AstStmt>,
    },
    Await(Box<AstExpr>),
    Try(Box<AstExpr>),
    Instantiate {
        domain: String,
        unit: String,
    },
    Call {
        callee: String,
        generic_args: Vec<AstTypeRef>,
        args: Vec<AstExpr>,
    },
    Invoke {
        callee: Box<AstExpr>,
        args: Vec<AstExpr>,
    },
    MethodCall {
        receiver: Box<AstExpr>,
        method: String,
        generic_args: Vec<AstTypeRef>,
        args: Vec<AstExpr>,
    },
    StructLiteral {
        type_name: String,
        type_args: Vec<AstTypeRef>,
        fields: Vec<(String, AstExpr)>,
    },
    FieldAccess {
        base: Box<AstExpr>,
        field: String,
    },
    Unary {
        op: AstUnaryOp,
        operand: Box<AstExpr>,
    },
    Binary {
        op: AstBinaryOp,
        lhs: Box<AstExpr>,
        rhs: Box<AstExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstUnaryOp {
    Not,
    Neg,
    Deref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstBinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
