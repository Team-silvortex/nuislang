pub(crate) const FINAL_OUTPUT_REPLAY_VOCABULARY_CONTRACT: &str =
    "nuis-final-output-replay-vocabulary-v1";

#[derive(Clone)]
pub(crate) struct FinalOutputReplayVocabulary {
    pub(crate) contract: &'static str,
    pub(crate) source_contract: String,
    pub(crate) ready: bool,
    pub(crate) status: String,
    pub(crate) checkpoint_count: usize,
    pub(crate) replayable_checkpoint_count: usize,
    pub(crate) command: Option<String>,
    pub(crate) next_action: String,
    pub(crate) next_command: Option<String>,
    pub(crate) first_blocker: Option<String>,
}

pub(crate) struct FinalOutputReplayVocabularySource<'a> {
    pub(crate) contract: &'a str,
    pub(crate) ready: bool,
    pub(crate) status: &'a str,
    pub(crate) checkpoint_count: usize,
    pub(crate) replayable_checkpoint_count: usize,
    pub(crate) command: Option<&'a str>,
    pub(crate) next_action: &'a str,
    pub(crate) next_command: Option<&'a str>,
    pub(crate) first_blocker: Option<&'a str>,
}

pub(crate) fn final_output_replay_vocabulary(
    source: FinalOutputReplayVocabularySource<'_>,
) -> FinalOutputReplayVocabulary {
    FinalOutputReplayVocabulary {
        contract: FINAL_OUTPUT_REPLAY_VOCABULARY_CONTRACT,
        source_contract: source.contract.to_owned(),
        ready: source.ready,
        status: source.status.to_owned(),
        checkpoint_count: source.checkpoint_count,
        replayable_checkpoint_count: source.replayable_checkpoint_count,
        command: source.command.map(str::to_owned),
        next_action: source.next_action.to_owned(),
        next_command: source.next_command.map(str::to_owned),
        first_blocker: if source.ready {
            None
        } else {
            Some(
                source
                    .first_blocker
                    .unwrap_or("final-output-replay-vocabulary-source-not-ready")
                    .to_owned(),
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source<'a>(
        ready: bool,
        first_blocker: Option<&'a str>,
    ) -> FinalOutputReplayVocabularySource<'a> {
        FinalOutputReplayVocabularySource {
            contract: "nsdb-payload-execution-replay-plan-v1",
            ready,
            status: if ready {
                "replay-evidence-ready"
            } else {
                "blocked"
            },
            checkpoint_count: 2,
            replayable_checkpoint_count: usize::from(ready) * 2,
            command: ready.then_some("nsdb replay out --json"),
            next_action: if ready {
                "replay-nsdb-payload-execution"
            } else {
                "resolve-final-output-nsdb-replay"
            },
            next_command: Some("nsld final-executable-output out/manifest.toml --json"),
            first_blocker,
        }
    }

    #[test]
    fn vocabulary_preserves_ready_replay_facts_without_reinterpreting_them() {
        let vocabulary = final_output_replay_vocabulary(source(true, None));

        assert_eq!(
            vocabulary.contract,
            "nuis-final-output-replay-vocabulary-v1"
        );
        assert_eq!(
            vocabulary.source_contract,
            "nsdb-payload-execution-replay-plan-v1"
        );
        assert!(vocabulary.ready);
        assert_eq!(vocabulary.checkpoint_count, 2);
        assert_eq!(vocabulary.replayable_checkpoint_count, 2);
        assert!(vocabulary.first_blocker.is_none());
    }

    #[test]
    fn vocabulary_fails_closed_with_the_source_blocker() {
        let vocabulary =
            final_output_replay_vocabulary(source(false, Some("payload:not-observed")));

        assert!(!vocabulary.ready);
        assert_eq!(vocabulary.status, "blocked");
        assert_eq!(
            vocabulary.first_blocker.as_deref(),
            Some("payload:not-observed")
        );
        assert!(vocabulary.command.is_none());
    }
}
