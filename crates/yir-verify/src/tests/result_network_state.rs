use super::*;

#[test]
fn accepts_network_control_result_probes() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "network0".to_owned(),
                kind: ResourceKind::parse("network.io"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_network_NetworkUnit_local_port",
                "network0",
                "network.const_i64",
                &["7001"],
            ),
            node(
                "project_profile_network_NetworkUnit_remote_port",
                "network0",
                "network.const_i64",
                &["7443"],
            ),
            node(
                "project_profile_network_NetworkUnit_connect_timeout_ms",
                "network0",
                "network.const_i64",
                &["1500"],
            ),
            node(
                "project_profile_network_NetworkUnit_read_timeout_ms",
                "network0",
                "network.const_i64",
                &["800"],
            ),
            node(
                "project_profile_network_NetworkUnit_write_timeout_ms",
                "network0",
                "network.const_i64",
                &["900"],
            ),
            node(
                "local_port_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_local_port",
                    "config_ready",
                ],
            ),
            node(
                "remote_port_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_remote_port",
                    "config_ready",
                ],
            ),
            node(
                "connect_timeout_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "config_ready",
                ],
            ),
            node(
                "read_timeout_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_read_timeout_ms",
                    "config_ready",
                ],
            ),
            node(
                "write_timeout_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_write_timeout_ms",
                    "config_ready",
                ],
            ),
            node(
                "local_port",
                "network0",
                "network.value",
                &["local_port_seed"],
            ),
            node(
                "remote_port",
                "network0",
                "network.value",
                &["remote_port_seed"],
            ),
            node(
                "connect_timeout",
                "network0",
                "network.value",
                &["connect_timeout_seed"],
            ),
            node(
                "read_timeout",
                "network0",
                "network.value",
                &["read_timeout_seed"],
            ),
            node(
                "write_timeout",
                "network0",
                "network.value",
                &["write_timeout_seed"],
            ),
            node(
                "socket_handle",
                "network0",
                "network.value",
                &["local_port_seed"],
            ),
            node(
                "connect_result",
                "network0",
                "network.connect",
                &["local_port", "remote_port", "connect_timeout"],
            ),
            node(
                "accept_probe",
                "cpu0",
                "cpu.extern_call_i64",
                &[
                    "c",
                    "host_network_accept_probe",
                    "local_port",
                    "read_timeout",
                    "write_timeout",
                ],
            ),
            node(
                "accept_result",
                "network0",
                "network.observe",
                &["accept_probe", "accept_ready"],
            ),
            node(
                "close_result",
                "network0",
                "network.close",
                &["socket_handle"],
            ),
            node(
                "connect_ready",
                "network0",
                "network.is_connect_ready",
                &["connect_result"],
            ),
            node(
                "accept_ready_probe",
                "network0",
                "network.is_accept_ready",
                &["accept_result"],
            ),
            node("closed", "network0", "network.is_closed", &["close_result"]),
        ],
        edges: vec![
            dep(
                "project_profile_network_NetworkUnit_local_port",
                "local_port_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_remote_port",
                "remote_port_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_connect_timeout_ms",
                "connect_timeout_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_read_timeout_ms",
                "read_timeout_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_write_timeout_ms",
                "write_timeout_seed",
            ),
            dep("local_port_seed", "local_port"),
            dep("remote_port_seed", "remote_port"),
            dep("connect_timeout_seed", "connect_timeout"),
            dep("read_timeout_seed", "read_timeout"),
            dep("write_timeout_seed", "write_timeout"),
            dep("local_port_seed", "socket_handle"),
            dep("local_port", "connect_result"),
            dep("remote_port", "connect_result"),
            dep("connect_timeout", "connect_result"),
            xfer("local_port", "accept_probe"),
            xfer("read_timeout", "accept_probe"),
            xfer("write_timeout", "accept_probe"),
            xfer("accept_probe", "accept_result"),
            dep("socket_handle", "close_result"),
            dep("connect_result", "connect_ready"),
            dep("accept_result", "accept_ready_probe"),
            dep("close_result", "closed"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn accepts_network_value_from_connect_result() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "network0".to_owned(),
                kind: ResourceKind::parse("network.io"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_network_NetworkUnit_local_port",
                "network0",
                "network.const_i64",
                &["7001"],
            ),
            node(
                "project_profile_network_NetworkUnit_remote_port",
                "network0",
                "network.const_i64",
                &["7443"],
            ),
            node(
                "project_profile_network_NetworkUnit_connect_timeout_ms",
                "network0",
                "network.const_i64",
                &["1500"],
            ),
            node(
                "local_port_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_local_port",
                    "config_ready",
                ],
            ),
            node(
                "remote_port_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_remote_port",
                    "config_ready",
                ],
            ),
            node(
                "connect_timeout_seed",
                "network0",
                "network.observe",
                &[
                    "project_profile_network_NetworkUnit_connect_timeout_ms",
                    "config_ready",
                ],
            ),
            node(
                "local_port",
                "network0",
                "network.value",
                &["local_port_seed"],
            ),
            node(
                "remote_port",
                "network0",
                "network.value",
                &["remote_port_seed"],
            ),
            node(
                "connect_timeout",
                "network0",
                "network.value",
                &["connect_timeout_seed"],
            ),
            node(
                "connect_result",
                "network0",
                "network.connect",
                &["local_port", "remote_port", "connect_timeout"],
            ),
            node(
                "connect_value",
                "network0",
                "network.value",
                &["connect_result"],
            ),
        ],
        edges: vec![
            dep(
                "project_profile_network_NetworkUnit_local_port",
                "local_port_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_remote_port",
                "remote_port_seed",
            ),
            dep(
                "project_profile_network_NetworkUnit_connect_timeout_ms",
                "connect_timeout_seed",
            ),
            dep("local_port_seed", "local_port"),
            dep("remote_port_seed", "remote_port"),
            dep("connect_timeout_seed", "connect_timeout"),
            dep("local_port", "connect_result"),
            dep("remote_port", "connect_result"),
            dep("connect_timeout", "connect_result"),
            dep("connect_result", "connect_value"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn accepts_network_observe_from_host_transport_probe() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "network0".to_owned(),
                kind: ResourceKind::parse("network.io"),
            },
        ],
        nodes: vec![
            node("stream_window", "cpu0", "cpu.const_i64", &["64"]),
            node("send_window", "cpu0", "cpu.const_i64", &["32"]),
            node("recv_window", "cpu0", "cpu.const_i64", &["32"]),
            node("local_port", "cpu0", "cpu.const_i64", &["9000"]),
            node("remote_port", "cpu0", "cpu.const_i64", &["443"]),
            node(
                "send_probe",
                "cpu0",
                "cpu.extern_call_i64",
                &[
                    "c",
                    "host_network_send_probe",
                    "stream_window",
                    "send_window",
                    "remote_port",
                ],
            ),
            node(
                "send_owned",
                "cpu0",
                "cpu.extern_call_i64",
                &[
                    "c",
                    "host_network_send_owned",
                    "remote_port",
                    "stream_window",
                    "send_window",
                ],
            ),
            node(
                "recv_probe",
                "cpu0",
                "cpu.extern_call_i64",
                &[
                    "c",
                    "host_network_recv_probe",
                    "stream_window",
                    "recv_window",
                    "local_port",
                ],
            ),
            node(
                "recv_owned",
                "cpu0",
                "cpu.extern_call_i64",
                &[
                    "c",
                    "host_network_recv_owned",
                    "local_port",
                    "stream_window",
                    "recv_window",
                ],
            ),
            node(
                "send_seed",
                "network0",
                "network.observe",
                &["send_probe", "send_ready"],
            ),
            node(
                "send_owned_seed",
                "network0",
                "network.observe",
                &["send_owned", "send_ready"],
            ),
            node(
                "close_probe",
                "cpu0",
                "cpu.extern_call_i64",
                &["c", "host_network_close", "local_port"],
            ),
            node(
                "close_seed",
                "network0",
                "network.observe",
                &["close_probe", "closed"],
            ),
            node(
                "recv_seed",
                "network0",
                "network.observe",
                &["recv_probe", "recv_ready"],
            ),
            node(
                "recv_owned_seed",
                "network0",
                "network.observe",
                &["recv_owned", "recv_ready"],
            ),
            node(
                "send_ready_probe",
                "network0",
                "network.is_send_ready",
                &["send_seed"],
            ),
            node(
                "recv_ready_probe",
                "network0",
                "network.is_recv_ready",
                &["recv_seed"],
            ),
            node("send_value", "network0", "network.value", &["send_seed"]),
            node("recv_value", "network0", "network.value", &["recv_seed"]),
        ],
        edges: vec![
            dep("stream_window", "send_probe"),
            dep("send_window", "send_probe"),
            dep("remote_port", "send_probe"),
            dep("remote_port", "send_owned"),
            dep("stream_window", "send_owned"),
            dep("send_window", "send_owned"),
            dep("stream_window", "recv_probe"),
            dep("recv_window", "recv_probe"),
            dep("local_port", "recv_probe"),
            dep("local_port", "recv_owned"),
            dep("stream_window", "recv_owned"),
            dep("recv_window", "recv_owned"),
            dep("local_port", "close_probe"),
            xfer("send_probe", "send_seed"),
            xfer("send_owned", "send_owned_seed"),
            xfer("close_probe", "close_seed"),
            xfer("recv_probe", "recv_seed"),
            xfer("recv_owned", "recv_owned_seed"),
            dep("send_seed", "send_ready_probe"),
            dep("recv_seed", "recv_ready_probe"),
            dep("send_seed", "send_value"),
            dep("recv_seed", "recv_value"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}
