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
    pub uses: Vec<AstUse>,
    pub domain: String,
    pub unit: String,
    pub externs: Vec<AstExternFunction>,
    pub extern_interfaces: Vec<AstExternInterface>,
    pub consts: Vec<AstConstItem>,
    pub type_aliases: Vec<AstTypeAlias>,
    pub structs: Vec<AstStructDef>,
    pub traits: Vec<AstTraitDef>,
    pub impls: Vec<AstImplDef>,
    pub functions: Vec<AstFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstConstItem {
    pub visibility: AstVisibility,
    pub name: String,
    pub ty: Option<AstTypeRef>,
    pub value: AstExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTypeAlias {
    pub visibility: AstVisibility,
    pub name: String,
    pub generic_params: Vec<AstGenericParam>,
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
    pub fields: Vec<AstStructField>,
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
    pub bound: Option<AstTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTraitMethodSig {
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstTraitDef {
    pub visibility: AstVisibility,
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
    pub is_async: bool,
    pub generic_params: Vec<AstGenericParam>,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstMatchPattern {
    Wildcard,
    Bool(bool),
    Int(i64),
    IntRangeInclusive(i64, i64),
    Or(Vec<AstMatchPattern>),
    StructFields {
        type_ref: AstTypeRef,
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
pub struct AstDestructureField {
    pub field: String,
    pub binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[rustfmt::skip]
pub enum AstStmt {
    Let { name: String, ty: Option<AstTypeRef>, value: AstExpr },
    DestructureLet { type_ref: AstTypeRef, fields: Vec<AstDestructureField>, value: AstExpr },
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
    Var(String),
    Lambda {
        params: Vec<AstParam>,
        return_type: Option<AstTypeRef>,
        body: Vec<AstStmt>,
    },
    Await(Box<AstExpr>),
    Instantiate {
        domain: String,
        unit: String,
    },
    Call {
        callee: String,
        args: Vec<AstExpr>,
    },
    Invoke {
        callee: Box<AstExpr>,
        args: Vec<AstExpr>,
    },
    MethodCall {
        receiver: Box<AstExpr>,
        method: String,
        args: Vec<AstExpr>,
    },
    StructLiteral {
        type_name: String,
        fields: Vec<(String, AstExpr)>,
    },
    FieldAccess {
        base: Box<AstExpr>,
        field: String,
    },
    Binary {
        op: AstBinaryOp,
        lhs: Box<AstExpr>,
        rhs: Box<AstExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstBinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirUse {
    pub domain: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirModule {
    pub uses: Vec<NirUse>,
    pub domain: String,
    pub unit: String,
    pub externs: Vec<NirExternFunction>,
    pub extern_interfaces: Vec<NirExternInterface>,
    pub consts: Vec<NirConstItem>,
    pub type_aliases: Vec<NirTypeAlias>,
    pub structs: Vec<NirStructDef>,
    pub traits: Vec<NirTraitDef>,
    pub impls: Vec<NirImplDef>,
    pub functions: Vec<NirFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirConstItem {
    pub visibility: NirVisibility,
    pub name: String,
    pub ty: NirTypeRef,
    pub value: NirExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirTypeAlias {
    pub visibility: NirVisibility,
    pub name: String,
    pub generic_params: Vec<NirGenericParam>,
    pub target: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirExternFunction {
    pub visibility: NirVisibility,
    pub abi: String,
    pub interface: Option<String>,
    pub name: String,
    pub host_symbol: Option<String>,
    pub params: Vec<NirParam>,
    pub return_type: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirExternInterface {
    pub visibility: NirVisibility,
    pub abi: String,
    pub name: String,
    pub methods: Vec<NirExternFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirStructField {
    pub visibility: NirVisibility,
    pub annotations: Vec<NirAnnotation>,
    pub name: String,
    pub ty: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirStructDef {
    pub visibility: NirVisibility,
    pub annotations: Vec<NirAnnotation>,
    pub name: String,
    pub fields: Vec<NirStructField>,
}

impl NirStructDef {
    pub fn field(&self, name: &str) -> Option<&NirStructField> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirGenericParam {
    pub name: String,
    pub bound: Option<NirTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirAttributeValue {
    Bool(bool),
    Int(i64),
    String(String),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirAttributeArg {
    pub name: Option<String>,
    pub value: NirAttributeValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirAnnotation {
    pub name: String,
    pub args: Vec<NirAttributeArg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirTraitMethodSig {
    pub name: String,
    pub params: Vec<NirParam>,
    pub return_type: Option<NirTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirTraitDef {
    pub visibility: NirVisibility,
    pub name: String,
    pub methods: Vec<NirTraitMethodSig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirImplMethod {
    pub name: String,
    pub params: Vec<NirParam>,
    pub return_type: Option<NirTypeRef>,
    pub body: Vec<NirStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirImplDef {
    pub trait_name: String,
    pub for_type: NirTypeRef,
    pub methods: Vec<NirImplMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirTypeRef {
    pub name: String,
    pub generic_args: Vec<NirTypeRef>,
    pub is_optional: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirScalarKind {
    Bool,
    I32,
    I64,
    F32,
    F64,
    Text,
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirResultFamily {
    Task,
    Data,
    Shader,
    Kernel,
    Network,
}

impl NirResultFamily {
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Task => "TaskResult",
            Self::Data => "DataResult",
            Self::Shader => "ShaderResult",
            Self::Kernel => "KernelResult",
            Self::Network => "NetworkResult",
        }
    }

    pub fn supports_stage(self, stage: NirResultStage) -> bool {
        self == stage.family()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirTypeShape {
    Scalar(NirScalarKind),
    Ref,
    Generic,
    Nominal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirContainerKind {
    Window,
    Pipe,
    Instance,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirWindowMode {
    Mutable,
    Immutable,
}

impl NirTypeRef {
    pub fn scalar_kind(&self) -> Option<NirScalarKind> {
        if self.is_ref || !self.generic_args.is_empty() {
            return None;
        }
        match self.name.as_str() {
            "bool" => Some(NirScalarKind::Bool),
            "i32" => Some(NirScalarKind::I32),
            "i64" => Some(NirScalarKind::I64),
            "f32" => Some(NirScalarKind::F32),
            "f64" => Some(NirScalarKind::F64),
            "String" => Some(NirScalarKind::Text),
            "Unit" => Some(NirScalarKind::Unit),
            _ => None,
        }
    }

    pub fn shape(&self) -> NirTypeShape {
        if let Some(kind) = self.scalar_kind() {
            NirTypeShape::Scalar(kind)
        } else if self.is_ref {
            NirTypeShape::Ref
        } else if !self.generic_args.is_empty() {
            NirTypeShape::Generic
        } else {
            NirTypeShape::Nominal
        }
    }

    pub fn is_integer_scalar(&self) -> bool {
        matches!(
            self.scalar_kind(),
            Some(NirScalarKind::I32 | NirScalarKind::I64)
        )
    }

    pub fn is_float_scalar(&self) -> bool {
        matches!(
            self.scalar_kind(),
            Some(NirScalarKind::F32 | NirScalarKind::F64)
        )
    }

    pub fn is_numeric_scalar(&self) -> bool {
        self.is_integer_scalar() || self.is_float_scalar()
    }

    pub fn is_bool_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Bool)
    }

    pub fn is_text_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Text)
    }

    pub fn is_unit_scalar(&self) -> bool {
        self.scalar_kind() == Some(NirScalarKind::Unit)
    }

    pub fn is_generic_named(&self, expected: &str, arity: usize) -> bool {
        self.name == expected && self.generic_args.len() == arity && !self.is_ref
    }

    pub fn container_kind(&self) -> Option<NirContainerKind> {
        match self.name.as_str() {
            "Window" | "WindowMut" if !self.is_ref => Some(NirContainerKind::Window),
            "Pipe" if !self.is_ref => Some(NirContainerKind::Pipe),
            "Instance" if !self.is_ref => Some(NirContainerKind::Instance),
            "Task" if !self.is_ref => Some(NirContainerKind::Task),
            _ => None,
        }
    }

    pub fn window_mode(&self) -> Option<NirWindowMode> {
        if self.is_ref {
            return None;
        }
        match self.name.as_str() {
            "Window" => Some(NirWindowMode::Immutable),
            "WindowMut" => Some(NirWindowMode::Mutable),
            _ => None,
        }
    }

    pub fn container_payload(&self) -> Option<&NirTypeRef> {
        if matches!(
            self.container_kind(),
            Some(
                NirContainerKind::Window
                    | NirContainerKind::Pipe
                    | NirContainerKind::Instance
                    | NirContainerKind::Task
            )
        ) {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn is_marker_type(&self) -> bool {
        self.name == "Marker" && !self.is_ref && self.generic_args.is_empty()
    }

    pub fn is_handle_table_type(&self) -> bool {
        self.name == "HandleTable" && !self.is_ref && self.generic_args.is_empty()
    }

    pub fn is_marker_family(&self) -> bool {
        self.name == "Marker" && !self.is_ref
    }

    pub fn is_handle_table_family(&self) -> bool {
        self.name == "HandleTable" && !self.is_ref
    }

    pub fn marker_tag(&self) -> Option<&NirTypeRef> {
        if self.is_marker_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn handle_table_schema(&self) -> Option<&NirTypeRef> {
        if self.is_handle_table_family() {
            self.generic_args.first()
        } else {
            None
        }
    }

    pub fn is_async_boundary_safe(&self) -> bool {
        if self.is_ref || self.is_optional {
            return false;
        }
        if matches!(
            self.container_kind(),
            Some(NirContainerKind::Instance | NirContainerKind::Task)
        ) {
            return false;
        }
        if self.is_result_family() {
            return false;
        }
        self.generic_args
            .iter()
            .all(NirTypeRef::is_async_boundary_safe)
    }

    pub fn is_result_family(&self) -> bool {
        self.result_family().is_some()
    }

    pub fn result_family(&self) -> Option<NirResultFamily> {
        if self.is_ref || self.generic_args.len() != 1 {
            return None;
        }
        match self.name.as_str() {
            "TaskResult" => Some(NirResultFamily::Task),
            "DataResult" => Some(NirResultFamily::Data),
            "ShaderResult" => Some(NirResultFamily::Shader),
            "KernelResult" => Some(NirResultFamily::Kernel),
            "NetworkResult" => Some(NirResultFamily::Network),
            _ => None,
        }
    }

    pub fn result_payload(&self) -> Option<&NirTypeRef> {
        self.result_family()?;
        self.generic_args.first()
    }

    fn is_nominal_semantic_payload(&self) -> bool {
        !self.is_ref
            && !self.is_optional
            && self.scalar_kind().is_none()
            && self.container_kind().is_none()
            && !self.is_marker_family()
            && !self.is_handle_table_family()
    }

    pub fn validate_container_contract(&self) -> Result<(), String> {
        for arg in &self.generic_args {
            arg.validate_container_contract()?;
        }

        match self.container_kind() {
            Some(NirContainerKind::Window) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("window payload");
                if payload.is_marker_type() || payload.is_handle_table_type() {
                    return Err(format!(
                        "`Window<...>` cannot carry control-plane payload `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Pipe) {
                    return Err("`Window<Pipe<...>>` is not a valid memory payload".to_owned());
                }
            }
            Some(NirContainerKind::Pipe) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("pipe payload");
                if payload.is_marker_type() || payload.is_handle_table_type() {
                    return Err(format!(
                        "`Pipe<...>` cannot carry control-plane payload `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Pipe) {
                    return Err("`Pipe<Pipe<...>>` is not a legal fabric primitive".to_owned());
                }
            }
            Some(NirContainerKind::Instance) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("instance payload");
                if payload.is_ref
                    || payload.is_optional
                    || payload.scalar_kind().is_some()
                    || payload.is_marker_type()
                    || payload.is_handle_table_type()
                    || payload.container_kind().is_some()
                {
                    return Err(format!(
                        "`Instance<...>` expects a nominal unit type, found `{}`",
                        payload.render()
                    ));
                }
            }
            Some(NirContainerKind::Task) => {
                if self.generic_args.len() != 1 {
                    return Err(format!(
                        "`{}` must carry exactly one payload type argument",
                        self.name
                    ));
                }
                let payload = self.container_payload().expect("task payload");
                if !payload.is_async_boundary_safe() {
                    return Err(format!(
                        "`Task<...>` expects an async-boundary-safe payload, found `{}`",
                        payload.render()
                    ));
                }
                if payload.container_kind() == Some(NirContainerKind::Task) {
                    return Err(
                        "`Task<Task<...>>` is not a supported explicit async primitive".to_owned(),
                    );
                }
            }
            None => {
                if self.is_marker_family() {
                    if self.generic_args.len() > 1 {
                        return Err("`Marker<...>` accepts at most one tag type".to_owned());
                    }
                    if let Some(tag) = self.marker_tag() {
                        if !tag.is_nominal_semantic_payload() {
                            return Err(format!(
                                "`Marker<...>` expects a nominal tag type, found `{}`",
                                tag.render()
                            ));
                        }
                    }
                }
                if self.is_handle_table_family() {
                    if self.generic_args.len() > 1 {
                        return Err("`HandleTable<...>` accepts at most one schema type".to_owned());
                    }
                    if let Some(schema) = self.handle_table_schema() {
                        if !schema.is_nominal_semantic_payload() {
                            return Err(format!(
                                "`HandleTable<...>` expects a nominal schema type, found `{}`",
                                schema.render()
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        if self.is_ref {
            out.push_str("ref ");
        }
        out.push_str(&self.name);
        if !self.generic_args.is_empty() {
            out.push('<');
            for (index, arg) in self.generic_args.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&arg.render());
            }
            out.push('>');
        }
        if self.is_optional {
            out.push('?');
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirParam {
    pub name: String,
    pub ty: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirFunction {
    pub visibility: NirVisibility,
    pub name: String,
    pub annotations: Vec<NirAnnotation>,
    pub test_name: Option<String>,
    pub test_ignored: bool,
    pub test_should_fail: bool,
    pub test_reason: Option<String>,
    pub test_timeout_ms: Option<i64>,
    pub test_clock_domain: Option<TestClockDomain>,
    pub test_clock_policy: Option<TestClockPolicy>,
    pub is_async: bool,
    pub generic_params: Vec<NirGenericParam>,
    pub params: Vec<NirParam>,
    pub return_type: Option<NirTypeRef>,
    pub body: Vec<NirStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirStmt {
    Let {
        name: String,
        ty: Option<NirTypeRef>,
        value: NirExpr,
    },
    Const {
        name: String,
        ty: NirTypeRef,
        value: NirExpr,
    },
    Print(NirExpr),
    Await(NirExpr),
    If {
        condition: NirExpr,
        then_body: Vec<NirStmt>,
        else_body: Vec<NirStmt>,
    },
    While {
        condition: NirExpr,
        body: Vec<NirStmt>,
    },
    Break,
    Continue,
    Expr(NirExpr),
    Return(Option<NirExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirGlmValueClass {
    Val,
    Res,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirGlmUseMode {
    Own,
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirGlmEffect {
    None,
    DomainMove,
    LifetimeEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirExprEffectClass {
    Pure,
    LocalReadOnly,
    HostReadOnly,
    DomainReadOnly,
    AsyncOpaque,
    CallOpaque,
    DomainOpaque,
    Stateful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirHostReadSurface {
    SchedulerLane,
    InputChannel,
    ClockTick,
    RenderDescriptor,
}

impl NirHostReadSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SchedulerLane => "scheduler_lane",
            Self::InputChannel => "input_channel",
            Self::ClockTick => "clock_tick",
            Self::RenderDescriptor => "render_descriptor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirHostSchedulerBridgeKind {
    HostMainLane,
    WorkerLane,
}

impl NirHostSchedulerBridgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HostMainLane => "host_main_lane",
            Self::WorkerLane => "worker_lane",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NirHostSchedulerBridge {
    pub kind: NirHostSchedulerBridgeKind,
    pub lane: i64,
}

impl NirHostSchedulerBridge {
    pub fn from_cpu_bind_core(lane: i64) -> Self {
        Self {
            kind: if lane == 0 {
                NirHostSchedulerBridgeKind::HostMainLane
            } else {
                NirHostSchedulerBridgeKind::WorkerLane
            },
            lane,
        }
    }

    pub fn resolved_source(self) -> &'static str {
        "cpu_bind_core_lane"
    }

    pub fn host_surface(self) -> NirHostReadSurface {
        NirHostReadSurface::SchedulerLane
    }

    pub fn as_str(self) -> &'static str {
        self.kind.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirHostTimingBridge {
    MonotonicTick,
    WallDeadline,
    GlobalToMonotonicTickBridge,
}

impl NirHostTimingBridge {
    pub fn from_test_clock_domain(domain: TestClockDomain) -> Self {
        match domain {
            TestClockDomain::Monotonic => Self::MonotonicTick,
            TestClockDomain::Wall => Self::WallDeadline,
            TestClockDomain::Global => Self::GlobalToMonotonicTickBridge,
        }
    }

    pub fn resolved_domain(self) -> TestClockDomain {
        match self {
            Self::MonotonicTick => TestClockDomain::Monotonic,
            Self::WallDeadline => TestClockDomain::Wall,
            Self::GlobalToMonotonicTickBridge => TestClockDomain::Monotonic,
        }
    }

    pub fn resolved_source(self) -> &'static str {
        match self {
            Self::MonotonicTick => "host_monotonic_deadline",
            Self::WallDeadline => "host_wall_deadline",
            Self::GlobalToMonotonicTickBridge => "host_monotonic_deadline",
        }
    }

    pub fn host_surface(self) -> NirHostReadSurface {
        match self {
            Self::MonotonicTick => NirHostReadSurface::ClockTick,
            Self::WallDeadline => NirHostReadSurface::ClockTick,
            Self::GlobalToMonotonicTickBridge => NirHostReadSurface::ClockTick,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MonotonicTick => "monotonic_tick",
            Self::WallDeadline => "wall_deadline",
            Self::GlobalToMonotonicTickBridge => "global_to_monotonic_tick_bridge",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirGlmAccess {
    pub class: NirGlmValueClass,
    pub mode: NirGlmUseMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirGlmProfile {
    pub result_class: NirGlmValueClass,
    pub accesses: Vec<NirGlmAccess>,
    pub effect: NirGlmEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirExpr {
    Bool(bool),
    Text(String),
    Int(i64),
    Var(String),
    Await(Box<NirExpr>),
    Instantiate {
        domain: String,
        unit: String,
    },
    Null,
    Borrow(Box<NirExpr>),
    BorrowEnd(Box<NirExpr>),
    Move(Box<NirExpr>),
    AllocNode {
        value: Box<NirExpr>,
        next: Box<NirExpr>,
    },
    AllocBuffer {
        len: Box<NirExpr>,
        fill: Box<NirExpr>,
    },
    LoadValue(Box<NirExpr>),
    LoadNext(Box<NirExpr>),
    BufferLen(Box<NirExpr>),
    LoadAt {
        buffer: Box<NirExpr>,
        index: Box<NirExpr>,
    },
    StoreValue {
        target: Box<NirExpr>,
        value: Box<NirExpr>,
    },
    StoreNext {
        target: Box<NirExpr>,
        next: Box<NirExpr>,
    },
    StoreAt {
        buffer: Box<NirExpr>,
        index: Box<NirExpr>,
        value: Box<NirExpr>,
    },
    DataBindCore(i64),
    DataMarker(String),
    DataOutputPipe(Box<NirExpr>),
    DataInputPipe(Box<NirExpr>),
    DataResult {
        value: Box<NirExpr>,
        state: NirDataFlowState,
    },
    DataReady(Box<NirExpr>),
    DataMoved(Box<NirExpr>),
    DataWindowed(Box<NirExpr>),
    DataValue(Box<NirExpr>),
    DataCopyWindow {
        input: Box<NirExpr>,
        offset: Box<NirExpr>,
        len: Box<NirExpr>,
    },
    DataReadWindow {
        window: Box<NirExpr>,
        index: Box<NirExpr>,
    },
    DataWriteWindow {
        window: Box<NirExpr>,
        index: Box<NirExpr>,
        value: Box<NirExpr>,
    },
    DataFreezeWindow(Box<NirExpr>),
    DataImmutableWindow {
        input: Box<NirExpr>,
        offset: Box<NirExpr>,
        len: Box<NirExpr>,
    },
    DataHandleTable(Vec<(String, String)>),
    CpuBindCore(i64),
    CpuWindow {
        width: i64,
        height: i64,
        title: String,
    },
    CpuInputI64 {
        channel: String,
        default: i64,
        min: Option<i64>,
        max: Option<i64>,
        step: Option<i64>,
    },
    CpuTickI64 {
        start: i64,
        step: i64,
    },
    CpuSpawn {
        callee: String,
        args: Vec<NirExpr>,
    },
    CpuJoin(Box<NirExpr>),
    CpuCancel(Box<NirExpr>),
    CpuJoinResult(Box<NirExpr>),
    CpuTaskCompleted(Box<NirExpr>),
    CpuTaskTimedOut(Box<NirExpr>),
    CpuTaskCancelled(Box<NirExpr>),
    CpuTaskValue(Box<NirExpr>),
    CpuTimeout {
        task: Box<NirExpr>,
        limit: Box<NirExpr>,
    },
    CpuPresentFrame(Box<NirExpr>),
    ShaderProfileTargetRef {
        unit: String,
    },
    ShaderProfileViewportRef {
        unit: String,
    },
    ShaderProfilePipelineRef {
        unit: String,
    },
    ShaderProfileVertexCountRef {
        unit: String,
    },
    ShaderProfileInstanceCountRef {
        unit: String,
    },
    ShaderProfilePacketColorSlotRef {
        unit: String,
    },
    ShaderProfilePacketSpeedSlotRef {
        unit: String,
    },
    ShaderProfilePacketRadiusSlotRef {
        unit: String,
    },
    ShaderProfilePacketTagRef {
        unit: String,
    },
    ShaderProfileMaterialModeRef {
        unit: String,
    },
    ShaderProfilePassKindRef {
        unit: String,
    },
    ShaderProfilePacketFieldCountRef {
        unit: String,
    },
    ShaderProfileColorSeed {
        unit: String,
        base: Box<NirExpr>,
        delta: Box<NirExpr>,
    },
    ShaderProfileSpeedSeed {
        unit: String,
        delta: Box<NirExpr>,
        scale: Box<NirExpr>,
        base: Box<NirExpr>,
    },
    ShaderProfileRadiusSeed {
        unit: String,
        base: Box<NirExpr>,
        delta: Box<NirExpr>,
    },
    ShaderProfilePacket {
        unit: String,
        packet_type_name: Option<String>,
        color: Box<NirExpr>,
        speed: Box<NirExpr>,
        radius: Box<NirExpr>,
        accent: Option<Box<NirExpr>>,
        toggle_state: Option<Box<NirExpr>>,
        focus_index: Option<Box<NirExpr>>,
    },
    DataProfileBindCoreRef {
        unit: String,
    },
    DataProfileWindowOffsetRef {
        unit: String,
    },
    DataProfileUplinkLenRef {
        unit: String,
    },
    DataProfileDownlinkLenRef {
        unit: String,
    },
    DataProfileHandleTableRef {
        unit: String,
    },
    DataProfileMarkerRef {
        unit: String,
        tag: String,
    },
    NetworkProfileBindCoreRef {
        unit: String,
    },
    NetworkProfileEndpointKindRef {
        unit: String,
    },
    NetworkProfileTransportFamilyRef {
        unit: String,
    },
    NetworkProfileLocalPortRef {
        unit: String,
    },
    NetworkProfileRemotePortRef {
        unit: String,
    },
    NetworkProfileConnectTimeoutRef {
        unit: String,
    },
    NetworkProfileReadTimeoutRef {
        unit: String,
    },
    NetworkProfileWriteTimeoutRef {
        unit: String,
    },
    NetworkProfileTimeoutBudgetRef {
        unit: String,
    },
    NetworkProfileRetryBudgetRef {
        unit: String,
    },
    NetworkProfileStreamWindowRef {
        unit: String,
    },
    NetworkProfileRecvWindowRef {
        unit: String,
    },
    NetworkProfileSendWindowRef {
        unit: String,
    },
    NetworkProfileProtocolKindRef {
        unit: String,
    },
    NetworkProfileProtocolVersionRef {
        unit: String,
    },
    NetworkProfileProtocolHeaderBytesRef {
        unit: String,
    },
    NetworkResult {
        value: Box<NirExpr>,
        state: NirNetworkFlowState,
    },
    NetworkConfigReady(Box<NirExpr>),
    NetworkSendReady(Box<NirExpr>),
    NetworkRecvReady(Box<NirExpr>),
    NetworkAcceptReady(Box<NirExpr>),
    NetworkValue(Box<NirExpr>),
    KernelProfileBindCoreRef {
        unit: String,
    },
    KernelProfileQueueDepthRef {
        unit: String,
    },
    KernelProfileBatchLanesRef {
        unit: String,
    },
    KernelResult {
        value: Box<NirExpr>,
        state: NirKernelFlowState,
    },
    KernelConfigReady(Box<NirExpr>),
    KernelValue(Box<NirExpr>),
    KernelTensor {
        rows: i64,
        cols: i64,
        elements_csv: String,
    },
    KernelShape(Box<NirExpr>),
    KernelRows(Box<NirExpr>),
    KernelCols(Box<NirExpr>),
    KernelRow(Box<NirExpr>),
    KernelCol(Box<NirExpr>),
    KernelElementAt {
        input: Box<NirExpr>,
        row: Box<NirExpr>,
        col: Box<NirExpr>,
    },
    KernelReshape {
        input: Box<NirExpr>,
        rows: i64,
        cols: i64,
    },
    KernelBroadcast {
        input: Box<NirExpr>,
        rows: i64,
        cols: i64,
    },
    KernelMap {
        input: Box<NirExpr>,
        op: NirKernelMapOp,
        scalar: Option<Box<NirExpr>>,
    },
    KernelMapAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
        op: NirKernelMapOp,
        scalar: Option<Box<NirExpr>>,
    },
    KernelZip {
        lhs: Box<NirExpr>,
        rhs: Box<NirExpr>,
        op: NirKernelZipOp,
    },
    KernelMatmul {
        lhs: Box<NirExpr>,
        rhs: Box<NirExpr>,
    },
    KernelAddBias {
        input: Box<NirExpr>,
        bias: Box<NirExpr>,
    },
    KernelRelu(Box<NirExpr>),
    KernelReduceSum(Box<NirExpr>),
    KernelReduceSumAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelReduceMax(Box<NirExpr>),
    KernelReduceMaxAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelReduceMean(Box<NirExpr>),
    KernelReduceMeanAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelArgmax(Box<NirExpr>),
    KernelArgmaxAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelArgmin(Box<NirExpr>),
    KernelArgminAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelSort(Box<NirExpr>),
    KernelSortAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
    },
    KernelTopk {
        input: Box<NirExpr>,
        k: i64,
    },
    KernelTopkAxis {
        input: Box<NirExpr>,
        axis: NirKernelAxis,
        k: i64,
    },
    DataProfileSendUplink {
        unit: String,
        input: Box<NirExpr>,
    },
    DataProfileSendDownlink {
        unit: String,
        input: Box<NirExpr>,
    },
    ShaderTarget {
        format: String,
        width: i64,
        height: i64,
    },
    ShaderViewport {
        width: i64,
        height: i64,
    },
    ShaderPipeline {
        name: String,
        topology: String,
    },
    ShaderInlineWgsl {
        entry: String,
        source: String,
    },
    ShaderResult {
        value: Box<NirExpr>,
        state: NirShaderFlowState,
    },
    ShaderPassReady(Box<NirExpr>),
    ShaderFrameReady(Box<NirExpr>),
    ShaderValue(Box<NirExpr>),
    ShaderBeginPass {
        target: Box<NirExpr>,
        pipeline: Box<NirExpr>,
        viewport: Box<NirExpr>,
    },
    ShaderDrawInstanced {
        pass: Box<NirExpr>,
        packet: Box<NirExpr>,
        vertex_count: Box<NirExpr>,
        instance_count: Box<NirExpr>,
    },
    ShaderProfileRender {
        unit: String,
        packet: Box<NirExpr>,
    },
    CpuExternCall {
        abi: String,
        interface: Option<String>,
        callee: String,
        args: Vec<NirExpr>,
    },
    CastI64ToI32(Box<NirExpr>),
    Free(Box<NirExpr>),
    IsNull(Box<NirExpr>),
    Call {
        callee: String,
        args: Vec<NirExpr>,
    },
    MethodCall {
        receiver: Box<NirExpr>,
        method: String,
        args: Vec<NirExpr>,
    },
    StructLiteral {
        type_name: String,
        fields: Vec<(String, NirExpr)>,
    },
    FieldAccess {
        base: Box<NirExpr>,
        field: String,
    },
    Binary {
        op: NirBinaryOp,
        lhs: Box<NirExpr>,
        rhs: Box<NirExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirBinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirDataFlowState {
    Ready,
    Moved,
    Windowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirResultStage {
    Data(NirDataFlowState),
    Shader(NirShaderFlowState),
    Kernel(NirKernelFlowState),
    Network(NirNetworkFlowState),
}

impl NirResultStage {
    pub fn family(self) -> NirResultFamily {
        match self {
            Self::Data(_) => NirResultFamily::Data,
            Self::Shader(_) => NirResultFamily::Shader,
            Self::Kernel(_) => NirResultFamily::Kernel,
            Self::Network(_) => NirResultFamily::Network,
        }
    }

    pub fn render(self) -> &'static str {
        match self {
            Self::Data(state) => state.render(),
            Self::Shader(state) => state.render(),
            Self::Kernel(state) => state.render(),
            Self::Network(state) => state.render(),
        }
    }

    pub fn validate_payload(self, payload: &NirTypeRef) -> Result<(), String> {
        match self {
            Self::Data(state) => match state {
                NirDataFlowState::Ready => {
                    if matches!(
                        payload.container_kind(),
                        Some(NirContainerKind::Pipe | NirContainerKind::Window)
                    ) {
                        return Err(format!(
                            "`data_result(...)->{}` cannot carry staged container payload `{}`",
                            self.render(),
                            payload.render()
                        ));
                    }
                    Ok(())
                }
                NirDataFlowState::Moved => {
                    if payload.container_kind() != Some(NirContainerKind::Pipe) {
                        return Err(format!(
                            "`data_result(...)->{}` expects `Pipe<...>` payload, found `{}`",
                            self.render(),
                            payload.render()
                        ));
                    }
                    Ok(())
                }
                NirDataFlowState::Windowed => {
                    if payload.container_kind() != Some(NirContainerKind::Window) {
                        return Err(format!(
                            "`data_result(...)->{}` expects `Window<...>` payload, found `{}`",
                            self.render(),
                            payload.render()
                        ));
                    }
                    Ok(())
                }
            },
            Self::Shader(state) => {
                let expected = match state {
                    NirShaderFlowState::PassReady => "Pass",
                    NirShaderFlowState::FrameReady => "Frame",
                };
                if payload.is_ref
                    || payload.is_optional
                    || !payload.generic_args.is_empty()
                    || payload.name != expected
                {
                    return Err(format!(
                        "`shader_result(...)->{}` expects `{expected}` payload, found `{}`",
                        self.render(),
                        payload.render()
                    ));
                }
                Ok(())
            }
            Self::Kernel(state) => match state {
                NirKernelFlowState::ConfigReady => {
                    if !payload.is_integer_scalar() {
                        return Err(format!(
                            "`kernel_result(...)->{}` expects integer scalar payload, found `{}`",
                            self.render(),
                            payload.render()
                        ));
                    }
                    Ok(())
                }
            },
            Self::Network(state) => match state {
                NirNetworkFlowState::ConfigReady
                | NirNetworkFlowState::SendReady
                | NirNetworkFlowState::RecvReady
                | NirNetworkFlowState::AcceptReady
                | NirNetworkFlowState::Closed => {
                    if !payload.is_integer_scalar() {
                        return Err(format!(
                            "`network_result(...)->{}` expects integer scalar payload, found `{}`",
                            self.render(),
                            payload.render()
                        ));
                    }
                    Ok(())
                }
            },
        }
    }
}

impl NirDataFlowState {
    pub fn render(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Moved => "moved",
            Self::Windowed => "windowed",
        }
    }

    pub fn validate_payload(self, payload: &NirTypeRef) -> Result<(), String> {
        NirResultStage::from(self).validate_payload(payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirShaderFlowState {
    PassReady,
    FrameReady,
}

impl NirShaderFlowState {
    pub fn render(self) -> &'static str {
        match self {
            Self::PassReady => "pass_ready",
            Self::FrameReady => "frame_ready",
        }
    }

    pub fn validate_payload(self, payload: &NirTypeRef) -> Result<(), String> {
        NirResultStage::from(self).validate_payload(payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirKernelFlowState {
    ConfigReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirNetworkFlowState {
    ConfigReady,
    SendReady,
    RecvReady,
    AcceptReady,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirKernelMapOp {
    Relu,
    AddScalar,
    MulScalar,
}

impl NirKernelMapOp {
    pub fn instruction(&self) -> &'static str {
        match self {
            Self::Relu => "relu",
            Self::AddScalar => "add_scalar",
            Self::MulScalar => "mul_scalar",
        }
    }

    pub fn render(&self) -> &'static str {
        match self {
            Self::Relu => "relu",
            Self::AddScalar => "add_scalar",
            Self::MulScalar => "mul_scalar",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirKernelZipOp {
    Add,
    Mul,
}

impl NirKernelZipOp {
    pub fn instruction(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Mul => "mul",
        }
    }

    pub fn render(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Mul => "mul",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirKernelAxis {
    Rows,
    Cols,
}

impl NirKernelAxis {
    pub fn render(&self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::Cols => "cols",
        }
    }
}

impl NirKernelFlowState {
    pub fn render(self) -> &'static str {
        match self {
            Self::ConfigReady => "config_ready",
        }
    }

    pub fn validate_payload(self, payload: &NirTypeRef) -> Result<(), String> {
        NirResultStage::from(self).validate_payload(payload)
    }
}

impl NirNetworkFlowState {
    pub fn render(self) -> &'static str {
        match self {
            Self::ConfigReady => "config_ready",
            Self::SendReady => "send_ready",
            Self::RecvReady => "recv_ready",
            Self::AcceptReady => "accept_ready",
            Self::Closed => "closed",
        }
    }

    pub fn validate_payload(self, payload: &NirTypeRef) -> Result<(), String> {
        NirResultStage::from(self).validate_payload(payload)
    }
}

impl From<NirDataFlowState> for NirResultStage {
    fn from(value: NirDataFlowState) -> Self {
        Self::Data(value)
    }
}

impl From<NirShaderFlowState> for NirResultStage {
    fn from(value: NirShaderFlowState) -> Self {
        Self::Shader(value)
    }
}

impl From<NirKernelFlowState> for NirResultStage {
    fn from(value: NirKernelFlowState) -> Self {
        Self::Kernel(value)
    }
}

impl From<NirNetworkFlowState> for NirResultStage {
    fn from(value: NirNetworkFlowState) -> Self {
        Self::Network(value)
    }
}

pub fn nir_glm_profile(expr: &NirExpr) -> Option<NirGlmProfile> {
    match expr {
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::CastI64ToI32(_)
        | NirExpr::Var(_)
        | NirExpr::Await(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::Call { .. }
        | NirExpr::MethodCall { .. }
        | NirExpr::StructLiteral { .. }
        | NirExpr::FieldAccess { .. }
        | NirExpr::Binary { .. }
        | NirExpr::IsNull(_) => None,
        NirExpr::Borrow(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::BorrowEnd(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::Move(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::AllocNode { .. } | NirExpr::AllocBuffer { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataResult { .. }
        | NirExpr::DataReady(_)
        | NirExpr::DataMoved(_)
        | NirExpr::DataWindowed(_)
        | NirExpr::DataValue(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::CpuSpawn { .. }
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuPresentFrame(_)
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::ShaderProfileColorSeed { .. }
        | NirExpr::ShaderProfileSpeedSeed { .. }
        | NirExpr::ShaderProfileRadiusSeed { .. }
        | NirExpr::ShaderProfilePacket { .. }
        | NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::NetworkResult { .. }
        | NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_)
        | NirExpr::NetworkValue(_)
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelResult { .. }
        | NirExpr::KernelConfigReady(_)
        | NirExpr::KernelValue(_)
        | NirExpr::KernelTensor { .. }
        | NirExpr::KernelShape(_)
        | NirExpr::KernelRows(_)
        | NirExpr::KernelCols(_)
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelElementAt { .. }
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_)
        | NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgmin(_)
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::ShaderResult { .. }
        | NirExpr::ShaderPassReady(_)
        | NirExpr::ShaderFrameReady(_)
        | NirExpr::ShaderValue(_)
        | NirExpr::ShaderBeginPass { .. }
        | NirExpr::ShaderDrawInstanced { .. }
        | NirExpr::ShaderProfileRender { .. } => None,
        NirExpr::DataOutputPipe(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::DataCopyWindow { .. }
        | NirExpr::DataReadWindow { .. }
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataFreezeWindow(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataInputPipe(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::LoadValue(_)
        | NirExpr::LoadNext(_)
        | NirExpr::BufferLen(_)
        | NirExpr::LoadAt { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::StoreValue { .. } | NirExpr::StoreNext { .. } | NirExpr::StoreAt { .. } => {
            Some(NirGlmProfile {
                result_class: NirGlmValueClass::Val,
                accesses: vec![NirGlmAccess {
                    class: NirGlmValueClass::Res,
                    mode: NirGlmUseMode::Write,
                }],
                effect: NirGlmEffect::None,
            })
        }
        NirExpr::Free(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::LifetimeEnd,
        }),
    }
}

pub fn nir_expr_effect_class(expr: &NirExpr) -> NirExprEffectClass {
    match expr {
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
        | NirExpr::CastI64ToI32(_)
        | NirExpr::StructLiteral { .. }
        | NirExpr::FieldAccess { .. }
        | NirExpr::Binary { .. }
        | NirExpr::IsNull(_) => NirExprEffectClass::Pure,
        NirExpr::Borrow(_)
        | NirExpr::BorrowEnd(_)
        | NirExpr::LoadValue(_)
        | NirExpr::LoadNext(_)
        | NirExpr::BufferLen(_)
        | NirExpr::LoadAt { .. } => NirExprEffectClass::LocalReadOnly,
        NirExpr::Await(_) => NirExprEffectClass::AsyncOpaque,
        NirExpr::Call { .. } | NirExpr::MethodCall { .. } => NirExprEffectClass::CallOpaque,
        NirExpr::Instantiate { .. } => NirExprEffectClass::DomainOpaque,
        NirExpr::CpuBindCore(_)
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. } => NirExprEffectClass::HostReadOnly,
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataResult { .. }
        | NirExpr::DataReady(_)
        | NirExpr::DataMoved(_)
        | NirExpr::DataWindowed(_)
        | NirExpr::DataValue(_)
        | NirExpr::DataCopyWindow { .. }
        | NirExpr::DataReadWindow { .. }
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataInputPipe(_)
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::ShaderProfileColorSeed { .. }
        | NirExpr::ShaderProfileSpeedSeed { .. }
        | NirExpr::ShaderProfileRadiusSeed { .. }
        | NirExpr::ShaderProfilePacket { .. }
        | NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::NetworkResult { .. }
        | NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_)
        | NirExpr::NetworkValue(_)
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelResult { .. }
        | NirExpr::KernelConfigReady(_)
        | NirExpr::KernelValue(_)
        | NirExpr::KernelTensor { .. }
        | NirExpr::KernelShape(_)
        | NirExpr::KernelRows(_)
        | NirExpr::KernelCols(_)
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelElementAt { .. }
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_)
        | NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgmin(_)
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. }
        | NirExpr::ShaderResult { .. } => NirExprEffectClass::DomainReadOnly,
        NirExpr::CpuWindow { .. }
        | NirExpr::CpuSpawn { .. }
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuPresentFrame(_)
        | NirExpr::ShaderPassReady(_)
        | NirExpr::ShaderFrameReady(_)
        | NirExpr::ShaderValue(_)
        | NirExpr::ShaderBeginPass { .. }
        | NirExpr::ShaderDrawInstanced { .. }
        | NirExpr::ShaderProfileRender { .. } => NirExprEffectClass::Stateful,
        NirExpr::Move(_)
        | NirExpr::AllocNode { .. }
        | NirExpr::AllocBuffer { .. }
        | NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::DataOutputPipe(_)
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::Free(_) => NirExprEffectClass::Stateful,
    }
}

pub fn nir_host_read_surface(expr: &NirExpr) -> Option<NirHostReadSurface> {
    match expr {
        NirExpr::CpuBindCore(_) => Some(NirHostReadSurface::SchedulerLane),
        NirExpr::CpuInputI64 { .. } => Some(NirHostReadSurface::InputChannel),
        NirExpr::CpuTickI64 { .. } => Some(NirHostReadSurface::ClockTick),
        NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. } => Some(NirHostReadSurface::RenderDescriptor),
        _ => None,
    }
}

pub fn nir_host_scheduler_bridge(expr: &NirExpr) -> Option<NirHostSchedulerBridge> {
    match expr {
        NirExpr::CpuBindCore(lane) => Some(NirHostSchedulerBridge::from_cpu_bind_core(*lane)),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirNode {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricPrimitive {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackage {
    pub package_id: String,
    pub domain_family: String,
    pub entry_crate: String,
    pub ops: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        nir_expr_effect_class, nir_host_read_surface, nir_host_scheduler_bridge, NirContainerKind,
        NirDataFlowState, NirExpr, NirExprEffectClass, NirHostReadSurface, NirHostSchedulerBridge,
        NirHostSchedulerBridgeKind, NirHostTimingBridge, NirKernelFlowState, NirResultFamily,
        NirResultStage, NirShaderFlowState, NirTypeRef, NirWindowMode, TestClockDomain,
    };

    fn named(name: &str) -> NirTypeRef {
        NirTypeRef {
            name: name.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    fn generic(name: &str, arg: NirTypeRef) -> NirTypeRef {
        NirTypeRef {
            name: name.to_owned(),
            generic_args: vec![arg],
            is_optional: false,
            is_ref: false,
        }
    }

    #[test]
    fn rejects_moved_data_state_with_non_pipe_payload() {
        let error = NirDataFlowState::Moved
            .validate_payload(&named("i64"))
            .unwrap_err();
        assert!(error.contains("moved"));
        assert!(error.contains("Pipe<...>"));
    }

    #[test]
    fn result_stage_reports_owning_family() {
        assert_eq!(
            NirResultStage::from(NirDataFlowState::Windowed).family(),
            NirResultFamily::Data
        );
        assert_eq!(
            NirResultStage::from(NirShaderFlowState::FrameReady).family(),
            NirResultFamily::Shader
        );
        assert_eq!(
            NirResultStage::from(NirKernelFlowState::ConfigReady).family(),
            NirResultFamily::Kernel
        );
        assert!(NirResultFamily::Data.supports_stage(NirDataFlowState::Ready.into()));
        assert!(!NirResultFamily::Data.supports_stage(NirShaderFlowState::PassReady.into()));
    }

    #[test]
    fn rejects_windowed_data_state_with_non_window_payload() {
        let error = NirDataFlowState::Windowed
            .validate_payload(&generic("Pipe", named("i64")))
            .unwrap_err();
        assert!(error.contains("windowed"));
        assert!(error.contains("Window<...>"));
    }

    #[test]
    fn tracks_window_mutability_in_type_metadata() {
        let immutable = generic("Window", named("i64"));
        let mutable = generic("WindowMut", named("i64"));

        assert_eq!(immutable.window_mode(), Some(NirWindowMode::Immutable));
        assert_eq!(mutable.window_mode(), Some(NirWindowMode::Mutable));
        assert_eq!(immutable.container_kind(), Some(NirContainerKind::Window));
        assert_eq!(mutable.container_kind(), Some(NirContainerKind::Window));
        immutable.validate_container_contract().unwrap();
        mutable.validate_container_contract().unwrap();
    }

    #[test]
    fn rejects_pass_ready_shader_state_with_non_pass_payload() {
        let error = NirShaderFlowState::PassReady
            .validate_payload(&named("Frame"))
            .unwrap_err();
        assert!(error.contains("pass_ready"));
        assert!(error.contains("Pass"));
    }

    #[test]
    fn rejects_kernel_config_ready_with_non_integer_payload() {
        let error = NirKernelFlowState::ConfigReady
            .validate_payload(&named("bool"))
            .unwrap_err();
        assert!(error.contains("config_ready"));
        assert!(error.contains("integer scalar"));
    }

    #[test]
    fn classifies_scalar_binary_as_pure() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::Binary {
                op: super::NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Int(2)),
                rhs: Box::new(NirExpr::Int(3)),
            }),
            NirExprEffectClass::Pure
        );
    }

    #[test]
    fn classifies_borrow_as_read_only() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned())))),
            NirExprEffectClass::LocalReadOnly
        );
    }

    #[test]
    fn classifies_profile_ref_as_domain_read_only() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::ShaderProfileVertexCountRef {
                unit: "Main".to_owned(),
            }),
            NirExprEffectClass::DomainReadOnly
        );
    }

    #[test]
    fn classifies_host_tick_as_host_read_only() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::CpuTickI64 { start: 0, step: 1 }),
            NirExprEffectClass::HostReadOnly
        );
        assert_eq!(
            nir_host_read_surface(&NirExpr::CpuTickI64 { start: 0, step: 1 }),
            Some(NirHostReadSurface::ClockTick)
        );
    }

    #[test]
    fn reports_render_descriptor_host_surface() {
        assert_eq!(
            nir_host_read_surface(&NirExpr::ShaderViewport {
                width: 640,
                height: 360,
            }),
            Some(NirHostReadSurface::RenderDescriptor)
        );
    }

    #[test]
    fn reports_scheduler_and_input_host_surfaces() {
        assert_eq!(
            nir_host_read_surface(&NirExpr::CpuBindCore(0)),
            Some(NirHostReadSurface::SchedulerLane)
        );
        assert_eq!(
            nir_host_read_surface(&NirExpr::CpuInputI64 {
                channel: "speed".to_owned(),
                default: 4,
                min: None,
                max: None,
                step: None,
            }),
            Some(NirHostReadSurface::InputChannel)
        );
    }

    #[test]
    fn resolves_host_main_scheduler_lane_bridge() {
        let bridge = nir_host_scheduler_bridge(&NirExpr::CpuBindCore(0))
            .expect("cpu.bind_core(0) should expose a scheduler bridge");
        assert_eq!(
            bridge,
            NirHostSchedulerBridge {
                kind: NirHostSchedulerBridgeKind::HostMainLane,
                lane: 0,
            }
        );
        assert_eq!(bridge.as_str(), "host_main_lane");
        assert_eq!(bridge.resolved_source(), "cpu_bind_core_lane");
        assert_eq!(bridge.host_surface(), NirHostReadSurface::SchedulerLane);
    }

    #[test]
    fn resolves_worker_scheduler_lane_bridge() {
        let bridge = nir_host_scheduler_bridge(&NirExpr::CpuBindCore(3))
            .expect("cpu.bind_core(3) should expose a scheduler bridge");
        assert_eq!(
            bridge,
            NirHostSchedulerBridge {
                kind: NirHostSchedulerBridgeKind::WorkerLane,
                lane: 3,
            }
        );
        assert_eq!(bridge.as_str(), "worker_lane");
        assert_eq!(bridge.resolved_source(), "cpu_bind_core_lane");
        assert_eq!(bridge.host_surface(), NirHostReadSurface::SchedulerLane);
    }

    #[test]
    fn resolves_global_timing_bridge_to_monotonic_tick() {
        let bridge = NirHostTimingBridge::from_test_clock_domain(TestClockDomain::Global);
        assert_eq!(bridge, NirHostTimingBridge::GlobalToMonotonicTickBridge);
        assert_eq!(bridge.resolved_domain(), TestClockDomain::Monotonic);
        assert_eq!(bridge.resolved_source(), "host_monotonic_deadline");
        assert_eq!(bridge.host_surface(), NirHostReadSurface::ClockTick);
        assert_eq!(bridge.as_str(), "global_to_monotonic_tick_bridge");
    }

    #[test]
    fn resolves_wall_timing_bridge_to_wall_deadline() {
        let bridge = NirHostTimingBridge::from_test_clock_domain(TestClockDomain::Wall);
        assert_eq!(bridge, NirHostTimingBridge::WallDeadline);
        assert_eq!(bridge.resolved_domain(), TestClockDomain::Wall);
        assert_eq!(bridge.resolved_source(), "host_wall_deadline");
        assert_eq!(bridge.host_surface(), NirHostReadSurface::ClockTick);
        assert_eq!(bridge.as_str(), "wall_deadline");
    }

    #[test]
    fn classifies_call_as_opaque() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::Call {
                callee: "compute".to_owned(),
                args: vec![],
            }),
            NirExprEffectClass::CallOpaque
        );
    }

    #[test]
    fn classifies_await_as_async_opaque() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::Await(Box::new(NirExpr::Var("task".to_owned())))),
            NirExprEffectClass::AsyncOpaque
        );
    }

    #[test]
    fn classifies_instantiate_as_domain_opaque() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::Instantiate {
                domain: "data".to_owned(),
                unit: "Pipe".to_owned(),
            }),
            NirExprEffectClass::DomainOpaque
        );
    }

    #[test]
    fn classifies_extern_call_as_stateful() {
        assert_eq!(
            nir_expr_effect_class(&NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_side_effect".to_owned(),
                args: vec![],
            }),
            NirExprEffectClass::Stateful
        );
    }
}
