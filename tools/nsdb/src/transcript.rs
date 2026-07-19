use crate::{
    model::NsdbInspectReport,
    replay::{build_replay_plan, NsdbReplayCheckpoint},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NsdbReplayControl {
    pub(crate) frame_selector: Option<String>,
    pub(crate) breakpoint_selector: Option<String>,
    pub(crate) breakpoint_phase: Option<String>,
    pub(crate) breakpoint_entry: Option<String>,
    pub(crate) resume_after_frame_id: Option<String>,
    pub(crate) resume_next_frame_id: Option<String>,
}

pub(crate) struct NsdbReplayTranscript {
    pub(crate) protocol: &'static str,
    pub(crate) source_contract: &'static str,
    pub(crate) control_protocol: &'static str,
    pub(crate) control_mode: &'static str,
    pub(crate) control_selector: Option<String>,
    pub(crate) control_status: &'static str,
    pub(crate) breakpoint_predicate_protocol: &'static str,
    pub(crate) breakpoint_phase: Option<String>,
    pub(crate) breakpoint_entry: Option<String>,
    pub(crate) resume_input_protocol: &'static str,
    pub(crate) resume_input_status: &'static str,
    pub(crate) resume_input_after_frame_id: Option<String>,
    pub(crate) resume_input_next_frame_id: Option<String>,
    pub(crate) selected_frame_index: Option<usize>,
    pub(crate) selected_frame_id: Option<String>,
    pub(crate) stop_reason: &'static str,
    pub(crate) resume_cursor_protocol: &'static str,
    pub(crate) resume_cursor_status: &'static str,
    pub(crate) resume_cursor_ready: bool,
    pub(crate) resume_after_frame_id: Option<String>,
    pub(crate) resume_next_frame_index: Option<usize>,
    pub(crate) resume_next_frame_id: Option<String>,
    pub(crate) status: &'static str,
    pub(crate) ready: bool,
    pub(crate) checkpoint_count: usize,
    pub(crate) replayed_checkpoint_count: usize,
    pub(crate) first_blocker: Option<String>,
    pub(crate) frames: Vec<NsdbReplayTranscriptFrame>,
}

pub(crate) struct NsdbReplayTranscriptFrame {
    pub(crate) index: usize,
    pub(crate) trace_id: String,
    pub(crate) frame_id: String,
    pub(crate) checkpoint_kind: String,
    pub(crate) execution_phase: String,
    pub(crate) entry_symbol: String,
    pub(crate) replay_status: String,
    pub(crate) consumed: bool,
    pub(crate) value_slot_id: String,
    pub(crate) value_snapshot_status: String,
    pub(crate) value_snapshot_type: String,
    pub(crate) value_snapshot_summary: String,
    pub(crate) value_content_status: String,
    pub(crate) value_content_summary: String,
    pub(crate) next_action: String,
}

pub(crate) fn build_replay_transcript(report: &NsdbInspectReport) -> NsdbReplayTranscript {
    build_replay_transcript_with_control(report, &NsdbReplayControl::default())
}

pub(crate) fn build_replay_transcript_with_control(
    report: &NsdbInspectReport,
    control: &NsdbReplayControl,
) -> NsdbReplayTranscript {
    let plan = build_replay_plan(report);
    let plan_ready = plan.status == "ready"
        && plan.checkpoint_count > 0
        && plan.replayable_checkpoint_count == plan.checkpoint_count;
    let mut frames = plan
        .checkpoints
        .iter()
        .map(|checkpoint| transcript_frame(checkpoint, false))
        .collect::<Vec<_>>();
    let control_result = apply_replay_control(&mut frames, plan_ready, control);
    let ready = plan_ready && control_result.blocker.is_none();
    let resume_cursor = replay_resume_cursor(&frames, &control_result);
    NsdbReplayTranscript {
        protocol: "nsdb-yir-replay-transcript-v1",
        source_contract: plan.protocol,
        control_protocol: "nsdb-yir-replay-control-v1",
        control_mode: control_result.mode,
        control_selector: control_result.selector,
        control_status: control_result.status,
        breakpoint_predicate_protocol: "nsdb-yir-breakpoint-predicate-v1",
        breakpoint_phase: control.breakpoint_phase.clone(),
        breakpoint_entry: control.breakpoint_entry.clone(),
        resume_input_protocol: "nsdb-yir-replay-resume-input-v1",
        resume_input_status: control_result.resume_input_status,
        resume_input_after_frame_id: control.resume_after_frame_id.clone(),
        resume_input_next_frame_id: control.resume_next_frame_id.clone(),
        selected_frame_index: control_result.selected_frame_index,
        selected_frame_id: control_result.selected_frame_id.clone(),
        stop_reason: if plan_ready {
            control_result.stop_reason
        } else {
            "transcript-blocked"
        },
        resume_cursor_protocol: "nsdb-yir-replay-resume-cursor-v1",
        resume_cursor_status: resume_cursor.status,
        resume_cursor_ready: resume_cursor.ready,
        resume_after_frame_id: resume_cursor.after_frame_id,
        resume_next_frame_index: resume_cursor.next_frame_index,
        resume_next_frame_id: resume_cursor.next_frame_id,
        status: if ready && control_result.mode == "all" {
            "transcript-consumed"
        } else if ready && control_result.mode == "resume" {
            "transcript-resumed"
        } else if ready {
            "transcript-stopped"
        } else if plan_ready {
            "transcript-control-blocked"
        } else {
            "transcript-blocked"
        },
        ready,
        checkpoint_count: plan.checkpoint_count,
        replayed_checkpoint_count: frames.iter().filter(|frame| frame.consumed).count(),
        first_blocker: if ready {
            None
        } else if plan_ready {
            control_result.blocker
        } else {
            plan.first_blocker
                .or_else(|| Some("payload-execution-replay:no-checkpoints".to_owned()))
        },
        frames,
    }
}

struct ReplayControlResult {
    mode: &'static str,
    selector: Option<String>,
    status: &'static str,
    selected_frame_index: Option<usize>,
    selected_frame_id: Option<String>,
    stop_reason: &'static str,
    resume_input_status: &'static str,
    blocker: Option<String>,
}

struct ReplayResumeCursor {
    status: &'static str,
    ready: bool,
    after_frame_id: Option<String>,
    next_frame_index: Option<usize>,
    next_frame_id: Option<String>,
}

fn apply_replay_control(
    frames: &mut [NsdbReplayTranscriptFrame],
    plan_ready: bool,
    control: &NsdbReplayControl,
) -> ReplayControlResult {
    let resume_requested =
        control.resume_after_frame_id.is_some() || control.resume_next_frame_id.is_some();
    let (mode, selector) = if let Some(selector) = &control.frame_selector {
        ("frame", Some(selector.clone()))
    } else if let Some(selector) = &control.breakpoint_selector {
        ("breakpoint", Some(selector.clone()))
    } else if control.breakpoint_phase.is_some() || control.breakpoint_entry.is_some() {
        ("predicate", None)
    } else if resume_requested {
        ("resume", None)
    } else {
        ("all", None)
    };
    if !plan_ready {
        return ReplayControlResult {
            mode,
            selector,
            status: "not-evaluated",
            selected_frame_index: None,
            selected_frame_id: None,
            stop_reason: "transcript-blocked",
            resume_input_status: "not-evaluated",
            blocker: None,
        };
    }
    let (start_position, resume_input_status) = match replay_resume_start(frames, control) {
        Ok(result) => result,
        Err(blocker) => {
            return ReplayControlResult {
                mode,
                selector,
                status: "resume-cursor-rejected",
                selected_frame_index: None,
                selected_frame_id: None,
                stop_reason: "resume-cursor-rejected",
                resume_input_status: "cursor-rejected",
                blocker: Some(blocker),
            };
        }
    };
    if mode == "predicate" {
        let matches = frames
            .iter()
            .enumerate()
            .skip(start_position)
            .filter(|(_, frame)| {
                control
                    .breakpoint_phase
                    .as_deref()
                    .is_none_or(|phase| frame.execution_phase == phase)
                    && control
                        .breakpoint_entry
                        .as_deref()
                        .is_none_or(|entry| frame.entry_symbol == entry)
            })
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        let Some(selected_position) = matches.first().copied() else {
            let predicate = format!(
                "phase={};entry={}",
                control.breakpoint_phase.as_deref().unwrap_or("*"),
                control.breakpoint_entry.as_deref().unwrap_or("*")
            );
            return ReplayControlResult {
                mode,
                selector: Some(predicate.clone()),
                status: "predicate-not-matched",
                selected_frame_index: None,
                selected_frame_id: None,
                stop_reason: "breakpoint-predicate-unmatched",
                resume_input_status,
                blocker: Some(format!("replay-control:predicate-not-matched:{predicate}")),
            };
        };
        for frame in frames
            .iter_mut()
            .skip(start_position)
            .take(selected_position - start_position + 1)
        {
            frame.consumed = true;
        }
        return ReplayControlResult {
            mode,
            selector: Some(format!(
                "phase={};entry={}",
                control.breakpoint_phase.as_deref().unwrap_or("*"),
                control.breakpoint_entry.as_deref().unwrap_or("*")
            )),
            status: "breakpoint-predicate-hit",
            selected_frame_index: Some(frames[selected_position].index),
            selected_frame_id: Some(frames[selected_position].frame_id.clone()),
            stop_reason: "breakpoint-predicate-hit",
            resume_input_status,
            blocker: None,
        };
    }
    let Some(selector_value) = selector.clone() else {
        for frame in frames.iter_mut().skip(start_position) {
            frame.consumed = true;
        }
        return ReplayControlResult {
            mode,
            selector,
            status: if mode == "resume" {
                "resume-consumed"
            } else {
                "all-frames-consumed"
            },
            selected_frame_index: None,
            selected_frame_id: None,
            stop_reason: if mode == "resume" {
                "resume-end-of-transcript"
            } else {
                "end-of-transcript"
            },
            resume_input_status,
            blocker: None,
        };
    };
    let matches = frames
        .iter()
        .enumerate()
        .skip(start_position)
        .filter(|(_, frame)| {
            frame.frame_id == selector_value || frame.index.to_string() == selector_value
        })
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        let reason = if matches.is_empty() {
            "target-not-found"
        } else {
            "target-ambiguous"
        };
        return ReplayControlResult {
            mode,
            selector,
            status: reason,
            selected_frame_index: None,
            selected_frame_id: None,
            stop_reason: "control-target-unresolved",
            resume_input_status,
            blocker: Some(format!("replay-control:{reason}:{selector_value}")),
        };
    }
    let selected_position = matches[0];
    if mode == "frame" {
        frames[selected_position].consumed = true;
    } else {
        for frame in frames
            .iter_mut()
            .skip(start_position)
            .take(selected_position - start_position + 1)
        {
            frame.consumed = true;
        }
    }
    ReplayControlResult {
        mode,
        selector,
        status: if mode == "frame" {
            "frame-selected"
        } else {
            "breakpoint-hit"
        },
        selected_frame_index: Some(frames[selected_position].index),
        selected_frame_id: Some(frames[selected_position].frame_id.clone()),
        stop_reason: if mode == "frame" {
            "frame-selected"
        } else {
            "breakpoint-hit"
        },
        resume_input_status,
        blocker: None,
    }
}

