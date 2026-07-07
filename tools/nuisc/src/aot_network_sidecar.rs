use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, render_target_specific_backend_fields,
    render_target_specific_lowering_fields,
};
use crate::aot_toml::escape_toml_string;
use std::fmt::Write as _;

pub(crate) fn render_domain_build_unit_network_ir_sidecar(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::with_capacity(4096);
    out.push_str("schema = \"nuis-network-ir-sidecar-v1\"\n");
    writeln!(
        out,
        "domain_family = \"{}\"",
        escape_toml_string(&unit.domain_family)
    )
    .unwrap();
    writeln!(
        out,
        "package_id = \"{}\"",
        escape_toml_string(&unit.package_id)
    )
    .unwrap();
    writeln!(
        out,
        "backend_family = \"{}\"",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    )
    .unwrap();
    writeln!(
        out,
        "selected_lowering_target = \"{}\"",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    )
    .unwrap();
    writeln!(
        out,
        "lowering_profile = \"{}\"",
        escape_toml_string(profile.profile_key)
    )
    .unwrap();
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    out.push_str("[lowering_capabilities]\n");
    out.push_str("binary_role = \"linker-input-sidecar\"\n");
    out.push_str("capability_owner = \"network-nustar\"\n");
    out.push_str("frontend_ir = \"nuis-yir.network\"\n");
    match profile.profile_key {
        "urlsession.socket-io" => {
            out.push_str("native_ir = \"foundation-url-request\"\n");
            out.push_str("transport_lowering = \"session-task-packet\"\n");
            out.push_str("resource_lowering = \"session-handle-table\"\n");
            out.push_str("dispatch_lowering = \"urlsession-task-submit\"\n");
            out.push_str("memory_lowering = \"managed-request-response-slots\"\n");
            out.push_str("result_lowering = \"completion-callback-response\"\n");
            out.push_str(
                "validation_contracts = [\"glm.network-handle-lifetime\", \"time.io-ready-order\", \"network.session-shape\"]\n",
            );
        }
        "winsock.socket-io" => {
            out.push_str("native_ir = \"winsock-overlapped\"\n");
            out.push_str("transport_lowering = \"overlapped-packet-reactor\"\n");
            out.push_str("resource_lowering = \"socket-completion-port-table\"\n");
            out.push_str("dispatch_lowering = \"winsock-overlapped-submit\"\n");
            out.push_str("memory_lowering = \"overlapped-packet-buffer\"\n");
            out.push_str("result_lowering = \"iocp-completion-response\"\n");
            out.push_str(
                "validation_contracts = [\"glm.network-handle-lifetime\", \"time.iocp-completion-order\", \"network.overlapped-shape\"]\n",
            );
        }
        "socket-abi.socket-io" => {
            out.push_str("native_ir = \"posix-socket\"\n");
            out.push_str("transport_lowering = \"packet-poll-reactor\"\n");
            out.push_str("resource_lowering = \"fd-handle-table\"\n");
            out.push_str("dispatch_lowering = \"poll-send-recv-submit\"\n");
            out.push_str("memory_lowering = \"borrowed-packet-buffer-window\"\n");
            out.push_str("result_lowering = \"poll-ready-response-token\"\n");
            out.push_str(
                "validation_contracts = [\"glm.fd-handle-lifetime\", \"time.poll-ready-order\", \"network.packet-shape\"]\n",
            );
        }
        _ => {
            out.push_str("native_ir = \"unknown\"\n");
            out.push_str("transport_lowering = \"unimplemented\"\n");
            out.push_str("resource_lowering = \"unimplemented\"\n");
            out.push_str("dispatch_lowering = \"unimplemented\"\n");
            out.push_str("memory_lowering = \"unimplemented\"\n");
            out.push_str("result_lowering = \"unimplemented\"\n");
            out.push_str("validation_contracts = [\"glm.network-handle-lifetime\"]\n");
        }
    }
    out.push_str("[session_shapes]\n");
    match profile.profile_key {
        "urlsession.socket-io" => {
            out.push_str("request = \"http-client-session\"\n");
            out.push_str("response = \"completion-callback\"\n");
            out.push_str("streaming = \"delegate-push-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"session.handle, request.packet, response.slot\"\n");
            out.push_str("argument_model = \"foundation-request-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"open_session\"\n");
            out.push_str("send = \"submit_request\"\n");
            out.push_str("recv = \"on_response\"\n");
            out.push_str("finalize = \"finish_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"fn open_session(authority: text) -> session { session(authority) }\"\n");
            out.push_str("send_body = \"fn submit_request(session: session, request: packet) -> task { task(session, request) }\"\n");
            out.push_str(
                "recv_body = \"fn on_response(task: task) -> response { response(task) }\"\n",
            );
            out.push_str("finalize_body = \"fn finish_exchange(response: response) -> status { commit(response) }\"\n");
        }
        "winsock.socket-io" => {
            out.push_str("request = \"overlapped-client-session\"\n");
            out.push_str("response = \"iocp-completion\"\n");
            out.push_str("streaming = \"completion-port-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"socket.handle, overlapped.packet, completion.port\"\n");
            out.push_str("argument_model = \"iocp-request-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"connect_overlapped\"\n");
            out.push_str("send = \"submit_overlapped_send\"\n");
            out.push_str("recv = \"await_iocp_completion\"\n");
            out.push_str("finalize = \"finish_iocp_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str(
                "connect_body = \"fn connect_overlapped(addr: text) -> socket { socket(addr) }\"\n",
            );
            out.push_str("send_body = \"fn submit_overlapped_send(socket: socket, packet: packet) -> overlapped { overlapped(socket, packet) }\"\n");
            out.push_str("recv_body = \"fn await_iocp_completion(op: overlapped) -> response { response(op) }\"\n");
            out.push_str("finalize_body = \"fn finish_iocp_exchange(response: response) -> status { finalize(response) }\"\n");
        }
        "socket-abi.socket-io" => {
            out.push_str("request = \"socket-reactor-session\"\n");
            out.push_str("response = \"poll-ready-response\"\n");
            out.push_str("streaming = \"fd-edge-stream\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"fd.handle, packet.buffer, ready.token\"\n");
            out.push_str("argument_model = \"socket-poll-bundle\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"open_fd_session\"\n");
            out.push_str("send = \"submit_send_recv\"\n");
            out.push_str("recv = \"poll_ready_response\"\n");
            out.push_str("finalize = \"finish_poll_exchange\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"fn open_fd_session(addr: text) -> fd { fd(addr) }\"\n");
            out.push_str("send_body = \"fn submit_send_recv(fd: fd, packet: packet) -> token { token(fd, packet) }\"\n");
            out.push_str("recv_body = \"fn poll_ready_response(token: token) -> response { response(token) }\"\n");
            out.push_str("finalize_body = \"fn finish_poll_exchange(response: response) -> status { release(response) }\"\n");
        }
        _ => {
            out.push_str("request = \"generic-session\"\n");
            out.push_str("response = \"generic-response\"\n");
            out.push_str("[entry_points]\n");
            out.push_str("connect = \"unimplemented\"\n");
            out.push_str("send = \"unimplemented\"\n");
            out.push_str("recv = \"unimplemented\"\n");
            out.push_str("[source_stub]\n");
            out.push_str("connect_body = \"unimplemented\"\n");
        }
    }
    out
}
