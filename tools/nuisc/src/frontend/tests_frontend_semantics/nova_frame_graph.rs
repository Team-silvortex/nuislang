use super::*;

#[test]
fn lowers_nova_resource_set_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
            let state: NovaResourceSetState = nova_resource_set_state(resource_set);
            let residency: i64 = nova_resource_set_state_residency(state);
            return residency;
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
        } => ty.render() == "NovaResourceSetState" && type_name == "NovaResourceSetState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_schedule_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
            let state: NovaScheduleState = nova_schedule_state(schedule);
            let budget: i64 = nova_schedule_state_async_budget(state);
            return budget;
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
        } => ty.render() == "NovaScheduleState" && type_name == "NovaScheduleState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_submission_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
            let state: NovaSubmissionState = nova_submission_state(submission);
            let batches: i64 = nova_submission_state_batches(state);
            return batches;
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
        } => ty.render() == "NovaSubmissionState" && type_name == "NovaSubmissionState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_queue_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
            let state: NovaQueueState = nova_queue_state(queue);
            let budget: i64 = nova_queue_state_budget(state);
            return budget;
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
        } => ty.render() == "NovaQueueState" && type_name == "NovaQueueState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_semaphore_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
            let state: NovaSemaphoreState = nova_semaphore_state(semaphore);
            let scope: i64 = nova_semaphore_state_scope(state);
            return scope;
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
        } => ty.render() == "NovaSemaphoreState" && type_name == "NovaSemaphoreState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_timeline_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
            let state: NovaTimelineState = nova_timeline_state(timeline);
            let epoch: i64 = nova_timeline_state_epoch(state);
            return epoch;
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
        } => ty.render() == "NovaTimelineState" && type_name == "NovaTimelineState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_fence_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
            let state: NovaFenceState = nova_fence_state(fence);
            let scope: i64 = nova_fence_state_scope(state);
            return scope;
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
        } => ty.render() == "NovaFenceState" && type_name == "NovaFenceState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_signal_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 4);
            let state: NovaSignalState = nova_signal_state(signal);
            let phase: i64 = nova_signal_state_phase(state);
            return phase;
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
        } => ty.render() == "NovaSignalState" && type_name == "NovaSignalState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_event_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let event: NovaEventPacket = nova_event_packet(1, 2, 3, 4);
            let state: NovaEventState = nova_event_state(event);
            let route: i64 = nova_event_state_route(state);
            return route;
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
        } => ty.render() == "NovaEventState" && type_name == "NovaEventState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_dispatch_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 4);
            let state: NovaDispatchState = nova_dispatch_state(dispatch);
            let queue_kind: i64 = nova_dispatch_state_queue_kind(state);
            return queue_kind;
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
        } => ty.render() == "NovaDispatchState" && type_name == "NovaDispatchState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_feedback_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 4);
            let state: NovaFeedbackState = nova_feedback_state(feedback);
            let status: i64 = nova_feedback_state_status(state);
            return status;
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
        } => ty.render() == "NovaFeedbackState" && type_name == "NovaFeedbackState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_intent_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 4);
            let state: NovaIntentState = nova_intent_state(intent);
            let target_slot: i64 = nova_intent_state_target(state);
            return target_slot;
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
        } => ty.render() == "NovaIntentState" && type_name == "NovaIntentState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_reaction_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 4);
            let state: NovaReactionState = nova_reaction_state(reaction);
            let result_slot: i64 = nova_reaction_state_result(state);
            return result_slot;
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
        } => ty.render() == "NovaReactionState" && type_name == "NovaReactionState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_outcome_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 4);
            let state: NovaOutcomeState = nova_outcome_state(outcome);
            let final_slot: i64 = nova_outcome_state_final(state);
            return final_slot;
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
        } => ty.render() == "NovaOutcomeState" && type_name == "NovaOutcomeState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_resolution_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 4);
            let state: NovaResolutionState = nova_resolution_state(resolution);
            let commit_slot: i64 = nova_resolution_state_commit(state);
            return commit_slot;
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
        } => ty.render() == "NovaResolutionState" && type_name == "NovaResolutionState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_commit_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 4);
            let state: NovaCommitState = nova_commit_state(commit);
            let applied_slot: i64 = nova_commit_state_applied(state);
            return applied_slot;
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
        } => ty.render() == "NovaCommitState" && type_name == "NovaCommitState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_snapshot_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 4);
            let state: NovaSnapshotState = nova_snapshot_state(snapshot);
            let source_slot: i64 = nova_snapshot_state_source(state);
            return source_slot;
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
        } => ty.render() == "NovaSnapshotState" && type_name == "NovaSnapshotState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_checkpoint_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 4);
            let state: NovaCheckpointState = nova_checkpoint_state(checkpoint);
            let anchor_slot: i64 = nova_checkpoint_state_anchor(state);
            return anchor_slot;
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
        } => ty.render() == "NovaCheckpointState" && type_name == "NovaCheckpointState",
        _ => false,
    }));
}