fn replay_resume_start(
    frames: &[NsdbReplayTranscriptFrame],
    control: &NsdbReplayControl,
) -> Result<(usize, &'static str), String> {
    match (
        control.resume_after_frame_id.as_deref(),
        control.resume_next_frame_id.as_deref(),
    ) {
        (None, None) => Ok((0, "not-requested")),
        (Some(_), None) | (None, Some(_)) => Err("replay-resume:cursor-pair-incomplete".to_owned()),
        (Some(after), Some(next)) => {
            let positions = frames
                .iter()
                .enumerate()
                .filter(|(_, frame)| frame.frame_id == after)
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            if positions.len() != 1 {
                return Err(format!("replay-resume:after-frame-unresolved:{after}"));
            }
            let next_position = positions[0] + 1;
            let Some(actual_next) = frames.get(next_position) else {
                return Err(format!("replay-resume:after-frame-terminal:{after}"));
            };
            if actual_next.frame_id != next {
                return Err(format!(
                    "replay-resume:next-frame-mismatch:expected={next}:actual={}",
                    actual_next.frame_id
                ));
            }
            Ok((next_position, "cursor-accepted"))
        }
    }
}

fn replay_resume_cursor(
    frames: &[NsdbReplayTranscriptFrame],
    control: &ReplayControlResult,
) -> ReplayResumeCursor {
    let Some(selected_id) = control.selected_frame_id.as_deref() else {
        return ReplayResumeCursor {
            status: "not-created",
            ready: false,
            after_frame_id: None,
            next_frame_index: None,
            next_frame_id: None,
        };
    };
    let Some(position) = frames
        .iter()
        .position(|frame| frame.frame_id == selected_id)
    else {
        return ReplayResumeCursor {
            status: "invalid-selected-frame",
            ready: false,
            after_frame_id: Some(selected_id.to_owned()),
            next_frame_index: None,
            next_frame_id: None,
        };
    };
    let Some(next) = frames.get(position + 1) else {
        return ReplayResumeCursor {
            status: "end-of-transcript",
            ready: false,
            after_frame_id: Some(selected_id.to_owned()),
            next_frame_index: None,
            next_frame_id: None,
        };
    };
    ReplayResumeCursor {
        status: "resume-ready",
        ready: true,
        after_frame_id: Some(selected_id.to_owned()),
        next_frame_index: Some(next.index),
        next_frame_id: Some(next.frame_id.clone()),
    }
}

