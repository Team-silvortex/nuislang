use super::{
    json_bool_field, json_optional_string_field, json_usize_field,
    link_plan_final_output_replay_vocabulary::FinalOutputReplayVocabulary,
};

pub(super) fn final_output_replay_vocabulary_json_fields(
    vocabulary: Option<&FinalOutputReplayVocabulary>,
) -> Vec<String> {
    let mut fields =
        vocabulary_projection_json_fields("nsld_final_executable_output_replay", vocabulary);
    fields.extend(vocabulary_projection_json_fields(
        "nsld_final_executable_output_object_package_replay",
        vocabulary,
    ));
    fields.extend(vocabulary_projection_json_fields(
        "nsld_final_executable_output_debugger_transcript_replay",
        vocabulary,
    ));
    fields
}

fn vocabulary_projection_json_fields(
    prefix: &str,
    vocabulary: Option<&FinalOutputReplayVocabulary>,
) -> Vec<String> {
    vec![
        json_optional_string_field(
            &format!("{prefix}_vocabulary_contract"),
            vocabulary.map(|value| value.contract),
        ),
        json_optional_string_field(
            &format!("{prefix}_source_contract"),
            vocabulary.map(|value| value.source_contract.as_str()),
        ),
        json_bool_field(
            &format!("{prefix}_ready"),
            vocabulary.is_some_and(|value| value.ready),
        ),
        json_optional_string_field(
            &format!("{prefix}_status"),
            vocabulary.map(|value| value.status.as_str()),
        ),
        json_usize_field(
            &format!("{prefix}_checkpoint_count"),
            vocabulary.map_or(0, |value| value.checkpoint_count),
        ),
        json_usize_field(
            &format!("{prefix}_replayable_checkpoint_count"),
            vocabulary.map_or(0, |value| value.replayable_checkpoint_count),
        ),
        json_optional_string_field(
            &format!("{prefix}_command"),
            vocabulary.and_then(|value| value.command.as_deref()),
        ),
        json_optional_string_field(
            &format!("{prefix}_next_action"),
            vocabulary.map(|value| value.next_action.as_str()),
        ),
        json_optional_string_field(
            &format!("{prefix}_next_command"),
            vocabulary.and_then(|value| value.next_command.as_deref()),
        ),
        json_optional_string_field(
            &format!("{prefix}_first_blocker"),
            vocabulary.and_then(|value| value.first_blocker.as_deref()),
        ),
    ]
}
