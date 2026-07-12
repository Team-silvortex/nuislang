use super::*;

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
pub enum NirShaderSampleMode {
    Dynamic,
    Nearest,
}

impl NirShaderSampleMode {
    pub fn render(self) -> &'static str {
        match self {
            Self::Dynamic => "sample",
            Self::Nearest => "sample_nearest",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NirShaderSampleUvMode {
    Dynamic,
    Nearest,
    Linear,
}

impl NirShaderSampleUvMode {
    pub fn render(self) -> &'static str {
        match self {
            Self::Dynamic => "sample_uv",
            Self::Nearest => "sample_uv_nearest",
            Self::Linear => "sample_uv_linear",
        }
    }
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
