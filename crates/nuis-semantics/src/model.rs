#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirIntent {
    pub op: String,
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
    pub structs: Vec<AstStructDef>,
    pub functions: Vec<AstFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExternFunction {
    pub abi: String,
    pub interface: Option<String>,
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_type: AstTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstExternInterface {
    pub abi: String,
    pub name: String,
    pub methods: Vec<AstExternFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructField {
    pub name: String,
    pub ty: AstTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstStructDef {
    pub name: String,
    pub fields: Vec<AstStructField>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstFunction {
    pub name: String,
    pub params: Vec<AstParam>,
    pub return_type: Option<AstTypeRef>,
    pub body: Vec<AstStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstStmt {
    Let {
        name: String,
        ty: Option<AstTypeRef>,
        value: AstExpr,
    },
    Const {
        name: String,
        ty: AstTypeRef,
        value: AstExpr,
    },
    Print(AstExpr),
    If {
        condition: AstExpr,
        then_body: Vec<AstStmt>,
        else_body: Vec<AstStmt>,
    },
    Expr(AstExpr),
    Return(Option<AstExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstExpr {
    Bool(bool),
    Text(String),
    Int(i64),
    Var(String),
    Instantiate {
        domain: String,
        unit: String,
    },
    Call {
        callee: String,
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
    Add,
    Sub,
    Mul,
    Div,
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
    pub structs: Vec<NirStructDef>,
    pub functions: Vec<NirFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirExternFunction {
    pub abi: String,
    pub interface: Option<String>,
    pub name: String,
    pub params: Vec<NirParam>,
    pub return_type: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirExternInterface {
    pub abi: String,
    pub name: String,
    pub methods: Vec<NirExternFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirStructField {
    pub name: String,
    pub ty: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirStructDef {
    pub name: String,
    pub fields: Vec<NirStructField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirTypeRef {
    pub name: String,
    pub generic_args: Vec<NirTypeRef>,
    pub is_optional: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirParam {
    pub name: String,
    pub ty: NirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirFunction {
    pub name: String,
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
    If {
        condition: NirExpr,
        then_body: Vec<NirStmt>,
        else_body: Vec<NirStmt>,
    },
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
    Instantiate {
        domain: String,
        unit: String,
    },
    Null,
    Borrow(Box<NirExpr>),
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
    DataCopyWindow {
        input: Box<NirExpr>,
        offset: Box<NirExpr>,
        len: Box<NirExpr>,
    },
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
        color: Box<NirExpr>,
        speed: Box<NirExpr>,
        radius: Box<NirExpr>,
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
    Add,
    Sub,
    Mul,
    Div,
}

pub fn nir_glm_profile(expr: &NirExpr) -> Option<NirGlmProfile> {
    match expr {
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
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
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
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
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
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
        NirExpr::DataCopyWindow { .. } | NirExpr::DataImmutableWindow { .. } => Some(NirGlmProfile {
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
        NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Write,
            }],
            effect: NirGlmEffect::None,
        }),
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
