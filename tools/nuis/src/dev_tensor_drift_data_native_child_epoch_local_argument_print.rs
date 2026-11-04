use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(super) const CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-borrowed-owner",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/epoch_block_owner.rs",
        required_patterns: &[
            "body: &'a [NirStmt]",
            "epoch: &'a print_epoch::Epoch",
            "std::ptr::eq(self.body, body) && std::ptr::eq(self.epoch, epoch)",
            "let owner = Owner { body, epoch }",
            "Some(&owner)",
            "epoch_local_owner_cannot_move_to_another_block_or_epoch",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-nonowning-branches",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs",
        required_patterns: &[
            "epoch_block_owner::validate_entry(",
            "print_epoch,\n        None,\n        depth,",
            "owner: Option<&epoch_block_owner::Owner<'_>>",
            "owner.is_some_and(|owner| owner.matches(body, print_epoch))",
            "&mut then_locals,", "&mut else_locals,",
            "mod child_epoch_local_argument_print_tests;",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-ready-entry-before-reset",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs",
        required_patterns: &[
            "print_epoch::Epoch::enter_child(", "&locals.available,",
            "child.available.retain(|name| !changed.contains(name))",
            "child_updates.extend(changed.iter().cloned())",
            "epoch_block_owner::validate_entry(", "&print_epoch,", "depth + 1,",
            "locals.available.extend(changed)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-source-refusals",
        path: "tools/nuisc/src/lowering/buffer_loop_outline/control_loops/child_epoch_local_argument_print_tests.rs",
        required_patterns: &[
            "child_epoch_local_source_admits_only_fresh_ready_entries",
            "child_epoch_local_source_rejects_branch_and_ancestor_grants",
            "child_epoch_local_source_keeps_aliases_duplicates_and_hidden_roots_closed",
            "let inherited = stale.replace(", "for bad in [stale, inherited, branch, grandchild]",
            ".replace(\"let epoch_word:\", \"let mut epoch_word:\")",
            "bool_word((initial_guard > 0) && (lazy_guard > 0))",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-nuis-entry-witness",
        path: "tools/nuisc/tests/native_application_bridge/child_epoch_local_argument_print.ns",
        required_patterns: &[
            "while outer < 1", "total = finish(total, 1)",
            "let mut current = Pair { total: total, code: left, flag: selected }",
            "while child < trips", "let epoch_word: i64 = checked64(",
            "current.flag && (checked64(epoch_word, lazy_guard) != 0)",
            "total = current.total", "outer = outer + 1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-native-profile-controls",
        path: "tools/nuisc/tests/native_application_bridge/child_epoch_local_argument_print.rs",
        required_patterns: &[
            "LOOP_LIMIT: u64 = 4", "ENTRY_LIMIT: u64 = 64",
            "child_epoch_local_native_preserves_fresh_entries_and_eager_faults",
            "child_epoch_local_reference_preserves_counts_state_and_faulted_close",
            "child_epoch_local_native_keeps_entry_exhaustion_before_publication",
            "child_epoch_local_native_rejects_branch_alias_and_duplicate_uses",
            "reference_sequences(SOURCE, 1)",
            "entry_exhaustion(SOURCE, LOOP_LIMIT)",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-shared-native-model",
        path: "tools/nuisc/tests/native_application_bridge/epoch_local_argument_print.rs",
        required_patterns: &[
            "pub(super) fn native_sequences(template: &str, loop_limit: u64, entry_limit: u64)",
            "pub(super) fn reference_sequences(template: &str, extra_entry_finishes: usize)",
            "row.base.trips as usize + 1 + extra_entry_finishes",
            "pub(super) fn entry_exhaustion(template: &str, loop_limit: u64)",
            "&template.replace(\"f64\", kind.ty())", "ApplicationSessionPhase::Faulted",
            "cases::sequences(kind).len(), 16", "cases::cli_sequences(kind).len(), 12",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-shared-delivery-profile",
        path: "tools/nuis/tests/native_session_workflow/child_epoch_local_argument_print.rs",
        required_patterns: &[
            "child_epoch_local_argument_print_fresh_cache_and_source_free_delivery",
            "super::epoch_local_argument_print::delivery(",
            "child_epoch_local_argument_print.ns", "4,", "64,",
            "child_epoch_local_argument_print_delivery",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-execution-receipt",
        path: "docs/reference/nuis-native-child-epoch-local-argument-print-v1.md",
        required_patterns: &[
            "Entry Ownership, Not Depth Permission", "parent-print laundering",
            "Twelve finite", "four shared loop-work units", "independent limit-8",
            "repeated parent entry", "Predecessor Lineage",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-child-epoch-local-functional-checkpoint",
        path: "tools/nuis/src/dev_tensor_data.rs",
        required_patterns: &[
            "Fresh child epoch-local argument print witnesses:",
            "Previous root-owned checkpoint:", "Previous root-owned task checkpoint:",
            "Previous root-owned blocker checkpoint:", "Previous root-owned action checkpoint:",
            "Previous root-owned validation checkpoint:", "Previous root-owned artifact checkpoint:",
        ],
    },
];