fn transcript_frame(
    checkpoint: &NsdbReplayCheckpoint,
    transcript_ready: bool,
) -> NsdbReplayTranscriptFrame {
    NsdbReplayTranscriptFrame {
        index: checkpoint.index,
        trace_id: checkpoint.trace_id.clone(),
        frame_id: checkpoint.frame_id.clone(),
        checkpoint_kind: checkpoint.checkpoint_kind.clone(),
        execution_phase: checkpoint.execution_phase.clone(),
        entry_symbol: checkpoint.entry_symbol.clone(),
        replay_status: checkpoint.replay_status.clone(),
        consumed: transcript_ready && checkpoint.replay_status == "replayable",
        value_slot_id: checkpoint.value_slot_id.clone(),
        value_snapshot_status: checkpoint.value_snapshot_status.clone(),
        value_snapshot_type: checkpoint.value_snapshot_type.clone(),
        value_snapshot_summary: checkpoint.value_snapshot_summary.clone(),
        value_content_status: checkpoint.value_content_status.clone(),
        value_content_summary: checkpoint.value_content_summary.clone(),
        next_action: checkpoint.next_action.clone(),
    }
}

#[cfg(test)]
mod control_tests {
    use super::{apply_replay_control, NsdbReplayControl, NsdbReplayTranscriptFrame};

