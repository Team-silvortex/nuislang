use super::link_plan_final_output_replay_vocabulary::FinalOutputReplayVocabulary;

pub(super) struct NsldFinalExecutableOutputClosureMirror {
    pub(super) contract: String,
    pub(super) ready: bool,
    pub(super) status: String,
    pub(super) next_action: String,
    pub(super) next_command: Option<String>,
    pub(super) first_blocker: Option<String>,
}

pub(super) fn object_package_summary(
    replay: &FinalOutputReplayVocabulary,
) -> NsldFinalExecutableOutputClosureMirror {
    closure_summary(
        replay,
        "nsld-object-package-summary-v1",
        "replay-ready",
        "replay-blocked",
        "consume-object-package-summary",
        "resolve-object-package-replay-evidence",
    )
}

pub(super) fn debugger_transcript_summary(
    replay: &FinalOutputReplayVocabulary,
) -> NsldFinalExecutableOutputClosureMirror {
    closure_summary(
        replay,
        "nsdb-yir-replay-transcript-v1",
        "transcript-ready",
        "transcript-blocked",
        "consume-nsdb-yir-replay-transcript",
        "resolve-nsdb-yir-replay-transcript",
    )
}

fn closure_summary(
    replay: &FinalOutputReplayVocabulary,
    contract: &str,
    ready_status: &str,
    blocked_status: &str,
    ready_action: &str,
    blocked_action: &str,
) -> NsldFinalExecutableOutputClosureMirror {
    NsldFinalExecutableOutputClosureMirror {
        contract: contract.to_owned(),
        ready: replay.ready,
        status: if replay.ready {
            ready_status
        } else {
            blocked_status
        }
        .to_owned(),
        next_action: if replay.ready {
            ready_action
        } else {
            blocked_action
        }
        .to_owned(),
        next_command: replay.next_command.clone(),
        first_blocker: (!replay.ready).then(|| {
            replay
                .first_blocker
                .as_deref()
                .unwrap_or("payload-execution-replay:unknown")
                .to_owned()
        }),
    }
}
