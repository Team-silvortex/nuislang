use super::*;

fn heap(id: u8) -> CacheObjectKey {
    CacheObjectKey {
        owner: CacheOwner::Heap,
        content_sha256: [id; 32],
    }
}

fn stack(project: u8, id: u8) -> CacheObjectKey {
    CacheObjectKey {
        owner: CacheOwner::Project([project; 32]),
        content_sha256: [id; 32],
    }
}

fn object(key: CacheObjectKey, dependencies: &[CacheObjectKey]) -> CacheObject {
    CacheObject {
        key,
        bytes: 10,
        dependencies: dependencies.to_vec(),
        unreachable_since_seconds: Some(10),
    }
}

fn snapshot(objects: Vec<CacheObject>, roots: Vec<CacheRoot>) -> CacheSnapshot {
    CacheSnapshot {
        generation: 7,
        captured_at_seconds: 100,
        complete: true,
        objects,
        roots,
    }
}

fn root(kind: RootKind, objects: &[CacheObjectKey]) -> CacheRoot {
    CacheRoot {
        kind,
        objects: objects.to_vec(),
    }
}

fn policy() -> GcPolicy {
    GcPolicy {
        grace_seconds: 30,
        max_heap_bytes: 100,
    }
}

#[test]
fn unmarked_packages_default_to_heap_and_stack_cannot_be_weakened() {
    assert_eq!(PackageLifetime::default(), PackageLifetime::Heap);
    assert_eq!(
        resolve_package_lifetime(None, false),
        Ok(PackageLifetime::Heap)
    );
    assert_eq!(
        resolve_package_lifetime(Some(PackageLifetime::Stack), false),
        Ok(PackageLifetime::Stack)
    );
    assert_eq!(
        resolve_package_lifetime(None, true),
        Ok(PackageLifetime::Stack)
    );
    assert!(resolve_package_lifetime(Some(PackageLifetime::Heap), true).is_err());
}

#[test]
fn each_root_kind_protects_the_complete_dependency_closure() {
    for kind in [
        RootKind::ProjectLock,
        RootKind::BuildLease,
        RootKind::ExplicitPin,
        RootKind::ToolchainPin,
    ] {
        let view = snapshot(
            vec![
                object(heap(1), &[heap(2)]),
                object(heap(2), &[heap(1)]),
                object(heap(3), &[]),
            ],
            vec![root(kind, &[heap(1)])],
        );
        let plan = plan_gc(&view, policy()).unwrap();
        assert_eq!(plan.candidates, vec![heap(3)]);
        assert_eq!(plan.retained_heap_bytes, 20);
        assert_eq!(plan.reclaimable_bytes, 10);
    }
}

#[test]
fn unreachable_cycles_are_collectible_without_reference_count_leaks() {
    let view = snapshot(
        vec![object(heap(1), &[heap(2)]), object(heap(2), &[heap(1)])],
        vec![],
    );
    let plan = plan_gc(&view, policy()).unwrap();
    assert_eq!(plan.candidates, vec![heap(1), heap(2)]);
    assert_eq!(plan.retained_heap_bytes, 0);
    assert_eq!(plan.reclaimable_bytes, 20);
}

#[test]
fn removing_one_project_root_does_not_collect_another_projects_dependencies() {
    let mut view = snapshot(
        vec![object(heap(1), &[heap(2)]), object(heap(2), &[])],
        vec![
            root(RootKind::ProjectLock, &[heap(1)]),
            root(RootKind::ProjectLock, &[heap(2)]),
        ],
    );
    assert!(plan_gc(&view, policy()).unwrap().candidates.is_empty());
    view.roots.remove(0);
    assert_eq!(plan_gc(&view, policy()).unwrap().candidates, vec![heap(1)]);
}

#[test]
fn stack_objects_and_their_heap_dependencies_are_never_gc_candidates() {
    let view = snapshot(
        vec![
            object(stack(1, 1), &[heap(2)]),
            object(heap(1), &[]),
            object(heap(2), &[]),
        ],
        vec![],
    );
    let plan = plan_gc(&view, policy()).unwrap();
    assert_eq!(plan.candidates, vec![heap(1)]);
    assert_eq!(plan.heap_bytes, 20);
    assert_eq!(plan.retained_heap_bytes, 10);
}

#[test]
fn heap_cannot_capture_stack_and_projects_cannot_capture_each_others_stack() {
    for (from, to) in [(heap(1), stack(1, 2)), (stack(1, 1), stack(2, 2))] {
        let view = snapshot(vec![object(from, &[to]), object(to, &[])], vec![]);
        assert_eq!(
            plan_gc(&view, policy()),
            Err(GcPlanError::InvalidOwnershipEdge(Box::new(OwnershipEdge {
                from,
                to,
            })))
        );
    }
    let view = snapshot(
        vec![
            object(stack(1, 1), &[stack(1, 2)]),
            object(stack(1, 2), &[]),
        ],
        vec![],
    );
    assert!(plan_gc(&view, policy()).unwrap().candidates.is_empty());
}

#[test]
fn grace_period_and_unknown_history_also_protect_dependencies() {
    for since in [None, Some(100), Some(71)] {
        let mut view = snapshot(
            vec![object(heap(1), &[heap(2)]), object(heap(2), &[])],
            vec![],
        );
        view.objects[0].unreachable_since_seconds = since;
        assert!(plan_gc(&view, policy()).unwrap().candidates.is_empty());
        view.objects[0].unreachable_since_seconds = Some(70);
        assert_eq!(plan_gc(&view, policy()).unwrap().candidates.len(), 2);
    }
}

