pub(super) fn reverse_network_data_bridge_module() -> &'static str {
    r#"
    use network NetworkUnit;
    use data FabricPlane;

    mod cpu NetworkDataBridge {
      pub fn probe_roundtrip() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        let endpoint_kind: NetworkResult<i64> =
          network_result(network_profile_endpoint_kind("NetworkUnit"));
        let send_window: NetworkResult<i64> =
          network_result(network_profile_send_window("NetworkUnit"));
        let value: i64 =
          network_value(bind_core)
          + network_value(endpoint_kind)
          + network_value(send_window);
        data_profile_bind_core("FabricPlane");
        let handles: HandleTable<FabricPlaneBindings> =
          data_profile_handle_table("FabricPlane");
        let uplink: Window<i64> =
          data_profile_send_uplink("FabricPlane", value);
        let downlink: Window<Window<i64>> =
          data_profile_send_downlink("FabricPlane", uplink);
        print(handles);
        print(downlink);
        return value;
      }
    }
    "#
}

pub(super) fn reverse_network_data_bridge_entry() -> &'static str {
    r#"
    use cpu NetworkDataBridge;
    use network NetworkUnit;
    use data FabricPlane;

    mod cpu Main {
      fn main() -> i64 {
        return NetworkDataBridge.probe_roundtrip();
      }
    }
    "#
}

pub(super) fn reverse_kernel_data_bridge_module() -> &'static str {
    r#"
    use kernel KernelUnit;
    use data FabricPlane;

    mod cpu KernelDataBridge {
      pub fn probe_roundtrip() -> i64 {
        let bind_core: KernelResult<i64> =
          kernel_result(kernel_profile_bind_core("KernelUnit"));
        let queue_depth: KernelResult<i64> =
          kernel_result(kernel_profile_queue_depth("KernelUnit"));
        let batch_lanes: KernelResult<i64> =
          kernel_result(kernel_profile_batch_lanes("KernelUnit"));
        let value: i64 =
          kernel_value(bind_core)
          + kernel_value(queue_depth)
          + kernel_value(batch_lanes);
        data_profile_bind_core("FabricPlane");
        let handles: HandleTable<FabricPlaneBindings> =
          data_profile_handle_table("FabricPlane");
        let uplink: Window<i64> =
          data_profile_send_uplink("FabricPlane", value);
        let downlink: Window<Window<i64>> =
          data_profile_send_downlink("FabricPlane", uplink);
        print(handles);
        print(downlink);
        return value;
      }
    }
    "#
}

pub(super) fn forward_network_data_bridge_missing_downlink_module() -> &'static str {
    r#"
    use network NetworkUnit;
    use data FabricPlane;

    mod cpu NetworkDataBridge {
      pub fn probe_roundtrip() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        let endpoint_kind: NetworkResult<i64> =
          network_result(network_profile_endpoint_kind("NetworkUnit"));
        let send_window: NetworkResult<i64> =
          network_result(network_profile_send_window("NetworkUnit"));
        let value: i64 =
          network_value(bind_core)
          + network_value(endpoint_kind)
          + network_value(send_window);
        data_profile_bind_core("FabricPlane");
        let handles: HandleTable<FabricPlaneBindings> =
          data_profile_handle_table("FabricPlane");
        let uplink: Window<i64> =
          data_profile_send_uplink("FabricPlane", value);
        print(handles);
        print(uplink);
        return value;
      }
    }
    "#
}

pub(super) fn kernel_task_async_shapes_entry() -> &'static str {
    r#"
    use cpu KernelTaskAsyncShapes;
    use data FabricPlane;
    use kernel KernelUnit;

    mod cpu Main {
      fn main() {
        let roundtrip_seed: i64 = KernelTaskAsyncShapes.roundtrip_seed();
        let uplink: Window<i64> = KernelTaskAsyncShapes.send_roundtrip(roundtrip_seed);
        let downlink: Window<Window<i64>> =
          KernelTaskAsyncShapes.receive_roundtrip(uplink);
        print(downlink);
      }
    }
    "#
}

