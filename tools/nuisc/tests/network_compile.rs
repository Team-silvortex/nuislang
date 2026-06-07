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
