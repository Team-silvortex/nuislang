use super::parse_nuis_module;
use nuis_semantics::model::{
    NirDataFlowState, NirExpr, NirKernelFlowState, NirShaderFlowState, NirStmt,
};

#[path = "tests_frontend_semantics/async_boundaries.rs"]
mod async_boundaries;
#[path = "tests_frontend_semantics/async_ffi_basics.rs"]
mod async_ffi_basics;
#[path = "tests_frontend_semantics/data_shader_helpers.rs"]
mod data_shader_helpers;
#[path = "tests_frontend_semantics/kernel_results.rs"]
mod kernel_results;
#[path = "tests_frontend_semantics/kernel_tensor_ops.rs"]
mod kernel_tensor_ops;
#[path = "tests_frontend_semantics/nova_controls.rs"]
mod nova_controls;
#[path = "tests_frontend_semantics/nova_frame_graph.rs"]
mod nova_frame_graph;
#[path = "tests_frontend_semantics/nova_panel_builder.rs"]
mod nova_panel_builder;
#[path = "tests_frontend_semantics/nova_scene_a.rs"]
mod nova_scene_a;
#[path = "tests_frontend_semantics/nova_scene_b.rs"]
mod nova_scene_b;
#[path = "tests_frontend_semantics/nova_selection_theme.rs"]
mod nova_selection_theme;
#[path = "tests_frontend_semantics/thread_mutex.rs"]
mod thread_mutex;