pub(super) fn kernel_task_async_shapes_module() -> &'static str {
    r#"
    use data FabricPlane;
    use kernel KernelUnit;

    mod cpu KernelTaskAsyncShapes {
      pub fn roundtrip_seed() -> i64 {
        let bind_core: KernelResult<i64> =
          kernel_result(kernel_profile_bind_core("KernelUnit"));
        let queue_depth: KernelResult<i64> =
          kernel_result(kernel_profile_queue_depth("KernelUnit"));
        let batch_lanes: KernelResult<i64> =
          kernel_result(kernel_profile_batch_lanes("KernelUnit"));
        return kernel_value(bind_core)
          + kernel_value(queue_depth)
          + kernel_value(batch_lanes);
      }

      pub fn send_roundtrip(value: i64) -> Window<i64> {
        data_profile_bind_core("FabricPlane");
        let handles: HandleTable<FabricPlaneBindings> =
          data_profile_handle_table("FabricPlane");
        return data_profile_send_uplink("FabricPlane", value);
      }

      pub fn receive_roundtrip(uplink: Window<i64>) -> Window<Window<i64>> {
        return data_profile_send_downlink("FabricPlane", uplink);
      }
    }
    "#
}

pub(super) fn network_task_async_probe_entry() -> &'static str {
    r#"
    use cpu NetworkTaskAsyncShapes;
    use network NetworkUnit;

    mod cpu Main {
      fn main() -> i64 {
        return NetworkTaskAsyncShapes.probe();
      }
    }
    "#
}

pub(super) fn network_task_async_probe_module() -> &'static str {
    r#"
    use network NetworkUnit;

    mod cpu NetworkTaskAsyncShapes {
      pub fn probe() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        let endpoint_kind: NetworkResult<i64> =
          network_result(network_profile_endpoint_kind("NetworkUnit"));
        let send_window: NetworkResult<i64> =
          network_result(network_profile_send_window("NetworkUnit"));
        if network_config_ready(bind_core) {
          return network_value(bind_core)
            + network_value(endpoint_kind)
            + network_value(send_window);
        }
        return 0;
      }
    }
    "#
}

pub(super) fn network_task_async_transport_entry() -> &'static str {
    r#"
    use cpu NetworkTaskAsyncShapes;
    use network NetworkUnit;

    mod cpu Main {
      fn main() -> i64 {
        return NetworkTaskAsyncShapes.transport_probe();
      }
    }
    "#
}

pub(super) fn network_task_async_transport_module() -> &'static str {
    r#"
    use network NetworkUnit;

    mod cpu NetworkTaskAsyncShapes {
      pub fn transport_probe() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        let endpoint_kind: NetworkResult<i64> =
          network_result(network_profile_endpoint_kind("NetworkUnit"));
        let transport_family: NetworkResult<i64> =
          network_result(network_profile_transport_family("NetworkUnit"));
        let protocol_kind: NetworkResult<i64> =
          network_result(network_profile_protocol_kind("NetworkUnit"));
        let protocol_version: NetworkResult<i64> =
          network_result(network_profile_protocol_version("NetworkUnit"));
        if network_config_ready(bind_core) {
          return network_value(bind_core)
            + network_value(endpoint_kind)
            + network_value(transport_family)
            + network_value(protocol_kind)
            + network_value(protocol_version);
        }
        return 0;
      }
    }
    "#
}

pub(super) fn direct_network_bind_core_entry() -> &'static str {
    r#"
    use network NetworkUnit;

    mod cpu Main {
      fn main() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        if network_config_ready(bind_core) {
          return network_value(bind_core);
        }
        return 0;
      }
    }
    "#
}

pub(super) fn direct_network_protocol_kind_entry() -> &'static str {
    r#"
    use network NetworkUnit;

    mod cpu Main {
      fn main() -> i64 {
        let bind_core: NetworkResult<i64> =
          network_result(network_profile_bind_core("NetworkUnit"));
        let endpoint_kind: NetworkResult<i64> =
          network_result(network_profile_endpoint_kind("NetworkUnit"));
        let protocol_kind: NetworkResult<i64> =
          network_result(network_profile_protocol_kind("NetworkUnit"));
        if network_config_ready(bind_core) {
          return network_value(bind_core)
            + network_value(endpoint_kind)
            + network_value(protocol_kind);
        }
        return 0;
      }
    }
    "#
}
