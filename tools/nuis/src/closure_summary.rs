pub(crate) struct FrontdoorClosureSummary {
    pub(crate) source: &'static str,
    pub(crate) status: String,
    pub(crate) ready: bool,
    pub(crate) primary_blocker: Option<String>,
    pub(crate) next_action: String,
    pub(crate) next_command: Option<String>,
}

impl FrontdoorClosureSummary {
    pub(crate) fn from_project_surface(
        source: &'static str,
        artifact_ready: bool,
        tests_missing: usize,
        recommended_next_step: &str,
        recommended_command: &str,
    ) -> Self {
        let ready = artifact_ready && tests_missing == 0;
        let primary_blocker = if tests_missing > 0 {
            Some("declared-tests-missing".to_owned())
        } else if artifact_ready {
            None
        } else {
            Some(recommended_next_step.to_owned())
        };
        Self {
            source,
            status: if ready { "ready" } else { "blocked" }.to_owned(),
            ready,
            primary_blocker,
            next_action: if ready {
                "run-artifact-or-release-check".to_owned()
            } else {
                recommended_next_step.to_owned()
            },
            next_command: if ready {
                None
            } else {
                Some(recommended_command.to_owned())
            },
        }
    }

    pub(crate) fn from_nsld_next_action(
        source: &'static str,
        action: &str,
        command: Option<&str>,
        reason: &str,
    ) -> Self {
        let ready = action == "ready";
        Self {
            source,
            status: if ready { "ready" } else { "blocked" }.to_owned(),
            ready,
            primary_blocker: if ready { None } else { Some(reason.to_owned()) },
            next_action: action.to_owned(),
            next_command: command.map(str::to_owned),
        }
    }

    pub(crate) fn json_fields(&self) -> Vec<String> {
        vec![
            crate::json_field("closure_summary_source", self.source),
            crate::json_field("closure_summary_status", &self.status),
            crate::json_bool_field("closure_summary_ready", self.ready),
            crate::json_optional_string_field(
                "closure_summary_primary_blocker",
                self.primary_blocker.as_deref(),
            ),
            crate::json_field("closure_summary_next_action", &self.next_action),
            crate::json_optional_string_field(
                "closure_summary_next_command",
                self.next_command.as_deref(),
            ),
        ]
    }
}
