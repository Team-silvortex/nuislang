mod ast;
mod expr;
mod expr_effects;
mod glm;
mod result_stage;
mod type_ref_methods;

pub use ast::*;
pub use expr::*;
pub use expr_effects::{nir_expr_effect_class, nir_host_read_surface, nir_host_scheduler_bridge};
pub use glm::nir_glm_profile;
pub use result_stage::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirUse {
    pub domain: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirModule {
    pub annotations: Vec<NirAnnotation>,
    pub uses: Vec<NirUse>,
    pub domain: String,
    pub unit: String,
    pub externs: Vec<NirExternFunction>,
    pub extern_interfaces: Vec<NirExternInterface>,
    pub consts: Vec<NirConstItem>,
    pub type_aliases: Vec<NirTypeAlias>,
    pub structs: Vec<NirStructDef>,
    pub enums: Vec<NirEnumDef>,
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
    pub where_bounds: Vec<NirWherePredicate>,
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
    pub generic_params: Vec<NirGenericParam>,
    pub where_bounds: Vec<NirWherePredicate>,
    pub fields: Vec<NirStructField>,
}

impl NirStructDef {
    pub fn field(&self, name: &str) -> Option<&NirStructField> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirEnumDef {
    pub visibility: NirVisibility,
    pub annotations: Vec<NirAnnotation>,
    pub name: String,
    pub generic_params: Vec<NirGenericParam>,
    pub where_bounds: Vec<NirWherePredicate>,
    pub variants: Vec<NirEnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirEnumVariant {
    pub name: String,
    pub kind: NirEnumVariantKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirEnumVariantKind {
    Unit,
    Tuple(Vec<NirTypeRef>),
    Struct(Vec<NirStructField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirGenericParam {
    pub name: String,
    pub bounds: Vec<NirTypeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirWherePredicate {
    pub param_name: String,
    pub bounds: Vec<NirTypeRef>,
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
pub enum NirAddressClass {
    Owned,
    Borrowed,
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
    pub benchmark_name: Option<String>,
    pub benchmark_warmup_iters: Option<i64>,
    pub benchmark_measure_iters: Option<i64>,
    pub benchmark_timeout_ms: Option<i64>,
    pub benchmark_clock_domain: Option<TestClockDomain>,
    pub benchmark_clock_policy: Option<TestClockPolicy>,
    pub is_async: bool,
    pub generic_params: Vec<NirGenericParam>,
    pub where_bounds: Vec<NirWherePredicate>,
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
mod tests;
