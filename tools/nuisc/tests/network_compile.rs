use std::path::Path;

#[test]
fn compiles_httpish_protocol_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_protocol_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("httpish protocol project should compile");
}

#[test]
fn compiles_http_request_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_request_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http request project should compile");
}

#[test]
fn compiles_http_client_lane_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_lane_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http client lane project should compile");
}

#[test]
fn compiles_http_service_lane_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_service_lane_recipe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("http service lane project should compile");
}

#[test]
fn compiles_httpish_client_session_packet_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_client_session_packet_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish client session packet project should compile");
}

#[test]
fn compiles_httpish_service_session_packet_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_service_session_packet_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish service session packet project should compile");
}

#[test]
fn compiles_httpish_header_session_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_session_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish header session project should compile");
}

#[test]
fn compiles_httpish_header_service_session_recipe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_header_service_session_recipe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish header service session project should compile");
}

#[test]
fn compiles_httpish_exchange_contract_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_contract_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange contract project should compile");
}

#[test]
fn compiles_httpish_exchange_contract_service_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_contract_service_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange contract service project should compile");
}

#[test]
fn compiles_httpish_exchange_blocks_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_blocks_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange blocks project should compile");
}

#[test]
fn compiles_httpish_exchange_blocks_service_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_httpish_exchange_blocks_service_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("httpish exchange blocks service project should compile");
}

#[test]
fn compiles_network_host_handle_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_handle_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network host handle runtime probe project should compile");
}

#[test]
fn compiles_http_client_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_client_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("http client runtime probe project should compile");
}

#[test]
fn compiles_tcp_socket_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_socket_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tcp socket runtime probe project should compile");
}

#[test]
fn compiles_tcp_send_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_tcp_send_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tcp send runtime probe project should compile");
}

#[test]
fn compiles_http_status_runtime_probe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/net_http_status_runtime_probe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("http status runtime probe project should compile");
}

#[test]
fn compiles_network_loopback_runtime_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_loopback_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network loopback runtime project should compile");
}

#[test]
fn compiles_network_host_open_surface_runtime_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_open_surface_runtime_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("network host open surface runtime project should compile");
}