    fn frames() -> Vec<NsdbReplayTranscriptFrame> {
        (0..3)
            .map(|index| NsdbReplayTranscriptFrame {
                index,
                trace_id: format!("trace-{index}"),
                frame_id: format!("frame-{index}"),
                checkpoint_kind: "payload-execution-checkpoint".to_owned(),
                execution_phase: if index == 1 {
                    "device-dispatch".to_owned()
                } else {
                    "payload-execution".to_owned()
                },
                entry_symbol: if index == 1 {
                    "pixelmagic.blur".to_owned()
                } else {
                    "main".to_owned()
                },
                replay_status: "replayable".to_owned(),
                consumed: false,
                value_slot_id: format!("slot-{index}"),
                value_snapshot_status: "snapshot-ready".to_owned(),
                value_snapshot_type: "i64".to_owned(),
                value_snapshot_summary: index.to_string(),
                value_content_status: "value-ready".to_owned(),
                value_content_summary: index.to_string(),
                next_action: "continue".to_owned(),
            })
            .collect()
    }

    #[test]
    fn selects_exact_frame_by_id() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: Some("frame-1".to_owned()),
                breakpoint_selector: None,
                breakpoint_phase: None,
                breakpoint_entry: None,
                resume_after_frame_id: None,
                resume_next_frame_id: None,
            },
        );

        assert_eq!(result.status, "frame-selected");
        assert_eq!(result.selected_frame_index, Some(1));
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.consumed)
                .collect::<Vec<_>>(),
            vec![false, true, false]
        );
    }

    #[test]
    fn breakpoint_consumes_through_selected_index() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: None,
                breakpoint_selector: Some("1".to_owned()),
                breakpoint_phase: None,
                breakpoint_entry: None,
                resume_after_frame_id: None,
                resume_next_frame_id: None,
            },
        );

        assert_eq!(result.status, "breakpoint-hit");
        assert_eq!(result.stop_reason, "breakpoint-hit");
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.consumed)
                .collect::<Vec<_>>(),
            vec![true, true, false]
        );
    }

    #[test]
    fn unresolved_control_target_fails_closed() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: Some("missing".to_owned()),
                breakpoint_selector: None,
                breakpoint_phase: None,
                breakpoint_entry: None,
                resume_after_frame_id: None,
                resume_next_frame_id: None,
            },
        );

        assert_eq!(result.status, "target-not-found");
        assert_eq!(
            result.blocker.as_deref(),
            Some("replay-control:target-not-found:missing")
        );
        assert!(frames.iter().all(|frame| !frame.consumed));
    }

    #[test]
    fn predicate_breakpoint_produces_resume_cursor() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: None,
                breakpoint_selector: None,
                breakpoint_phase: Some("device-dispatch".to_owned()),
                breakpoint_entry: Some("pixelmagic.blur".to_owned()),
                resume_after_frame_id: None,
                resume_next_frame_id: None,
            },
        );
        let cursor = super::replay_resume_cursor(&frames, &result);

        assert_eq!(result.status, "breakpoint-predicate-hit");
        assert_eq!(result.selected_frame_id.as_deref(), Some("frame-1"));
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.consumed)
                .collect::<Vec<_>>(),
            vec![true, true, false]
        );
        assert_eq!(cursor.status, "resume-ready");
        assert!(cursor.ready);
        assert_eq!(cursor.after_frame_id.as_deref(), Some("frame-1"));
        assert_eq!(cursor.next_frame_index, Some(2));
        assert_eq!(cursor.next_frame_id.as_deref(), Some("frame-2"));
    }

    #[test]
    fn accepted_resume_cursor_consumes_only_the_suffix() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: None,
                breakpoint_selector: None,
                breakpoint_phase: None,
                breakpoint_entry: None,
                resume_after_frame_id: Some("frame-0".to_owned()),
                resume_next_frame_id: Some("frame-1".to_owned()),
            },
        );

        assert_eq!(result.mode, "resume");
        assert_eq!(result.status, "resume-consumed");
        assert_eq!(result.resume_input_status, "cursor-accepted");
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame.consumed)
                .collect::<Vec<_>>(),
            vec![false, true, true]
        );
    }

    #[test]
    fn tampered_resume_cursor_fails_closed() {
        let mut frames = frames();
        let result = apply_replay_control(
            &mut frames,
            true,
            &NsdbReplayControl {
                frame_selector: None,
                breakpoint_selector: None,
                breakpoint_phase: None,
                breakpoint_entry: None,
                resume_after_frame_id: Some("frame-0".to_owned()),
                resume_next_frame_id: Some("frame-2".to_owned()),
            },
        );

        assert_eq!(result.status, "resume-cursor-rejected");
        assert_eq!(result.resume_input_status, "cursor-rejected");
        assert!(result
            .blocker
            .as_deref()
            .is_some_and(|blocker| blocker.contains("next-frame-mismatch")));
        assert!(frames.iter().all(|frame| !frame.consumed));
    }
}
