use std::fmt;

use super::*;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::I32(value) => write!(f, "{value}i32"),
            Self::Int(value) => write!(f, "{value}"),
            Self::F32(value) => write!(f, "{}f32", trim_float(*value as f64)),
            Self::F64(value) => write!(f, "{}f64", trim_float(*value)),
            Self::Symbol(value) => write!(f, "{value}"),
            Self::Tensor(tensor) => write!(f, "{tensor}"),
            Self::Pointer(pointer) => match pointer {
                Some(address) => write!(f, "&{address}"),
                None => write!(f, "null"),
            },
            Self::Tuple(values) => {
                write!(f, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Self::Struct(value) => write!(f, "{value}"),
            Self::VariantUnion(value) => write!(f, "{value}"),
            Self::DataWindow(window) => write!(f, "{window}"),
            Self::DataPipe(pipe) => write!(f, "{pipe}"),
            Self::DataResult(result) => write!(f, "{result}"),
            Self::DataMarker(marker) => write!(f, "{marker}"),
            Self::DataHandleTable(table) => write!(f, "{table}"),
            Self::DataCoreBinding(binding) => write!(f, "{binding}"),
            Self::ShaderResult(result) => write!(f, "{result}"),
            Self::KernelResult(result) => write!(f, "{result}"),
            Self::NetworkResult(result) => write!(f, "{result}"),
            Self::Target(target) => write!(f, "{target}"),
            Self::Viewport(viewport) => write!(f, "{viewport}"),
            Self::Pipeline(pipeline) => write!(f, "{pipeline}"),
            Self::VertexLayout(layout) => write!(f, "{layout}"),
            Self::VertexBuffer(buffer) => write!(f, "{buffer}"),
            Self::IndexBuffer(buffer) => write!(f, "{buffer}"),
            Self::Texture(texture) => write!(f, "{texture}"),
            Self::Sampler(sampler) => write!(f, "{sampler}"),
            Self::Blend(blend) => write!(f, "{blend}"),
            Self::Depth(depth) => write!(f, "{depth}"),
            Self::Raster(raster) => write!(f, "{raster}"),
            Self::RenderState(render_state) => write!(f, "{render_state}"),
            Self::Binding(binding) => write!(f, "{binding}"),
            Self::BindingSet(binding_set) => write!(f, "{binding_set}"),
            Self::RenderPass(pass) => write!(f, "{pass}"),
            Self::Frame(frame) => write!(f, "{frame}"),
            Self::Task(task) => match task.limit {
                Some(limit) => {
                    if matches!(task.state, TaskLifecycleState::Cancelled) {
                        write!(f, "task<{}; cancelled; limit={limit}>", task.label)
                    } else {
                        write!(f, "task<{}; limit={limit}>", task.label)
                    }
                }
                None if matches!(task.state, TaskLifecycleState::Cancelled) => {
                    write!(f, "task<cancelled; {}>", task.label)
                }
                None => write!(f, "task<{}>", task.label),
            },
            Self::Thread(thread) => write!(f, "thread<{}:{}>", thread.label, thread.state),
            Self::TaskResult(result) => write!(f, "task_result<{}:{}>", result.label, result.state),
            Self::Mutex(mutex) => write!(f, "mutex<{}>", mutex.label),
            Self::MutexGuard(guard) => write!(f, "mutex_guard<{}>", guard.label),
            Self::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for TaskLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("pending"),
            Self::Completed => f.write_str("completed"),
            Self::TimedOut => f.write_str("timed_out"),
            Self::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl Value {
    pub fn result_family(&self) -> Option<YirResultFamily> {
        match self {
            Self::TaskResult(_) => Some(YirResultFamily::Task),
            Self::DataResult(_) => Some(YirResultFamily::Data),
            Self::ShaderResult(_) => Some(YirResultFamily::Shader),
            Self::KernelResult(_) => Some(YirResultFamily::Kernel),
            Self::NetworkResult(_) => Some(YirResultFamily::Network),
            _ => None,
        }
    }

    pub fn result_state(&self) -> Option<YirResultState> {
        match self {
            Self::TaskResult(result) => Some(YirResultState::Task(result.state)),
            Self::DataResult(result) => Some(YirResultState::Data(result.state)),
            Self::ShaderResult(result) => Some(YirResultState::Shader(result.state)),
            Self::KernelResult(result) => Some(YirResultState::Kernel(result.state)),
            Self::NetworkResult(result) => Some(YirResultState::Network(result.state)),
            _ => None,
        }
    }

    pub fn result_payload(&self) -> Option<&Value> {
        match self {
            Self::TaskResult(result) => result.result.as_deref(),
            Self::DataResult(result) => Some(result.value.as_ref()),
            Self::ShaderResult(result) => Some(result.value.as_ref()),
            Self::KernelResult(result) => Some(result.value.as_ref()),
            Self::NetworkResult(result) => Some(result.value.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for DataResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for DataFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready => f.write_str("ready"),
            Self::Moved => f.write_str("moved"),
            Self::Windowed => f.write_str("windowed"),
        }
    }
}

impl fmt::Display for ShaderResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shader_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for ShaderFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PassReady => f.write_str("pass_ready"),
            Self::FrameReady => f.write_str("frame_ready"),
        }
    }
}

