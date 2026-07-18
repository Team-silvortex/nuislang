use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirExpr {
    Bool(bool),
    Text(String),
    Int(i64),
    F32(String),
    F64(String),
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
    CpuTaskFailed(Box<NirExpr>),
    CpuTaskValue(Box<NirExpr>),
    CpuThreadSpawn {
        callee: String,
        args: Vec<NirExpr>,
    },
    CpuThreadJoin(Box<NirExpr>),
    CpuThreadJoinResult(Box<NirExpr>),
    CpuMutexNew(Box<NirExpr>),
    CpuMutexLock(Box<NirExpr>),
    CpuMutexUnlock(Box<NirExpr>),
    CpuMutexValue(Box<NirExpr>),
    CpuTimeout {
        task: Box<NirExpr>,
        limit: Box<NirExpr>,
    },
    CpuReadyAfter {
        task: Box<NirExpr>,
        delay: Box<NirExpr>,
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
    ShaderProfileSliderColorSlotRef {
        unit: String,
    },
    ShaderProfileSliderSpeedSlotRef {
        unit: String,
    },
    ShaderProfileSliderRadiusSlotRef {
        unit: String,
    },
    ShaderProfileHeaderAccentSlotRef {
        unit: String,
    },
    ShaderProfileToggleLiveSlotRef {
        unit: String,
    },
    ShaderProfileFocusSlotRef {
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
    ShaderTexture2d {
        format: String,
        width: i64,
        height: i64,
        texels: String,
    },
    ShaderSampler {
        filter: String,
        address_mode: String,
    },
    ShaderUv {
        u: i64,
        v: i64,
    },
    ShaderSample {
        texture: Box<NirExpr>,
        sampler: Box<NirExpr>,
        x: Box<NirExpr>,
        y: Box<NirExpr>,
        mode: NirShaderSampleMode,
    },
    ShaderSampleUv {
        texture: Box<NirExpr>,
        sampler: Box<NirExpr>,
        uv: Box<NirExpr>,
        mode: NirShaderSampleUvMode,
    },
    ShaderBinding {
        kind: String,
        slot: i64,
        layout: Option<String>,
        profile_contract: Option<String>,
        value: Box<NirExpr>,
    },
    ShaderBindSet {
        pipeline: Box<NirExpr>,
        bindings: Vec<NirExpr>,
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
    CpuExternCallI32 {
        abi: String,
        interface: Option<String>,
        callee: String,
        args: Vec<NirExpr>,
    },
    HostBufferHandle(Box<NirExpr>),
    CastI64ToI32(Box<NirExpr>),
    CastI32ToI64(Box<NirExpr>),
    CastI64ToBool(Box<NirExpr>),
    CastBoolToI64(Box<NirExpr>),
    CastI64ToF32(Box<NirExpr>),
    CastF32ToI64(Box<NirExpr>),
    CastI64ToF64(Box<NirExpr>),
    CastF64ToI64(Box<NirExpr>),
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
        type_args: Vec<NirTypeRef>,
        fields: Vec<(String, NirExpr)>,
    },
    FieldAccess {
        base: Box<NirExpr>,
        field: String,
    },
    VariantIs {
        base: Box<NirExpr>,
        variant: String,
    },
    VariantFieldAccess {
        base: Box<NirExpr>,
        variant: String,
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
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