#[test]
fn incomplete_registry_or_clock_rollback_never_produces_a_plan() {
    let mut view = snapshot(vec![object(heap(1), &[])], vec![]);
    view.complete = false;
    assert_eq!(
        plan_gc(&view, policy()),
        Err(GcPlanError::IncompleteSnapshot)
    );
    view.complete = true;
    view.objects[0].unreachable_since_seconds = Some(101);
    assert_eq!(
        plan_gc(&view, policy()),
        Err(GcPlanError::FutureUnreachableTime(heap(1)))
    );
}

#[test]
fn duplicate_inventory_and_dangling_edges_or_roots_fail_closed() {
    let view = snapshot(vec![object(heap(1), &[]), object(heap(1), &[])], vec![]);
    assert_eq!(
        plan_gc(&view, policy()),
        Err(GcPlanError::DuplicateObject(heap(1)))
    );
    for view in [
        snapshot(vec![object(heap(1), &[heap(2)])], vec![]),
        snapshot(
            vec![object(heap(1), &[])],
            vec![root(RootKind::BuildLease, &[heap(2)])],
        ),
    ] {
        assert_eq!(
            plan_gc(&view, policy()),
            Err(GcPlanError::MissingObject(heap(2)))
        );
    }
}

#[test]
fn budget_pressure_reports_shortfall_instead_of_evicting_live_content() {
    let view = snapshot(
        vec![object(heap(1), &[]), object(heap(2), &[])],
        vec![root(RootKind::BuildLease, &[heap(1)])],
    );
    let plan = plan_gc(
        &view,
        GcPolicy {
            max_heap_bytes: 0,
            ..policy()
        },
    )
    .unwrap();
    assert_eq!(plan.candidates, vec![heap(2)]);
    assert_eq!(plan.retained_heap_bytes, 10);
    assert_eq!(plan.over_budget_bytes, 10);
}

#[test]
fn byte_accounting_is_checked_and_excludes_project_owned_storage() {
    let mut view = snapshot(vec![object(heap(1), &[]), object(heap(2), &[])], vec![]);
    view.objects[0].bytes = u64::MAX;
    assert_eq!(
        plan_gc(&view, policy()),
        Err(GcPlanError::ByteCountOverflow)
    );
    view.objects[0].key = stack(1, 1);
    assert_eq!(plan_gc(&view, policy()).unwrap().heap_bytes, 10);
}

#[test]
fn plans_are_deterministic_and_do_not_mutate_the_snapshot() {
    let mut view = snapshot(
        vec![
            object(heap(3), &[]),
            object(heap(1), &[]),
            object(heap(2), &[]),
        ],
        vec![root(RootKind::ExplicitPin, &[heap(3)])],
    );
    let before = view.clone();
    let plan = plan_gc(&view, policy()).unwrap();
    assert_eq!(view, before);
    view.objects.reverse();
    assert_eq!(plan_gc(&view, policy()).unwrap(), plan);
    assert_eq!(plan.protocol, CACHE_LIFECYCLE_PROTOCOL);
    assert_eq!(plan.snapshot_generation, 7);
    assert_eq!(plan.candidates, vec![heap(1), heap(2)]);
}

#[test]
fn empty_registry_has_no_candidates_or_budget_shortfall() {
    let plan = plan_gc(&snapshot(vec![], vec![]), policy()).unwrap();
    assert!(plan.candidates.is_empty());
    assert_eq!(plan.heap_bytes, 0);
    assert_eq!(plan.over_budget_bytes, 0);
}

#[test]
fn all_three_object_graphs_match_an_independent_reachability_oracle() {
    for edges in 0_u16..512 {
        let mut reach = [[false; 3]; 3];
        let mut objects = Vec::new();
        for (from, row) in reach.iter_mut().enumerate() {
            let mut dependencies = Vec::new();
            for (to, reachable) in row.iter_mut().enumerate() {
                if edges & (1 << (from * 3 + to)) != 0 {
                    dependencies.push(heap(to as u8 + 1));
                    *reachable = true;
                }
            }
            row[from] = true;
            objects.push(object(heap(from as u8 + 1), &dependencies));
        }
        // Fixed-size transitive closure is independent of the planner's worklist.
        for via in 0..3 {
            for from in 0..3 {
                for to in 0..3 {
                    reach[from][to] |= reach[from][via] && reach[via][to];
                }
            }
        }
        for root_bits in 0_u8..8 {
            let roots = (0..3)
                .filter(|from| root_bits & (1 << from) != 0)
                .map(|from| heap(from + 1))
                .collect::<Vec<_>>();
            let expected = (0..3)
                .filter(|to| !(0..3).any(|from| root_bits & (1 << from) != 0 && reach[from][*to]))
                .map(|to| heap(to as u8 + 1))
                .collect::<Vec<_>>();
            let view = snapshot(objects.clone(), vec![root(RootKind::ProjectLock, &roots)]);
            let plan = plan_gc(&view, policy()).unwrap();
            assert_eq!(
                plan.candidates, expected,
                "edges={edges}, roots={root_bits}"
            );
            assert_eq!(plan.reclaimable_bytes, expected.len() as u64 * 10);
        }
    }
}
