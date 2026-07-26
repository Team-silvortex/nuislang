pub(super) struct NsldFinalExecutableOutputClosureMirror {
    pub(super) contract: String,
    pub(super) ready: bool,
    pub(super) status: String,
    pub(super) next_action: String,
    pub(super) next_command: Option<String>,
    pub(super) first_blocker: Option<String>,
}

pub(super) fn object_package_summary(
    ready: bool,
    next_command: Option<&str>,
    first_blocker: Option<&str>,
) -> NsldFinalExecutableOutputClosureMirror {
    closure_summary(
        ready,
        "nsld-object-package-summary-v1",
        "replay-ready",
        "replay-blocked",
        "consume-object-package-summary",
        "resolve-object-package-replay-evidence",
        next_command,
        first_blocker,
    )
}

pub(super) fn debugger_transcript_summary(
    ready: bool,
    next_command: Option<&str>,
    first_blocker: Option<&str>,
) -> NsldFinalExecutableOutputClosureMirror {
    closure_summary(
        ready,
        "nsdb-yir-replay-transcript-v1",
        "transcript-ready",
        "transcript-blocked",
        "consume-nsdb-yir-replay-transcript",
        "resolve-nsdb-yir-replay-transcript",
        next_command,
        first_blocker,
    )
}

#[allow(clippy::too_many_arguments)]
fn closure_summary(
    ready: bool,
    contract: &str,
    ready_status: &str,
    blocked_status: &str,
    ready_action: &str,
    blocked_action: &str,
    next_command: Option<&str>,
    first_blocker: Option<&str>,
) -> NsldFinalExecutableOutputClosureMirror {
    NsldFinalExecutableOutputClosureMirror {
        contract: contract.to_owned(),
        ready,
        status: if ready { ready_status } else { blocked_status }.to_owned(),
        next_action: if ready { ready_action } else { blocked_action }.to_owned(),
        next_command: next_command.map(str::to_owned),
        first_blocker: (!ready).then(|| {
            first_blocker
                .unwrap_or("payload-execution-replay:unknown")
                .to_owned()
        }),
    }
}
