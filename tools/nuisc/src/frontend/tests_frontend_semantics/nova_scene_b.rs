use super::*;

#[test]
fn lowers_nova_budget_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
            let state: NovaBudgetState = nova_budget_state(budget);
            let total: i64 = nova_budget_state_total(state);
            return total;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaBudgetState" && type_name == "NovaBudgetState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_pressure_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
            let state: NovaPressureState = nova_pressure_state(pressure);
            let level: i64 = nova_pressure_state_level(state);
            return level;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaPressureState" && type_name == "NovaPressureState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_thermal_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
            let state: NovaThermalState = nova_thermal_state(thermal);
            let level: i64 = nova_thermal_state_level(state);
            return level;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaThermalState" && type_name == "NovaThermalState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_power_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
            let state: NovaPowerState = nova_power_state(power);
            let level: i64 = nova_power_state_level(state);
            return level;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaPowerState" && type_name == "NovaPowerState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_latency_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
            let state: NovaLatencyState = nova_latency_state(latency);
            let frame: i64 = nova_latency_state_frame(state);
            return frame;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaLatencyState" && type_name == "NovaLatencyState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_frame_pacing_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
            let state: NovaFramePacingState = nova_frame_pacing_state(pacing);
            let cadence: i64 = nova_frame_pacing_state_cadence(state);
            return cadence;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaFramePacingState" && type_name == "NovaFramePacingState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_jank_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
            let state: NovaJankState = nova_jank_state(jank);
            let spikes: i64 = nova_jank_state_spikes(state);
            return spikes;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaJankState" && type_name == "NovaJankState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_frame_variance_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
            let state: NovaFrameVarianceState = nova_frame_variance_state(variance);
            let frame: i64 = nova_frame_variance_state_frame(state);
            return frame;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaFrameVarianceState" && type_name == "NovaFrameVarianceState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_pass_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
            let state: NovaPassState = nova_pass_state(pass);
            let samples: i64 = nova_pass_state_sample_count(state);
            return samples;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaPassState" && type_name == "NovaPassState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_frame_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
            let state: NovaFrameState = nova_frame_state(frame);
            let exposure: i64 = nova_frame_state_exposure(state);
            return exposure;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaFrameState" && type_name == "NovaFrameState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_target_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
            let state: NovaTargetState = nova_target_state(target);
            let msaa: i64 = nova_target_state_multisample(state);
            return msaa;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaTargetState" && type_name == "NovaTargetState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_frame_graph_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
            let state: NovaFrameGraphState = nova_frame_graph_state(frame_graph);
            let passes: i64 = nova_frame_graph_state_passes(state);
            return passes;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaFrameGraphState" && type_name == "NovaFrameGraphState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_attachment_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
            let state: NovaAttachmentState = nova_attachment_state(attachment);
            let format_kind: i64 = nova_attachment_state_format_kind(state);
            return format_kind;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaAttachmentState" && type_name == "NovaAttachmentState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_pass_chain_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
            let state: NovaPassChainState = nova_pass_chain_state(pass_chain);
            let stages: i64 = nova_pass_chain_state_stages(state);
            return stages;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaPassChainState" && type_name == "NovaPassChainState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_barrier_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
            let state: NovaBarrierState = nova_barrier_state(barrier);
            let flush_mode: i64 = nova_barrier_state_flush_mode(state);
            return flush_mode;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaBarrierState" && type_name == "NovaBarrierState",
        _ => false,
    }));
}
