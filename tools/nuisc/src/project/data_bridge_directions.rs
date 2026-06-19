#[derive(Clone, Copy)]
pub(super) struct DataBridgeDirection {
    pub index: usize,
    pub is_uplink: bool,
    pub name: &'static str,
    pub pipe_marker: &'static str,
    pub pipe_class_marker: &'static str,
    pub payload_class_marker: &'static str,
    pub payload_shape_marker: &'static str,
    pub window_policy_marker: &'static str,
    pub window_policy_payload: &'static str,
}

pub(super) fn data_bridge_directions() -> [DataBridgeDirection; 2] {
    [
        DataBridgeDirection {
            index: 0,
            is_uplink: true,
            name: "uplink",
            pipe_marker: "uplink_pipe",
            pipe_class_marker: "uplink_pipe_class",
            payload_class_marker: "marker:uplink_payload_class",
            payload_shape_marker: "marker:uplink_payload_shape",
            window_policy_marker: "marker:uplink_window_policy",
            window_policy_payload: "UplinkWindowPolicy",
        },
        DataBridgeDirection {
            index: 1,
            is_uplink: false,
            name: "downlink",
            pipe_marker: "downlink_pipe",
            pipe_class_marker: "downlink_pipe_class",
            payload_class_marker: "marker:downlink_payload_class",
            payload_shape_marker: "marker:downlink_payload_shape",
            window_policy_marker: "marker:downlink_window_policy",
            window_policy_payload: "DownlinkWindowPolicy",
        },
    ]
}
