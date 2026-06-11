use super::*;

#[derive(Clone, Copy)]
pub(super) enum ResultLoweringDomain {
    Data,
    Shader,
    Kernel,
    Network,
}

impl ResultLoweringDomain {
    pub(super) fn module_name(self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Shader => "shader",
            Self::Kernel => "kernel",
            Self::Network => "network",
        }
    }

    pub(super) fn resource_name(self) -> &'static str {
        match self {
            Self::Data => "fabric0",
            Self::Shader => "shader0",
            Self::Kernel => "kernel0",
            Self::Network => "network0",
        }
    }

    pub(super) fn ensure_resource(self, yir: &mut YirModule) {
        match self {
            Self::Data => ensure_fabric_resource(yir),
            Self::Shader => ensure_shader_resource(yir),
            Self::Kernel => ensure_kernel_resource(yir),
            Self::Network => ensure_network_resource(yir),
        }
    }
}

pub(super) struct LoweringState<'a> {
    pub(super) yir: &'a mut YirModule,
    pub(super) function_map: BTreeMap<&'a str, &'a NirFunction>,
    pub(super) direct_call_functions: BTreeSet<String>,
    pub(super) async_helper_functions: BTreeSet<String>,
    pub(super) pure_helpers: BTreeSet<String>,
    pub(super) value_counter: usize,
    pub(super) print_counter: usize,
    pub(super) await_counter: usize,
    pub(super) call_stack: Vec<String>,
    pub(super) last_effect_anchor: Option<String>,
}

pub(super) fn next_name(state: &mut LoweringState<'_>, prefix: &str) -> String {
    let name = format!("{prefix}_{}", state.value_counter);
    state.value_counter += 1;
    name
}