impl fmt::Display for KernelResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kernel_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for KernelFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigReady => f.write_str("config_ready"),
        }
    }
}

impl fmt::Display for NetworkResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "network_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for NetworkFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigReady => f.write_str("config_ready"),
            Self::SendReady => f.write_str("send_ready"),
            Self::RecvReady => f.write_str("recv_ready"),
            Self::ConnectReady => f.write_str("connect_ready"),
            Self::AcceptReady => f.write_str("accept_ready"),
            Self::Closed => f.write_str("closed"),
        }
    }
}

fn trim_float(value: f64) -> String {
    let mut rendered = value.to_string();
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.push('0');
        }
    }
    rendered
}

impl fmt::Display for TensorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tensor[{}x{} ", self.rows, self.cols)?;
        for row in 0..self.rows {
            if row > 0 {
                write!(f, " | ")?;
            }
            for col in 0..self.cols {
                if col > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", self.elements[row * self.cols + col])?;
            }
        }
        write!(f, "]")
    }
}

impl fmt::Display for StructValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{{", self.type_name)?;
        for (index, (name, value)) in self.fields.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {value}")?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for VariantUnionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}<active={}>[",
            self.parent_type_name, self.active_variant
        )?;
        for (index, variant) in self.variants.keys().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{variant}")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for DataWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.immutable { "immutable" } else { "copy" };
        write!(
            f,
            "window[{mode} offset={} len={} base={}]",
            self.offset, self.len, self.base
        )
    }
}

impl fmt::Display for DataPipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipe[{} {}]", self.direction, self.payload)
    }
}

impl fmt::Display for DataPipeDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
        }
    }
}

impl fmt::Display for DataMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "marker[{}]", self.tag)
    }
}

impl fmt::Display for DataHandleTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "handle_table[")?;
        for (index, (slot, resource)) in self.entries.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}={}", slot, resource)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for DataCoreBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "core_binding[core={}]", self.core_index)
    }
}

impl fmt::Display for SurfaceTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "target[{} {}x{}]", self.format, self.width, self.height)
    }
}

impl fmt::Display for Viewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "viewport[{}x{}]", self.width, self.height)
    }
}

impl fmt::Display for RenderPipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipeline[{} {}]", self.shading_model, self.topology)
    }
}

impl fmt::Display for VertexLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex_layout[stride={} attrs=", self.stride)?;
        for (index, attr) in self.attributes.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            write!(f, "{attr}")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for VertexBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex_buffer[count={}]", self.vertex_count)
    }
}

impl fmt::Display for IndexBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index_buffer[count={}]", self.indices.len())
    }
}

impl fmt::Display for Texture2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "texture2d[{} {}x{}]",
            self.format, self.width, self.height
        )
    }
}

impl fmt::Display for SamplerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sampler[{} {}]", self.filter, self.address_mode)
    }
}

impl fmt::Display for BlendState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blend[enabled={} mode={}]", self.enabled, self.mode)
    }
}

impl fmt::Display for DepthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "depth[test={} write={} compare={}]",
            self.test_enabled, self.write_enabled, self.compare
        )
    }
}

impl fmt::Display for RasterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "raster[cull={} front={}]",
            self.cull_mode, self.front_face
        )
    }
}

impl fmt::Display for RenderStateSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "render_state[pipeline={}, {}, {}, {}]",
            self.pipeline, self.blend, self.depth, self.raster
        )
    }
}

impl fmt::Display for ShaderBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}={}", self.kind, self.slot, self.value)
    }
}

impl fmt::Display for ShaderBindingSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bind_set[pipeline={}, bindings=", self.pipeline)?;
        for (index, binding) in self.bindings.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{binding}")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pass[target={}, pipeline={}, viewport={}]",
            self.target, self.pipeline, self.viewport
        )
    }
}

impl fmt::Display for FrameSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frame[{}x{}] ", self.width, self.height)?;
        for (index, row) in self.rows.iter().enumerate() {
            if index > 0 {
                write!(f, "|")?;
            }
            write!(f, "{row}")?;
        }
        Ok(())
    }
}
