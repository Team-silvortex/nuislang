#[cfg(test)]
mod tests {
    use super::super::{normalize_inline_wgsl_source, summarize_inline_wgsl_source};

    #[test]
    fn normalizes_top_level_stage_blocks_into_standard_wgsl_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

stage vertex {
  fn vs_main() -> VsOut {
    var out: VsOut;
    return out;
  }
}

stage fragment {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("stage blocks normalize");

        assert!(normalized.contains("@vertex"), "{normalized}");
        assert!(normalized.contains("@fragment"), "{normalized}");
        assert!(!normalized.contains("stage vertex"), "{normalized}");
        assert!(!normalized.contains("stage fragment"), "{normalized}");
    }

    #[test]
    fn normalizes_compute_stage_workgroup_size_metadata() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage compute(workgroup_size(8, 4, 1)) {
  fn cs_main() {
  }
}
"#,
        )
        .expect("compute stage metadata normalizes");

        assert!(normalized.contains("@compute"), "{normalized}");
        assert!(
            normalized.contains("@workgroup_size(8, 4, 1)"),
            "{normalized}"
        );
        assert!(!normalized.contains("stage compute"), "{normalized}");
    }

    #[test]
    fn normalizes_fragment_stage_metadata_lists() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage fragment(early_depth_test) {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("fragment stage metadata normalizes");

        assert!(normalized.contains("@fragment"), "{normalized}");
        assert!(normalized.contains("@early_depth_test"), "{normalized}");
        assert!(!normalized.contains("stage fragment"), "{normalized}");
    }

    #[test]
    fn normalizes_multiple_stage_metadata_entries() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage compute(workgroup_size(8, 4, 1)) {
  fn cs_main() {
  }
}

stage fragment(early_depth_test) {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("multiple stage metadata entries normalize");

        assert!(
            normalized.contains("@workgroup_size(8, 4, 1)"),
            "{normalized}"
        );
        assert!(normalized.contains("@early_depth_test"), "{normalized}");
    }

    #[test]
    fn normalizes_top_level_binding_declarations() {
        let normalized = normalize_inline_wgsl_source(
            r#"
binding(0, 0) var color_sampler: sampler;
binding(0, 1) var color_tex: texture_2d<f32>;
"#,
        )
        .expect("binding declarations normalize");

        assert!(normalized.contains("@group(0)\n@binding(0)\nvar color_sampler: sampler;"));
        assert!(normalized.contains("@group(0)\n@binding(1)\nvar color_tex: texture_2d<f32>;"));
        assert!(!normalized.contains("binding(0, 0)"), "{normalized}");
    }

    #[test]
    fn normalizes_binding_declarations_with_uniform_generics() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct Globals {
  exposure: f32,
};

binding(0, 2) var<uniform> globals: Globals;
"#,
        )
        .expect("uniform binding declaration normalizes");

        assert!(normalized.contains("@group(0)\n@binding(2)\nvar<uniform> globals: Globals;"));
    }

    #[test]
    fn normalizes_bare_builtin_and_location_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  builtin(position) pos: vec4<f32>,
  location(0) uv: vec2<f32>,
};

stage vertex {
  fn vs_main(builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.uv = vec2<f32>(f32(vid), 0.0);
    return out;
  }
}

stage fragment {
  fn fs_main(location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
  }
}
"#,
        )
        .expect("builtin/location attributes normalize");

        assert!(
            normalized.contains("@builtin(position) pos: vec4<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("@location(0) uv: vec2<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("fn vs_main(@builtin(vertex_index) vid: u32)"),
            "{normalized}"
        );
        assert!(
            normalized.contains("fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  builtin(position)"),
            "{normalized}"
        );
        assert!(!normalized.contains("\n  location(0)"), "{normalized}");
    }

    #[test]
    fn normalizes_bare_interpolate_and_invariant_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  invariant builtin(position) pos: vec4<f32>,
  interpolate(flat) location(0) uv: vec2<f32>,
};

stage fragment {
  fn fs_main(interpolate(flat) location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
  }
}
"#,
        )
        .expect("interpolate/invariant attributes normalize");

        assert!(
            normalized.contains("@invariant @builtin(position) pos: vec4<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("@interpolate(flat) @location(0) uv: vec2<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains(
                "fn fs_main(@interpolate(flat) @location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"
            ),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  invariant builtin(position)"),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  interpolate(flat) location(0)"),
            "{normalized}"
        );
    }

    #[test]
    fn summarizes_inline_wgsl_stages_and_bindings_after_normalization() {
        let summary = summarize_inline_wgsl_source(
            r#"
binding(0, 0) var color_sampler: sampler;
binding(0, 1) var color_tex: texture_2d<f32>;
binding(1, 0) var<uniform> globals: Globals;
binding(1, 1) var<storage, read_write> pixels: array<u32>;

stage vertex {
  fn vs_main() -> VsOut {
    var out: VsOut;
    return out;
  }
}

stage fragment {
  fn fs_main(location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
  }
}

stage compute(workgroup_size(8, 4, 1)) {
  fn cs_main() {
  }
}
"#,
        )
        .expect("summary should parse normalized inline wgsl");

        assert_eq!(summary.schema, "nuis-inline-wgsl-summary-v1");
        assert!(summary.has_stage("vertex"));
        assert!(summary.has_stage("fragment"));
        assert!(summary.has_stage("compute"));
        assert_eq!(
            summary
                .stages
                .iter()
                .find(|stage| stage.stage == "compute")
                .and_then(|stage| stage.workgroup_size.as_deref()),
            Some("8, 4, 1")
        );
        assert_eq!(
            summary
                .stages
                .iter()
                .find(|stage| stage.stage == "fragment")
                .and_then(|stage| stage.return_type.as_deref()),
            Some("vec4<f32>")
        );
        assert_eq!(summary.bindings.len(), 4);
        assert!(summary.bindings.iter().any(|binding| {
            binding.group == 0
                && binding.binding == 0
                && binding.name == "color_sampler"
                && binding.kind == "sampler"
        }));
        assert!(summary.bindings.iter().any(|binding| {
            binding.group == 1
                && binding.binding == 0
                && binding.name == "globals"
                && binding.address_space.as_deref() == Some("uniform")
                && binding.kind == "uniform"
        }));
        assert!(summary.bindings.iter().any(|binding| {
            binding.group == 1
                && binding.binding == 1
                && binding.name == "pixels"
                && binding.address_space.as_deref() == Some("storage, read_write")
                && binding.kind == "storage"
        }));
    }

    #[test]
    fn summary_ignores_stage_markers_inside_comments() {
        let summary = summarize_inline_wgsl_source(
            r#"
// @vertex fn fake_vs() {}
/*
@fragment
fn fake_fs() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
*/
"#,
        )
        .expect("comment-only source should still parse");

        assert!(!summary.has_stage("vertex"));
        assert!(!summary.has_stage("fragment"));
        assert!(summary.stages.is_empty());
    }
}
