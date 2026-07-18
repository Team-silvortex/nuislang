use super::*;

#[test]
fn c_shim_source_includes_native_cli_runtime_hooks() {
    let ast = host_runtime_hooks_ast();
    let shim = c_shim_source(&ast);
    assert!(shim.contains("int main(int argc, char** argv)"));
    assert!(shim.contains("nuis_argc = argc;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_surface_slots = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_bootstrap_entry_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_tick_once_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_shutdown_v1(int64_t status)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_yalivia_rpc_hook_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_bridge_bind_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_task_poll_v1(void)"));
    assert!(shim.contains("static int64_t nuis_scheduler_task_states[256];"));
    assert!(shim.contains("typedef struct {\n    int64_t kind;"));
    assert!(shim
        .contains("static NuisSchedulerTaskThunkPacket nuis_scheduler_task_thunk_packets[256];"));
    assert!(shim.contains("static int64_t nuis_scheduler_task_execute_thunk_v1(int64_t index)"));
    assert!(!shim.contains("nuis_scheduler_task_thunks_zero_i64[256]"));
    assert!(!shim.contains("nuis_scheduler_task_thunks_binary_i64[256]"));
    assert!(shim.contains("nuis_scheduler_task_states[index] = 1;"));
    assert!(shim.contains("int64_t nuis_scheduler_task_spawn_i64_v1(int64_t payload)"));
    assert!(shim.contains("int64_t nuis_scheduler_task_spawn_invoker_i64_v1("));
    assert!(shim.contains("void nuis_scheduler_task_ready_after_v1("));
    assert!(shim.contains(
        "nuis_scheduler_task_ready_ticks[index] = nuis_lifecycle_state.tick_count + delay;"
    ));
    assert!(shim.contains("int64_t result = invoker(context);"));
    assert!(shim.contains("nuis_scheduler_task_release_context_v1(index);"));
    assert!(shim.contains("int64_t nuis_scheduler_task_join_state_v1(int64_t task_handle)"));
    assert!(shim.contains("int64_t nuis_scheduler_task_value_i64_v1(int64_t task_handle)"));
    assert!(shim.contains("typedef struct {\n    void* data;\n    int64_t size;"));
    assert!(shim.contains("NuisSchedulerPayloadMoveV1 move_hook;"));
    assert!(shim.contains("NuisSchedulerPayloadDropV1 drop_hook;"));
    assert!(shim.contains("static int nuis_scheduler_owned_payload_valid_v1("));
    assert!(shim.contains("int64_t nuis_scheduler_task_spawn_owned_v1("));
    assert!(shim.contains("int64_t nuis_scheduler_task_spawn_owned_invoker_v1("));
    assert!(shim.contains("nuis_scheduler_task_thunk_packets[index].kind = 2;"));
    assert!(shim.contains("owned_template.data = (void*)(uintptr_t)result;"));
    assert!(shim.contains("const NuisSchedulerOwnedPayloadV1* payload_ref"));
    assert!(shim.contains("int64_t nuis_scheduler_task_take_owned_v1("));
    assert!(shim.contains("void nuis_scheduler_owned_payload_drop_v1("));
    assert!(shim.contains("void nuis_scheduler_payload_free_v1(void* data)"));
    assert!(shim.contains("payload->drop_hook(payload->data);"));
    assert!(shim.contains("nuis_scheduler_task_release_owned_payload_v1(index);"));
    assert!(shim.contains("nuis_scheduler_task_payload_kinds[index] = 0;"));
    assert!(shim.contains("(void)nuis_lifecycle_tick_once_v1();"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_result_commit_v1(int64_t status)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_summary_flush_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_sample_network_bridge_progress_v1(void)"));
    assert!(
        shim.contains("static int64_t nuis_lifecycle_sample_hetero_submission_progress_v1(void)")
    );
    assert!(shim.contains("static int64_t nuis_lifecycle_on_network_bridge_progress_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_hetero_submission_progress_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_managed_rpc_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_shutdown_prepare_v1(int64_t status)"));
    assert!(shim.contains("int64_t nuis_lifecycle_bootstrap_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_bootstrap_entry_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_tick_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_tick_once_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_shutdown_export_v1(int64_t status) {"));
    assert!(shim.contains("return nuis_lifecycle_shutdown_v1(status);"));
    assert!(shim.contains("int64_t nuis_lifecycle_yalivia_rpc_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_yalivia_rpc_hook_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_network_bridge_progress_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_state.network_bridge_progress_count;"));
    assert!(shim.contains("int64_t nuis_lifecycle_hetero_submission_progress_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_state.hetero_submission_progress_count;"));
    assert!(shim.contains("if (nuis_lifecycle_bootstrap_entry_v1() != 0) {"));
    assert!(shim.contains("(void)nuis_lifecycle_on_bridge_bind_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_managed_rpc_v1();"));
    assert!(shim.contains("int64_t entry_status = nuis_yir_entry();"));
    assert!(shim.contains("(void)nuis_lifecycle_tick_once_v1();"));
    assert!(shim
        .contains("(void)nuis_lifecycle_on_scheduler_tick_v1(nuis_lifecycle_state.tick_count);"));
    assert!(shim.contains("(void)nuis_lifecycle_on_task_poll_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_network_bridge_progress_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_hetero_submission_progress_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_result_commit_v1(status);"));
    assert!(shim.contains("(void)nuis_lifecycle_on_summary_flush_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_shutdown_prepare_v1(status);"));
    assert!(shim.contains("return (int)nuis_lifecycle_shutdown_v1(entry_status);"));
    assert!(shim.contains("return nuis_host_argv_count();"));
    assert!(shim.contains("return nuis_host_cwd_handle();"));
    assert!(shim.contains("return nuis_host_monotonic_time_ns();"));
}
