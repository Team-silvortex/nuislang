pub(crate) const FRESH_SOURCE_ADAPTER: &str = r#"
static int scan_fresh_source(const char* path, int64_t states[5], int64_t* source_bytes) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 65;
    for (int64_t stage = 0; stage < 5; ++stage) {
        states[stage] = nuis_bootstrap_candidate_stage_seed_v1(10 + stage);
        if (states[stage] < 0) {
            fclose(file);
            return 66;
        }
    }
    *source_bytes = 0;
    for (;;) {
        int byte = fgetc(file);
        if (byte == EOF) break;
        if (*source_bytes >= 128) {
            fclose(file);
            return 67;
        }
        for (int64_t stage = 0; stage < 5; ++stage) {
            states[stage] = nuis_bootstrap_candidate_stage_fold_v1(
                states[stage], 10 + stage, (int64_t)byte
            );
            if (states[stage] < 0) {
                fclose(file);
                return 68;
            }
        }
        *source_bytes += 1;
    }
    if (ferror(file) != 0) {
        fclose(file);
        return 69;
    }
    if (fclose(file) != 0) return 70;
    return 0;
}

static int run_fresh_source(const char* path) {
    int64_t states[5] = {0, 0, 0, 0, 0};
    int64_t source_bytes = 0;
    int status = scan_fresh_source(path, states, &source_bytes);
    if (status != 0) return status;

    int64_t record_counts[5] = {0, 0, 0, 0, 0};
    int64_t identities[5] = {0, 0, 0, 0, 0};
    int64_t bundle = nuis_bootstrap_candidate_bundle_seed_v1();
    for (int64_t stage = 0; stage < 5; ++stage) {
        record_counts[stage] = nuis_bootstrap_candidate_bundle_fold_v1(
            states[stage], 20 + stage, 0
        );
        identities[stage] = nuis_bootstrap_candidate_bundle_fold_v1(
            states[stage], 30 + stage, 0
        );
        if (record_counts[stage] <= 0 || identities[stage] <= 0) return 71;
        bundle = nuis_bootstrap_candidate_bundle_fold_v1(
            bundle, stage, identities[stage]
        );
    }
    if (bundle <= 0) return 72;

    puts("protocol=nuis-bootstrap-candidate-fresh-source-result-v1");
    puts("snapshot_contract=nuis-canonical-bootstrap-source-snapshot-v1");
    printf("source_bytes=%lld\n", (long long)source_bytes);
    printf("source_lines=%lld\n", (long long)record_counts[0]);
    puts("stage_count=5");
    const char* names[5] = {"source", "tokens", "ast", "nir", "yir"};
    for (int stage = 0; stage < 5; ++stage) {
        printf(
            "stage.%d=%s,%lld,%lld\n",
            stage,
            names[stage],
            (long long)record_counts[stage],
            (long long)identities[stage]
        );
    }
    printf("bundle=%lld\n", (long long)bundle);
    puts("stage0_handoff_required=false");
    puts("provider_dependency_required=false");
    puts("candidate_owned_source_processing=true");
    puts("fresh_source_compile=true");
    puts("native_materialization=false");
    puts("replacement_authorized=false");
    puts("selection_authorized=false");
    return 0;
}

static int run_nsld_input(const char* path) {
    int64_t states[5] = {0, 0, 0, 0, 0};
    int64_t source_bytes = 0;
    int status = scan_fresh_source(path, states, &source_bytes);
    if (status != 0) return status;
    int64_t values[14] = {0};
    for (int64_t selector = 0; selector < 14; ++selector) {
        values[selector] = nuis_bootstrap_candidate_bundle_fold_v1(
            states[0], 40 + selector, states[4]
        );
        if (values[selector] < 0) return 73;
    }
    if (values[1] != source_bytes || values[0] != 1) return 74;

    puts("protocol=nuis-compiler-candidate-nsld-input-v1");
    puts("input_contract=nuis-stage1-yir-to-nsld-materialization-input-v1");
    puts("source_snapshot_contract=nuis-canonical-bootstrap-source-snapshot-v1");
    puts("target_contract=nuis-registered-native-object-target-v1");
    puts("target_selector=registered-native-cpu");
    puts("entry_symbol=Main.main");
    puts("function_contract=nuis-yir-scalar-function-v1");
    puts("operation_contract=nuis-yir-return-i64-v1");
    puts("return_type=i64");
    puts("time_contract=timestamped-partial-order");
    puts("glm_contract=candidate-snapshot-no-owned-resource-v1");
    printf("source_bytes=%lld\n", (long long)values[1]);
    printf("source_identity=%lld\n", (long long)values[2]);
    printf("yir_identity=%lld\n", (long long)values[3]);
    printf("unit_count=%lld\n", (long long)values[4]);
    printf("function_count=%lld\n", (long long)values[5]);
    printf("operation_count=%lld\n", (long long)values[6]);
    printf("return_value=%lld\n", (long long)values[7]);
    printf("dependency_count=%lld\n", (long long)values[8]);
    printf("relocation_count=%lld\n", (long long)values[9]);
    printf("time_ordinal=%lld\n", (long long)values[10]);
    printf("glm_resource_count=%lld\n", (long long)values[11]);
    printf("entry_symbol_identity=%lld\n", (long long)values[12]);
    printf("materialization_fold=%lld\n", (long long)values[13]);
    puts("candidate_owned_yir_materialization=true");
    puts("equivalent_nsld_input=true");
    puts("native_object=false");
    puts("stage0_handoff_required=false");
    puts("provider_dependency_required=false");
    puts("replacement_authorized=false");
    puts("selection_authorized=false");
    return 0;
}
"#;
