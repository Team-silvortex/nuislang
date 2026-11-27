use super::*;

// Exercise the same three profile reads as the production packet builder,
// without inspecting the opaque packet or submitting any GPU work.
const PROBE: &str = r#"
  struct ProfileSample { color: i64, speed: i64, radius: i64 }
  fn profile_open() -> ProfileSample {
    return ProfileSample {color: 0, speed: 0, radius: 0};
  }
  fn profile_event(state: ProfileSample, frame: i64) -> ProfileSample {
    let color: i64 = shader_profile_color_seed("PixelMagicImageSurface", frame, 0);
    let speed: i64 = shader_profile_speed_seed("PixelMagicImageSurface", frame, 2, 184);
    let radius: i64 = shader_profile_radius_seed("PixelMagicImageSurface", 122 + frame, 0);
    return ProfileSample {color: color, speed: speed, radius: radius};
  }
  fn profile_close(state: ProfileSample) -> ProfileSample { return state; }
  fn window_probe_event(state: ProfileSample, frame: i64) -> ProfileSample {
    let seeds: ProfileSample = profile_event(state, frame);
    let packet: ProfileSample = ProfileSample {color: seeds.color, speed: seeds.speed, radius: seeds.radius};
    let window: Window<ProfileSample> = data_immutable_window(packet, 0, 1);
    let rotated: ProfileSample = ProfileSample {color: seeds.radius, speed: seeds.color, radius: seeds.speed};
    let second: Window<ProfileSample> = data_immutable_window(rotated, 0, 1);
    let sample: ProfileSample = data_read_window(window, 0);
    let other: ProfileSample = data_read_window(second, 0);
    return ProfileSample {color: sample.color, speed: other.radius, radius: sample.radius};
  }
  fn pipe_probe_event(state: ProfileSample, frame: i64) -> ProfileSample {
    let seeds: ProfileSample = profile_event(state, frame);
    let packet: ProfileSample = ProfileSample {color: seeds.radius, speed: seeds.color, radius: seeds.speed};
    let window: Window<ProfileSample> = data_immutable_window(packet, 0, 1);
    let pipe: Pipe<Window<ProfileSample>> = data_output_pipe(window);
    let received: Window<ProfileSample> = data_input_pipe(pipe);
    let sample: ProfileSample = data_read_window(received, 0);
    return ProfileSample {color: sample.color, speed: sample.speed, radius: sample.radius};
  }
  fn frame_pipe_event(state: ProfileSample, frame_index: i64) -> ProfileSample {
    FRAME_PACKET_PREFIX
    return profile_event(state, frame_index);
  }
"#;

