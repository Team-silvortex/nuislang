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
    pub is_async: bool,
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
    Await(AstExpr),
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
    Await(Box<AstExpr>),
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

impl NirStructDef {
    pub fn field(&self, name: &str) -> Option<&NirStructField> {
        self.fields.iter().find(|field| field.name == name)
    }
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
            "Window" if !self.is_ref => Some(NirContainerKind::Window),
            "Pipe" if !self.is_ref => Some(NirContainerKind::Pipe),
            "Instance" if !self.is_ref => Some(NirContainerKind::Instance),
            _ => None,
        }
    }

    pub fn container_payload(&self) -> Option<&NirTypeRef> {
        if matches!(
            self.container_kind(),
            Some(NirContainerKind::Window | NirContainerKind::Pipe | NirContainerKind::Instance)
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
    pub name: String,
    pub is_async: bool,
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
    KernelProfileBindCoreRef {
        unit: String,
    },
    KernelProfileQueueDepthRef {
        unit: String,
    },
    KernelProfileBatchLanesRef {
        unit: String,
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
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
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
        NirExpr::DataCopyWindow { .. } | NirExpr::DataImmutableWindow { .. } => {
            Some(NirGlmProfile {
                result_class: NirGlmValueClass::Val,
                accesses: vec![NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                }],
                effect: NirGlmEffect::None,
            })
        }
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
