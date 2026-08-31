pub(crate) const FRESH_SOURCE_ADAPTER: &str = r#"
static int run_fresh_source(const char* path) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 65;
    int64_t states[5] = {0, 0, 0, 0, 0};
    for (int64_t stage = 0; stage < 5; ++stage) {
        states[stage] = nuis_bootstrap_candidate_stage_seed_v1(10 + stage);
        if (states[stage] < 0) {
            fclose(file);
            return 66;
        }
    }
    int64_t source_bytes = 0;
    for (;;) {
        int byte = fgetc(file);
        if (byte == EOF) break;
        if (source_bytes >= 128) {
            fclose(file);
            return 67;
        }
        for (int64_t stage = 0; stage < 5; ++stage) {
            states[stage] = nuis_bootstrap_candidate_stage_fold_v1(
                states[stage],
                10 + stage,
                (int64_t)byte
            );
            if (states[stage] < 0) {
                fclose(file);
                return 68;
            }
        }
        source_bytes += 1;
    }
    if (ferror(file) != 0) {
        fclose(file);
        return 69;
    }
    if (fclose(file) != 0) return 70;

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
"#;