struct Fixture(PathBuf);
impl Fixture {
    fn new(suffix: &str) -> Self {
        use std::fs;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/domains/ns_nova_image_showcase");
        let dir =
            std::env::temp_dir().join(format!("nuis-profile-read-{}-{suffix}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let project = Self(dir);
        let source = fs::read_to_string(root.join("main.ns")).unwrap();
        let (before, after) = source.split_once("mod cpu Main {").unwrap();
        let frame = after.split_once("fn render_showcase_frame(").unwrap().1;
        let packet = &frame[frame.find("let packet:").unwrap()..frame.find("let pass:").unwrap()];
        let probe = PROBE.replace("FRAME_PACKET_PREFIX", packet);
        fs::write(
            project.0.join("main.ns"),
            format!("{before}mod cpu Main {{\n{probe}\n{after}"),
        )
        .unwrap();
        let manifest = fs::read_to_string(root.join("nuis.toml")).unwrap();
        fs::write(project.0.join("nuis.toml"), manifest.replacen("application_sessions = [",
            "application_sessions = [\n  \"probe open=profile_open event=profile_event close=profile_close state=state\",\n  \"window_probe open=profile_open event=window_probe_event close=profile_close state=state\",\n  \"pipe_probe open=profile_open event=pipe_probe_event close=profile_close state=state\",\n  \"frame_pipe open=profile_open event=frame_pipe_event close=profile_close state=state\",", 1)).unwrap();
        fs::copy(
            root.join("fabric_plane.ns"),
            project.0.join("fabric_plane.ns"),
        )
        .unwrap();
        project
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(target_os = "macos")]
#[test]
fn native_showcase_seeds_match_embedded_shader_profile_values() {
    compare_registered_states("probe", "native-session-aot-bundle:probe");
}

#[cfg(target_os = "macos")]
#[test]
fn native_inline_packet_window_matches_embedded_at_each_delivery() {
    compare_registered_states("window_probe", "native-session-aot-bundle:window_probe");
}

#[cfg(target_os = "macos")]
#[test]
fn native_pipe_packet_readback_matches_reference_events() {
    compare_registered_states("pipe_probe", "native-session-aot-bundle:pipe_probe");
}

#[cfg(target_os = "macos")]
#[test]
fn native_showcase_frame_pipe_matches_reference_events() {
    compare_registered_states("frame_pipe", "native-session-aot-bundle:frame_pipe");
}

#[cfg(target_os = "macos")]
fn compare_registered_states(id: &str, mode: &str) {
    use std::{fs, process::Command};
    let project = Fixture::new(id);
    let out = project.0.join("out");
    let built = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("compile")
        .arg(&project.0)
        .arg(&out)
        .args(["--packaging-mode", mode])
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_BUILD_JOBS", "1")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let manifest =
        nuisc::aot::verify_build_manifest(&out.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(manifest.packaging_mode, mode);
    let module = yir_syntax::parse_module(
        &fs::read_to_string(out.join(format!("{}.yir", manifest.artifact_binary_name))).unwrap(),
    )
    .unwrap();
    let registry = yir_verify::default_registry();
    if id == "window_probe" {
        for op in ["data.immutable_window", "data.read_window"] {
            assert_eq!(
                module
                    .nodes
                    .iter()
                    .filter(|n| n.op.full_name() == op
                        && module
                            .node_lanes
                            .get(&n.name)
                            .is_some_and(|l| l == "fn:window_probe_event"))
                    .count(),
                2,
                "{op}"
            );
        }
    }
    let (mut session, _) =
        ApplicationSession::open_registered(&module, &registry, id, vec![]).unwrap();
    let mut expected = vec![format!("open:{}", session.state())];
    let mut timeline = vec![format!("state=open:{}", session.state())];
    let mut native = Command::new(out.join(manifest.artifact_binary_name));
    native
        .current_dir(&out)
        .env("PATH", "")
        .args(["--application-session", id, "--open-args", ""]);
    let pipe_probe = matches!(id, "pipe_probe" | "frame_pipe");
    let frames: &[i64] = if pipe_probe {
        &[0, 3, 3, 1]
    } else {
        &[0, 3, 1]
    };
    let mut payloads = BTreeSet::new();
    for &frame in frames {
        native.args(["--event-args", &frame.to_string()]);
        let trace = session.event(vec![Value::Int(frame)]).unwrap();
        let effects: Vec<_> = trace
            .events
            .iter()
            .filter(|s| {
                s.starts_with("effect data.output_pipe ")
                    || s.starts_with("effect data.input_pipe ")
            })
            .collect();
        if pipe_probe {
            assert_eq!(effects.len(), 2, "{effects:?}");
            assert!(effects[0].starts_with("effect data.output_pipe "));
            assert!(effects[1].starts_with("effect data.input_pipe "));
            let sent = effects[0].split_once(": ").unwrap().1;
            let received = effects[1].split_once(": ").unwrap().1;
            payloads.insert(received.to_owned());
            assert_eq!(sent, format!("pipe[output {received}]"));
            if id == "pipe_probe" {
                assert_eq!(
                    received,
                    format!("window[immutable offset=0 len=1 base={}]", session.state())
                );
            }
            assert!(
                effects[1].contains(if id == "frame_pipe" {
                    "PixelMagicImageSurfacePacket"
                } else {
                    "ProfileSample"
                }),
                "{effects:?}"
            );
        } else {
            assert!(effects.is_empty());
        }
        timeline.extend(effects.into_iter().map(|e| format!("data_event={e}")));
        timeline.push(format!("state=event:{}", session.state()));
        assert!(field(session.state(), "speed") > 0);
        expected.push(format!("event:{}", session.state()));
    }
    if pipe_probe {
        assert_eq!(payloads.len(), 3);
    }
    assert_eq!(expected[1..].iter().collect::<BTreeSet<_>>().len(), 3);
    session.close(vec![]).unwrap().unwrap();
    session.completion_status().unwrap();
    expected.push(format!("close:{}", session.state()));
    timeline.push(format!("state=close:{}", session.state()));
    let run = native.args(["--close-args", ""]).output().unwrap();
    let log = String::from_utf8(run.stderr).unwrap();
    assert!(run.status.success(), "{log}");
    assert_eq!(
        log.lines()
            .filter_map(|l| l.strip_prefix("nuis: native_session_"))
            .filter(|s| s.starts_with("state=") || s.starts_with("data_event="))
            .collect::<Vec<_>>(),
        timeline
    );
    assert_eq!(
        log.lines()
            .filter_map(|s| s.strip_prefix("nuis: native_session_state="))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        log.lines()
            .filter(|s| *s == "nuis: native_session_completed=1")
            .count(),
        1
    );
}

#[test]
fn native_pipe_requires_a_single_ordered_local_receiver() {
    use yir_core::{Edge, EdgeKind};
    use yir_lower_llvm::native_session::emit_registered;
    let fixture = Fixture::new("pipe-admission");
    let m = nuisc::pipeline::compile_project(&fixture.0).unwrap().yir;
    let name = |op| {
        m.nodes
            .iter()
            .find(|n| {
                n.op.full_name() == op
                    && m.node_lanes
                        .get(&n.name)
                        .is_some_and(|l| l == "fn:pipe_probe_event")
            })
            .unwrap()
            .name
            .clone()
    };
    let send = name("data.output_pipe");
    let receive = name("data.input_pipe");
    let ir = emit_registered(&m, "pipe_probe").unwrap().llvm_ir;
    assert_eq!(
        ir.matches("call i32 @nuis_native_data_pipe_event_v1(")
            .count(),
        2
    );
    let window = name("data.immutable_window");
    let mut bare = m.clone();
    let record = m.nodes.iter().find(|n| n.name == window).unwrap().op.args[0].clone();
    bare.nodes
        .iter_mut()
        .find(|n| n.name == send)
        .unwrap()
        .op
        .args = vec![record.clone()];
    bare.edges.push(Edge {
        from: record,
        to: send.clone(),
        kind: EdgeKind::CrossDomainExchange,
    });
    let err = emit_registered(&bare, "pipe_probe").unwrap_err();
    assert!(
        err.contains("expects window input for read_window"),
        "{err}"
    );
    let mut unordered = m.clone();
    unordered
        .edges
        .retain(|e| !(e.from == send && e.to == receive && e.kind == EdgeKind::Effect));
    let err = emit_registered(&unordered, "pipe_probe").unwrap_err();
    assert!(err.contains("explicit effect edge"), "{err}");
    let mut branched = m.clone();
    let mut duplicate = m.nodes.iter().find(|n| n.name == receive).unwrap().clone();
    duplicate.name = "second_pipe_receiver".into();
    branched
        .node_lanes
        .insert(duplicate.name.clone(), "fn:pipe_probe_event".into());
    branched
        .functions
        .iter_mut()
        .find(|f| f.name == "pipe_probe_event")
        .unwrap()
        .body_nodes
        .push(duplicate.name.clone());
    for kind in [EdgeKind::Dep, EdgeKind::Effect] {
        branched.edges.push(Edge {
            kind,
            from: send.clone(),
            to: duplicate.name.clone(),
        });
    }
    branched.nodes.push(duplicate);
    let err = emit_registered(&branched, "pipe_probe").unwrap_err();
    assert!(err.contains("single receiver"), "{err}");
}

#[test]
fn inline_window_bounds_types_and_handoffs_fail_closed() {
    use yir_core::{Edge, EdgeKind};
    use yir_lower_llvm::native_session::emit_registered;
    let fixture = Fixture::new("window-bounds");
    let mut m = nuisc::pipeline::compile_project(&fixture.0).unwrap().yir;
    let lane = "fn:window_probe_event";
    let site = |op: &str| {
        m.nodes
            .iter()
            .position(|n| {
                n.op.full_name() == op && m.node_lanes.get(&n.name).is_some_and(|l| l == lane)
            })
            .unwrap()
    };
    let window = site("data.immutable_window");
    let read = site("data.read_window");
    let name = m.nodes[window].name.clone();
    let packet = m.nodes[window].op.args[0].clone();
    let offset = "project_profile_data_FabricPlane_window_offset";
    let len = "project_profile_data_FabricPlane_uplink_len";
    for (index, src) in [(1, offset), (2, len)] {
        m.nodes[window].op.args[index] = src.into();
        m.edges.push(Edge {
            from: src.into(),
            to: name.clone(),
            kind: EdgeKind::CrossDomainExchange,
        });
    }
    emit_registered(&m, "window_probe").unwrap();
    let mut invalid = m.clone();
    invalid.nodes[window].op.args[2] = "2".into();
    assert!(emit_registered(&invalid, "window_probe")
        .unwrap_err()
        .contains("constant length 1"));

    let mut invalid = m.clone();
    let frame = &m
        .functions
        .iter()
        .find(|f| f.name == "window_probe_event")
        .unwrap()
        .parameters
        .last()
        .unwrap()
        .node;
    invalid.nodes[read].op.args[1] = frame.clone();
    invalid.edges.push(Edge {
        from: frame.clone(),
        to: m.nodes[read].name.clone(),
        kind: EdgeKind::CrossDomainExchange,
    });
    assert!(emit_registered(&invalid, "window_probe")
        .unwrap_err()
        .contains("requires constant"));

    for kind in [EdgeKind::Lifetime, EdgeKind::Effect] {
        let mut invalid = m.clone();
        invalid.edges.push(Edge {
            from: offset.into(),
            to: name.clone(),
            kind,
        });
        assert!(emit_registered(&invalid, "window_probe")
            .unwrap_err()
            .contains("rejects external"));
    }

    let mut invalid = m.clone();
    let mut flag = invalid
        .nodes
        .iter()
        .find(|n| n.name == offset)
        .unwrap()
        .clone();
    flag.name = "window_non_i64_field".into();
    flag.op.instruction = "const_bool".into();
    flag.op.args = vec!["true".into()];
    let base = invalid.nodes.iter_mut().find(|n| n.name == packet).unwrap();
    base.op.args[1] = format!("color={}", flag.name);
    invalid.edges.push(Edge {
        from: flag.name.clone(),
        to: packet,
        kind: EdgeKind::Dep,
    });
    invalid.node_lanes.insert(flag.name.clone(), lane.into());
    invalid
        .functions
        .iter_mut()
        .find(|f| f.name == "window_probe_event")
        .unwrap()
        .body_nodes
        .push(flag.name.clone());
    invalid.nodes.push(flag);
    let err = emit_registered(&invalid, "window_probe").unwrap_err();
    assert!(err.contains("flat packet of distinct i64 fields"), "{err}");

    let err = emit_registered(&m, "window").unwrap_err();
    assert!(
        err.contains("input_pipe") && err.contains("cannot hand off to shader.uniform_binding"),
        "{err}"
    );
}

#[test]
fn profile_read_refuses_nonliteral_or_ordered_ingress() {
    use yir_core::{Edge, EdgeKind};
    use yir_lower_llvm::native_session::emit_registered;
    let fixture = Fixture::new("ingress");
    let module = nuisc::pipeline::compile_project(&fixture.0).unwrap().yir;
    let source = "project_profile_shader_PixelMagicImageSurface_packet_color_slot";
    let src = module.nodes.iter().position(|n| n.name == source).unwrap();
    let reader = module
        .nodes
        .iter()
        .find(|n| {
            n.op.instruction == "add"
                && n.op.args.iter().any(|a| a == source)
                && module
                    .node_lanes
                    .get(&n.name)
                    .is_some_and(|l| l == "fn:profile_event")
        })
        .unwrap();
    let other = reader
        .op
        .args
        .iter()
        .find(|a| a.as_str() != source)
        .unwrap();
    let dep = module
        .edges
        .iter()
        .position(|e| e.from == source && e.to == reader.name && e.kind == EdgeKind::Dep)
        .unwrap();
    emit_registered(&module, "probe").unwrap();
    let reject = |m: &YirModule| {
        assert!(
            emit_registered(m, "probe").is_err(),
            "external input was admitted"
        )
    };
    let reject_external = |m: &YirModule, external: &str| {
        let err = emit_registered(m, "probe").unwrap_err();
        assert!(err.contains(&format!("depends on `{external}`")), "{err}");
    };
    for kind in [
        EdgeKind::Effect,
        EdgeKind::Lifetime,
        EdgeKind::CrossDomainExchange,
    ] {
        let mut m = module.clone();
        m.edges.push(Edge {
            from: source.into(),
            to: reader.name.clone(),
            kind: kind.clone(),
        });
        reject(&m);
        let mut m = module.clone();
        m.edges.push(Edge {
            from: other.clone(),
            to: source.into(),
            kind,
        });
        reject(&m);
    }
    let mut m = module.clone();
    m.edges.remove(dep);
    reject(&m);
    // This is a real immutable literal with an ordinary Dep, but it is
    // not an operand of this addition. Import permission is per read.
    let mut m = module.clone();
    let non_operand = "project_profile_shader_PixelMagicImageSurface_packet_speed_slot";
    assert!(m.nodes.iter().any(|n| n.name == non_operand));
    assert!(!reader.op.args.iter().any(|a| a == non_operand));
    m.edges.push(Edge {
        from: non_operand.into(),
        to: reader.name.clone(),
        kind: EdgeKind::Dep,
    });
    reject_external(&m, non_operand);
    let mut m = module.clone();
    m.node_lanes.insert(source.into(), "fn:parent_open".into());
    m.functions
        .iter_mut()
        .find(|f| f.name == "parent_open")
        .unwrap()
        .body_nodes
        .push(source.into());
    reject(&m);
    let mut m = module.clone();
    let mut buffer = m.nodes[src].clone();
    buffer.name = "profile_probe_buffer".into();
    buffer.op.instruction = "alloc_buffer".into();
    buffer.op.args = vec![non_operand.into(), non_operand.into()];
    m.node_lanes.insert(buffer.name.clone(), "profile".into());
    for (from, to) in [
        (non_operand, buffer.name.as_str()),
        (buffer.name.as_str(), source),
        (non_operand, source),
    ] {
        m.edges.push(Edge {
            from: from.into(),
            to: to.into(),
            kind: EdgeKind::Dep,
        });
    }
    m.nodes[src].op.instruction = "load_at".into();
    m.nodes[src].op.args = vec![buffer.name.clone(), non_operand.into()];
    m.nodes.push(buffer);
    reject_external(&m, source);
    let mut m = module.clone();
    m.nodes[src].op.instruction = "print".into();
    m.nodes[src].op.args = vec![non_operand.into()];
    m.edges.push(Edge {
        from: non_operand.into(),
        to: source.into(),
        kind: EdgeKind::Dep,
    });
    reject_external(&m, source);
    let mut m = module.clone();
    m.nodes[src].op.args = vec!["9223372036854775808".into()];
    reject(&m);
}
