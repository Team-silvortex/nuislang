use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, render_target_specific_backend_fields,
    render_target_specific_lowering_fields,
};
use crate::aot_toml::escape_toml_string;

pub(crate) fn render_domain_build_unit_network_ir_sidecar(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-network-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
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
