//! Public-contract tests: all normal fixtures are immutable, stateless plugins.
//! FaultControl uses external AtomicBool switches solely for deliberate native
//! nonconformance/failure injection; this is not an example of a conforming plugin.
use ge4g_pentomino::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn id(s: &str) -> PluginId {
    PluginId::new(s).unwrap()
}
fn cap(s: &str) -> CapabilityId {
    CapabilityId::new(s).unwrap()
}
fn v() -> Version {
    Version {
        major: 1,
        minor: 0,
        patch: 0,
    }
}
fn descriptor(name: &str) -> PluginDescriptor {
    PluginDescriptor {
        id: id(name),
        release: v(),
        contract: v(),
        provides: BTreeMap::new(),
        requires: BTreeMap::new(),
        resources: BTreeSet::from(["n".into(), "seen".into()]),
        event_kinds: BTreeSet::from(["pulse".into()]),
    }
}
fn error<T>(result: Result<T, Error>, expected: ErrorCode) {
    match result {
        Err(e) => assert_eq!(e.code(), expected, "{}", e.detail()),
        Ok(_) => panic!("expected {expected:?}"),
    }
}
#[derive(Clone)]
enum Behavior {
    Quiet,
    Pulse,
    Reader(PluginId),
    DeniedRead(PluginId),
    PoisonSet,
    PoisonEmit,
    PoisonBudget,
    WriteDeclared,
    RecordTick,
    Sets(usize),
    Emits {
        init: usize,
        tick: usize,
    },
    InitFail,
    FaultControl {
        init: Arc<AtomicBool>,
        tick: Arc<AtomicBool>,
    },
}
struct Fixture {
    desc: PluginDescriptor,
    behavior: Behavior,
}
impl Plugin for Fixture {
    fn descriptor(&self) -> PluginDescriptor {
        self.desc.clone()
    }
    fn initialize(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error> {
        match &self.behavior {
            Behavior::Pulse | Behavior::FaultControl { .. } | Behavior::InitFail => {
                tx.set("n", 0)?;
                tx.draw()?;
                tx.emit("pulse", 7)?;
                if matches!(self.behavior, Behavior::InitFail) {
                    return Err(Error::new(
                        ErrorCode::PluginFailed,
                        "deliberate init failure",
                    ));
                }
                if let Behavior::FaultControl { init, .. } = &self.behavior
                    && init.load(Ordering::SeqCst)
                {
                    return Err(Error::new(ErrorCode::PluginFailed, "injected init failure"));
                }
            }
            Behavior::Emits { init, .. } => {
                for i in 0..*init {
                    tx.emit("pulse", i as i64)?;
                }
            }
            Behavior::WriteDeclared => {
                for key in &self.desc.resources {
                    tx.set(key, i64::MIN)?;
                }
            }
            Behavior::RecordTick => {
                tx.set("n", ctx.tick() as i64)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn tick(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error> {
        match &self.behavior {
            Behavior::Quiet | Behavior::InitFail => {}
            Behavior::Pulse | Behavior::FaultControl { .. } => {
                let n = ctx
                    .own("n")?
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| Error::new(ErrorCode::Overflow, "counter"))?;
                tx.set("n", n)?;
                let draw = tx.draw()?;
                tx.emit("pulse", (draw & 0x7fff_ffff_ffff_ffff) as i64)?;
                if let Behavior::FaultControl { tick, .. } = &self.behavior
                    && tick.load(Ordering::SeqCst)
                {
                    return Err(Error::new(ErrorCode::PluginFailed, "injected tick failure"));
                }
            }
            Behavior::Reader(provider) => {
                tx.set("n", ctx.read(provider, "n")?.unwrap_or(-1))?;
                let total = ctx.events(provider)?.iter().try_fold(0_i64, |sum, event| {
                    sum.checked_add(event.value)
                        .ok_or_else(|| Error::new(ErrorCode::Overflow, "sum"))
                })?;
                tx.set("seen", total)?;
            }
            Behavior::DeniedRead(provider) => {
                ctx.read(provider, "n")?;
            }
            Behavior::PoisonSet => {
                tx.set("n", 99)?;
                let _ = tx.set("undeclared", 1);
            }
            Behavior::PoisonEmit => {
                tx.set("n", 99)?;
                let _ = tx.emit("undeclared", 1);
            }
            Behavior::PoisonBudget => {
                for _ in 0..1025 {
                    let _ = tx.draw();
                }
            }
            Behavior::WriteDeclared => {
                for key in &self.desc.resources {
                    tx.set(key, i64::MAX)?;
                }
            }
            Behavior::RecordTick => {
                tx.set("n", ctx.tick() as i64)?;
            }
            Behavior::Sets(count) => {
                for i in 0..*count {
                    tx.set("n", i as i64)?;
                }
            }
            Behavior::Emits { tick, .. } => {
                tx.set("n", ctx.tick() as i64)?;
                tx.draw()?;
                for i in 0..*tick {
                    tx.emit("pulse", i as i64)?;
                }
            }
        }
        Ok(())
    }
}
fn install(host: &mut Host, name: &str, behavior: Behavior) {
    host.install(
        Box::new(Fixture {
            desc: descriptor(name),
            behavior,
        }),
        BTreeMap::new(),
    )
    .unwrap();
}
fn observe(host: &Host, name: &str) -> Observation {
    host.observe(host.owner(&id(name)).unwrap()).unwrap()
}
fn host() -> Host {
    Host::new(42, "test:content-v1").unwrap()
}
type Projection = (
    u64,
    BTreeMap<String, i64>,
    u64,
    Vec<(u64, String, String, i64)>,
);
fn projection(host: &Host, name: &str) -> Projection {
    let o = observe(host, name);
    (
        o.tick,
        o.resources,
        o.rng_state,
        o.events
            .into_iter()
            .map(|e| (e.tick, e.owner.as_str().into(), e.kind, e.value))
            .collect(),
    )
}
fn provider_desc(name: &str) -> PluginDescriptor {
    let mut d = descriptor(name);
    d.provides.insert(cap("test.counter"), v());
    d
}
fn install_provider(host: &mut Host, name: &str) {
    host.install(
        Box::new(Fixture {
            desc: provider_desc(name),
            behavior: Behavior::Pulse,
        }),
        BTreeMap::new(),
    )
    .unwrap();
}
fn reader_desc() -> PluginDescriptor {
    let mut d = descriptor("test.reader");
    d.requires.insert(cap("test.counter"), v());
    d
}
fn reader_bindings(provider: &str) -> Bindings {
    BTreeMap::from([(cap("test.counter"), id(provider))])
}
fn install_reader(host: &mut Host, provider: &str) {
    host.install(
        Box::new(Fixture {
            desc: reader_desc(),
            behavior: Behavior::Reader(id(provider)),
        }),
        reader_bindings(provider),
    )
    .unwrap();
}

#[test]
fn independent_unload_reinstall_preserves_equal_tick_projection() {
    let mut combined = host();
    let mut b_only = host();
    install(&mut combined, "test.a", Behavior::Pulse);
    install(&mut combined, "test.b", Behavior::Pulse);
    install(&mut b_only, "test.b", Behavior::Pulse);
    for tick in 1..=3 {
        combined.step(tick).unwrap();
        b_only.step(tick).unwrap();
    }
    assert_eq!(
        projection(&combined, "test.b"),
        projection(&b_only, "test.b")
    );
    let stale = combined.owner(&id("test.a")).unwrap();
    combined
        .remove(combined.owner(&id("test.a")).unwrap())
        .unwrap();
    error(combined.observe(stale), ErrorCode::StaleHandle);
    assert_eq!(combined.describe().plugins.len(), 1);
    assert_eq!(
        projection(&combined, "test.b"),
        projection(&b_only, "test.b")
    );
    for tick in 4..=7 {
        combined.step(tick).unwrap();
        b_only.step(tick).unwrap();
    }
    install(&mut combined, "test.a", Behavior::Pulse);
    for tick in 8..=10 {
        combined.step(tick).unwrap();
        b_only.step(tick).unwrap();
    }
    assert_eq!(
        projection(&combined, "test.b"),
        projection(&b_only, "test.b")
    );
    assert!(
        combined.hash().unwrap() != b_only.hash().unwrap(),
        "whole-host state includes A and global accounting"
    );
}

#[test]
fn dependency_reads_pre_tick_resources_and_pending_events_and_blocks_remove() {
    let mut h = host();
    install_provider(&mut h, "test.provider");
    install_reader(&mut h, "test.provider");
    let before = h.hash().unwrap();
    error(
        h.remove(h.owner(&id("test.provider")).unwrap()),
        ErrorCode::DependencyInUse,
    );
    assert_eq!(h.hash().unwrap(), before);
    let discovery = h.describe();
    let p = discovery
        .plugins
        .iter()
        .find(|p| p.descriptor.id == id("test.provider"))
        .unwrap();
    assert_eq!(p.removal_blockers, vec![id("test.reader")]);
    h.step(1).unwrap();
    let reader = observe(&h, "test.reader");
    assert_eq!(reader.resources["n"], 0);
    assert_eq!(reader.resources["seen"], 7);
    let pulse = observe(&h, "test.provider").events.last().unwrap().value;
    h.step(2).unwrap();
    assert_eq!(observe(&h, "test.reader").resources["n"], 1);
    assert_eq!(observe(&h, "test.reader").resources["seen"], pulse);
    h.remove(h.owner(&id("test.reader")).unwrap()).unwrap();
    h.remove(h.owner(&id("test.provider")).unwrap()).unwrap();
    assert!(h.describe().plugins.is_empty());
}

#[test]
fn registration_dependency_validation_is_atomic_and_precedes_initialize() {
    let mut h = host();
    install_provider(&mut h, "test.provider");
    let before = h.save().unwrap();
    let cases = [
        (reader_desc(), BTreeMap::new(), ErrorCode::InvalidBinding),
        (
            reader_desc(),
            reader_bindings("test.absent"),
            ErrorCode::MissingCapability,
        ),
        (
            reader_desc(),
            reader_bindings("test.reader"),
            ErrorCode::DependencyCycle,
        ),
    ];
    for (desc, bindings, code) in cases {
        error(
            h.install(
                Box::new(Fixture {
                    desc,
                    behavior: Behavior::InitFail,
                }),
                bindings,
            ),
            code,
        );
        assert_eq!(h.save().unwrap(), before);
    }
    let mut extra = reader_bindings("test.provider");
    extra.insert(cap("test.extra"), id("test.provider"));
    error(
        h.install(
            Box::new(Fixture {
                desc: reader_desc(),
                behavior: Behavior::InitFail,
            }),
            extra,
        ),
        ErrorCode::InvalidBinding,
    );
    let mut wrong = reader_desc();
    wrong.requires.insert(
        cap("test.counter"),
        Version {
            major: 1,
            minor: 1,
            patch: 0,
        },
    );
    error(
        h.install(
            Box::new(Fixture {
                desc: wrong,
                behavior: Behavior::InitFail,
            }),
            reader_bindings("test.provider"),
        ),
        ErrorCode::VersionMismatch,
    );
    install(&mut h, "test.quiet", Behavior::Quiet);
    let before = h.hash().unwrap();
    error(
        h.install(
            Box::new(Fixture {
                desc: reader_desc(),
                behavior: Behavior::InitFail,
            }),
            reader_bindings("test.quiet"),
        ),
        ErrorCode::MissingCapability,
    );
    error(
        h.install(
            Box::new(Fixture {
                desc: descriptor("test.quiet"),
                behavior: Behavior::InitFail,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::DuplicateId,
    );
    assert_eq!(h.hash().unwrap(), before);
}

#[test]
fn undeclared_provider_read_is_denied_without_tick_mutation() {
    let mut h = host();
    install_provider(&mut h, "test.provider");
    install(
        &mut h,
        "test.spy",
        Behavior::DeniedRead(id("test.provider")),
    );
    let before = h.save().unwrap();
    error(h.step(1), ErrorCode::PermissionDenied);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn init_failure_leaks_no_descriptor_records_rng_events_and_retry_is_clean() {
    let mut h = host();
    install(&mut h, "test.b", Behavior::Pulse);
    let before = h.save().unwrap();
    let fault = Arc::new(AtomicBool::new(true));
    error(
        h.install(
            Box::new(Fixture {
                desc: descriptor("test.a"),
                behavior: Behavior::FaultControl {
                    init: fault.clone(),
                    tick: Arc::new(AtomicBool::new(false)),
                },
            }),
            BTreeMap::new(),
        ),
        ErrorCode::PluginFailed,
    );
    assert_eq!(h.save().unwrap(), before);
    assert_eq!(h.describe().plugins.len(), 1);
    fault.store(false, Ordering::SeqCst);
    install(&mut h, "test.a", Behavior::Pulse);
    let mut clean = host();
    install(&mut clean, "test.b", Behavior::Pulse);
    install(&mut clean, "test.a", Behavior::Pulse);
    assert_eq!(h.save().unwrap(), clean.save().unwrap());
}

#[test]
fn whole_tick_failure_rolls_back_earlier_plugin_rng_events_and_is_retryable() {
    let mut h = host();
    let mut clean = host();
    let tick_fault = Arc::new(AtomicBool::new(true));
    install(&mut h, "test.a", Behavior::Pulse);
    install(&mut clean, "test.a", Behavior::Pulse);
    h.install(
        Box::new(Fixture {
            desc: descriptor("test.z"),
            behavior: Behavior::FaultControl {
                init: Arc::new(AtomicBool::new(false)),
                tick: tick_fault.clone(),
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    install(&mut clean, "test.z", Behavior::Pulse);
    let before = h.save().unwrap();
    error(h.step(1), ErrorCode::PluginFailed);
    assert_eq!(h.save().unwrap(), before);
    assert_eq!(h.describe().tick, 0);
    tick_fault.store(false, Ordering::SeqCst);
    h.step(1).unwrap();
    clean.step(1).unwrap();
    assert_eq!(h.save().unwrap(), clean.save().unwrap());
    h.step(2).unwrap();
    clean.step(2).unwrap();
    assert_eq!(h.hash().unwrap(), clean.hash().unwrap());
}

#[test]
fn swallowed_invalid_operations_poison_transaction() {
    for behavior in [Behavior::PoisonSet, Behavior::PoisonEmit] {
        let mut h = host();
        install(&mut h, "test.poison", behavior);
        let before = h.save().unwrap();
        error(h.step(1), ErrorCode::UndeclaredKey);
        assert_eq!(h.save().unwrap(), before);
    }
}

#[test]
fn rejected_ticks_do_not_mutate_state_and_the_expected_tick_remains_usable() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    for target in [0, 2, u64::MAX] {
        let before = h.hash().unwrap();
        error(h.step(target), ErrorCode::InvalidTick);
        assert_eq!(h.hash().unwrap(), before);
    }
    h.step(1).unwrap();
    for target in [0, 1, 3] {
        let before = h.hash().unwrap();
        error(h.step(target), ErrorCode::InvalidTick);
        assert_eq!(h.hash().unwrap(), before);
    }
    h.step(2).unwrap();
    assert_eq!(observe(&h, "test.a").resources["n"], 2);
}

#[test]
fn owner_tokens_reject_cross_host_removed_reinstalled_and_restored_generations() {
    let mut a = host();
    let mut b = host();
    install(&mut a, "test.a", Behavior::Pulse);
    install(&mut b, "test.a", Behavior::Pulse);
    error(
        b.observe(a.owner(&id("test.a")).unwrap()),
        ErrorCode::StaleHandle,
    );
    let removed = a.owner(&id("test.a")).unwrap();
    let stale_remove = a.owner(&id("test.a")).unwrap();
    a.remove(a.owner(&id("test.a")).unwrap()).unwrap();
    install(&mut a, "test.a", Behavior::Pulse);
    error(a.observe(removed), ErrorCode::StaleHandle);
    error(a.remove(stale_remove), ErrorCode::StaleHandle);
    let restored = a.owner(&id("test.a")).unwrap();
    let bytes = a.save().unwrap();
    a.restore(&bytes).unwrap();
    error(a.observe(restored), ErrorCode::StaleHandle);
    assert_eq!(
        a.observe(a.owner(&id("test.a")).unwrap())
            .unwrap()
            .resources["n"],
        0
    );
}

#[test]
fn canonical_restore_preserves_pending_replay_and_reissues_all_tokens() {
    let mut a = host();
    install_provider(&mut a, "test.provider");
    install_reader(&mut a, "test.provider");
    let bytes = a.save().unwrap();
    let digest = a.hash().unwrap();
    assert_eq!(digest, format!("{:x}", Sha256::digest(&bytes)));
    let old_p = a.owner(&id("test.provider")).unwrap();
    let old_r = a.owner(&id("test.reader")).unwrap();
    a.step(1).unwrap();
    let next = a.save().unwrap();
    a.restore(&bytes).unwrap();
    assert_eq!(a.save().unwrap(), bytes);
    assert_eq!(a.hash().unwrap(), digest);
    error(a.observe(old_p), ErrorCode::StaleHandle);
    error(a.observe(old_r), ErrorCode::StaleHandle);
    a.step(1).unwrap();
    assert_eq!(a.save().unwrap(), next);
    assert_eq!(observe(&a, "test.reader").resources["seen"], 7);
    a.restore(&next).unwrap();
    a.step(2).unwrap();
    assert_eq!(observe(&a, "test.reader").resources["n"], 1);
}

fn replace(bytes: &[u8], old: &str, new: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).unwrap();
    assert!(text.contains(old), "fixture pattern missing: {old}");
    text.replacen(old, new, 1).into_bytes()
}
fn rejected_restore(h: &mut Host, bytes: &[u8], code: ErrorCode) {
    let before = h.save().unwrap();
    let tokens: Vec<_> = h
        .describe()
        .plugins
        .iter()
        .map(|p| h.owner(&p.descriptor.id).unwrap())
        .collect();
    error(h.restore(bytes), code);
    assert_eq!(h.save().unwrap(), before);
    for token in tokens {
        h.observe(token).unwrap();
    }
}

#[test]
fn malformed_noncanonical_duplicate_unknown_oversized_saves_are_atomic() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    let bytes = h.save().unwrap();
    let invalid = vec![
        b"{".to_vec(),
        b"null".to_vec(),
        [&b" "[..], bytes.as_slice()].concat(),
        [bytes.as_slice(), &b" "[..]].concat(),
        [bytes.as_slice(), &b"{}"[..]].concat(),
        replace(
            &bytes,
            "\"format_version\":1",
            "\"format_version\":1,\"unknown\":0",
        ),
        replace(
            &bytes,
            "\"format_version\":1",
            "\"format_version\":1,\"format_version\":1",
        ),
        replace(
            &bytes,
            "\"resources\":{\"n\":0}",
            "\"resources\":{\"n\":0,\"n\":0}",
        ),
        replace(&bytes, "\"n\":0", "\"n\":0.0"),
        vec![b' '; 1_048_577],
    ];
    for malformed in invalid {
        rejected_restore(&mut h, &malformed, ErrorCode::InvalidSave);
    }
}

#[test]
fn restore_immutable_content_seed_and_exact_descriptors_must_match() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    let bytes = h.save().unwrap();
    for mismatch in [
        replace(
            &bytes,
            "\"content_binding\":\"test:content-v1\"",
            "\"content_binding\":\"test:other\"",
        ),
        replace(&bytes, "\"seed\":42", "\"seed\":43"),
        replace(
            &bytes,
            "\"release\":{\"major\":1,\"minor\":0,\"patch\":0}",
            "\"release\":{\"major\":1,\"minor\":0,\"patch\":1}",
        ),
    ] {
        rejected_restore(&mut h, &mismatch, ErrorCode::SaveMismatch);
    }
    let mut different = host();
    install(&mut different, "test.b", Behavior::Pulse);
    rejected_restore(&mut h, &different.save().unwrap(), ErrorCode::SaveMismatch);
}

#[test]
fn restore_exact_binding_mismatch_keeps_live_tokens() {
    let mut h = host();
    install_provider(&mut h, "test.p1");
    install_provider(&mut h, "test.p2");
    install_reader(&mut h, "test.p1");
    let bytes = h.save().unwrap();
    let mismatch = replace(
        &bytes,
        "\"bindings\":{\"test.counter\":\"test.p1\"}",
        "\"bindings\":{\"test.counter\":\"test.p2\"}",
    );
    rejected_restore(&mut h, &mismatch, ErrorCode::SaveMismatch);
}

#[test]
fn invalid_event_ownership_pending_accounting_and_resource_wire_values_reject_atomically() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    let bytes = h.save().unwrap();
    for invalid in [
        replace(
            &bytes,
            "\"resources\":{\"n\":0}",
            "\"resources\":{\"bad\":0}",
        ),
        replace(&bytes, "\"n\":0", "\"n\":9223372036854775808"),
        replace(&bytes, "\"owner\":\"test.a\"", "\"owner\":\"test.unknown\""),
        replace(&bytes, "\"kind\":\"pulse\"", "\"kind\":\"other\""),
        replace(&bytes, "\"next_sequence\":1", "\"next_sequence\":2"),
        replace(&bytes, "\"events_dropped\":0", "\"events_dropped\":1"),
        replace(&bytes, "\"tick\":0", "\"tick\":1"),
        replace(&bytes, "\"sequence\":0", "\"sequence\":1"),
    ] {
        rejected_restore(&mut h, &invalid, ErrorCode::InvalidSave);
    }
    // The pending event changes while the corresponding retained history stays intact.
    let text = String::from_utf8(bytes.clone()).unwrap();
    let split = text.find("\"pending\":").unwrap();
    let (prefix, pending) = text.split_at(split);
    let invalid = format!(
        "{}{}",
        prefix,
        pending.replacen("\"value\":7", "\"value\":8", 1)
    );
    rejected_restore(&mut h, invalid.as_bytes(), ErrorCode::InvalidSave);
}

#[test]
fn event_history_eviction_and_owner_purge_preserve_global_accounting() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Emits { init: 0, tick: 64 });
    install(&mut h, "test.b", Behavior::Emits { init: 0, tick: 64 });
    for tick in 1..=3 {
        h.step(tick).unwrap();
    }
    let a = observe(&h, "test.a");
    let b = observe(&h, "test.b");
    assert_eq!(a.events.len() + b.events.len(), 256);
    assert_eq!(
        (a.next_sequence, a.events_dropped, a.first_retained_sequence),
        (384, 128, 128)
    );
    assert_eq!(
        b.first_retained_sequence, 128,
        "global oldest sequence, not B's oldest"
    );
    h.remove(h.owner(&id("test.a")).unwrap()).unwrap();
    let b = observe(&h, "test.b");
    assert_eq!(b.events.len(), 128);
    assert_eq!(
        (b.next_sequence, b.events_dropped, b.first_retained_sequence),
        (384, 256, 192)
    );
    assert!(b.events.iter().all(|e| e.owner == id("test.b")));
    let bytes = h.save().unwrap();
    h.restore(&bytes).unwrap();
    h.step(4).unwrap();
    let b = observe(&h, "test.b");
    assert_eq!(
        (b.next_sequence, b.events_dropped, b.events.len()),
        (448, 256, 192)
    );
    h.remove(h.owner(&id("test.b")).unwrap()).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&h.save().unwrap()).unwrap();
    assert_eq!(saved["next_sequence"], 448);
    assert_eq!(saved["events_dropped"], 448);
    assert_eq!(saved["history"].as_array().unwrap().len(), 0);
    assert_eq!(saved["pending"].as_array().unwrap().len(), 0);
}

#[test]
fn splitmix_stream_matches_seeded_public_definition_and_independent_installation() {
    let initial: [u8; 8] = Sha256::digest([42_u64.to_le_bytes().as_slice(), b"test.a"].concat())
        [..8]
        .try_into()
        .unwrap();
    let mut state = u64::from_le_bytes(initial);
    fn draw(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    draw(&mut state);
    assert_eq!(observe(&h, "test.a").rng_state, state);
    for tick in 1..=5 {
        let expected = draw(&mut state);
        h.step(tick).unwrap();
        let a = observe(&h, "test.a");
        assert_eq!(a.rng_state, state);
        assert_eq!(
            a.events.last().unwrap().value,
            (expected & 0x7fff_ffff_ffff_ffff) as i64
        );
    }
    let mut same = host();
    install(&mut same, "test.z", Behavior::Pulse);
    install(&mut same, "test.a", Behavior::Pulse);
    for tick in 1..=5 {
        same.step(tick).unwrap();
    }
    assert_eq!(projection(&h, "test.a"), projection(&same, "test.a"));
    let mut other = Host::new(43, "test:content-v1").unwrap();
    install(&mut other, "test.a", Behavior::Pulse);
    assert_ne!(
        observe(&h, "test.a").rng_state,
        observe(&other, "test.a").rng_state
    );
}

#[test]
fn command_budget_boundary_and_event_commit_and_pending_bounds_are_atomic() {
    let mut allowed = host();
    install(&mut allowed, "test.a", Behavior::Sets(1024));
    allowed.step(1).unwrap();
    assert_eq!(observe(&allowed, "test.a").resources["n"], 1023);
    let mut denied = host();
    install(&mut denied, "test.a", Behavior::Sets(1025));
    let before = denied.save().unwrap();
    error(denied.step(1), ErrorCode::BudgetExceeded);
    assert_eq!(denied.save().unwrap(), before);
    for count in [128, 129] {
        let mut h = host();
        let before = h.save().unwrap();
        let result = h.install(
            Box::new(Fixture {
                desc: descriptor("test.a"),
                behavior: Behavior::Emits {
                    init: count,
                    tick: count,
                },
            }),
            BTreeMap::new(),
        );
        if count == 128 {
            result.unwrap();
            h.step(1).unwrap();
            assert_eq!(observe(&h, "test.a").events.len(), 256);
        } else {
            error(result, ErrorCode::BudgetExceeded);
            assert_eq!(h.save().unwrap(), before);
        }
    }
    let mut h = host();
    install(&mut h, "test.a", Behavior::Emits { init: 128, tick: 0 });
    let before = h.save().unwrap();
    error(
        h.install(
            Box::new(Fixture {
                desc: descriptor("test.b"),
                behavior: Behavior::Emits { init: 1, tick: 0 },
            }),
            BTreeMap::new(),
        ),
        ErrorCode::BudgetExceeded,
    );
    assert_eq!(h.save().unwrap(), before);
    let mut h = host();
    install(&mut h, "test.a", Behavior::Emits { init: 0, tick: 128 });
    install(&mut h, "test.b", Behavior::Emits { init: 0, tick: 1 });
    let before = h.save().unwrap();
    error(h.step(1), ErrorCode::BudgetExceeded);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn descriptor_and_plugin_limits_are_exposed_and_enforced_before_init() {
    let mut h = host();
    let l = h.describe().limits;
    assert_eq!(
        (
            l.max_plugins,
            l.max_provides,
            l.max_requires,
            l.max_resources,
            l.max_event_kinds
        ),
        (16, 32, 32, 64, 32)
    );
    assert_eq!(
        (
            l.max_id_bytes,
            l.max_events_per_commit,
            l.max_pending_events,
            l.max_history_events,
            l.max_commands
        ),
        (128, 128, 128, 256, 1024)
    );
    assert_eq!(
        (
            l.max_error_bytes,
            l.max_content_binding_bytes,
            l.max_save_bytes
        ),
        (4096, 128, 1_048_576)
    );
    let mut resource = descriptor("test.bad");
    resource.resources = (0..65).map(|i| format!("r{i}")).collect();
    let mut kinds = descriptor("test.bad");
    kinds.event_kinds = (0..33).map(|i| format!("e{i}")).collect();
    let mut provides = descriptor("test.bad");
    provides.provides = (0..33).map(|i| (cap(&format!("test.c{i}")), v())).collect();
    let mut requires = descriptor("test.bad");
    requires.requires = provides.provides.clone();
    for desc in [resource, kinds, provides, requires] {
        let before = h.save().unwrap();
        error(
            h.install(
                Box::new(Fixture {
                    desc,
                    behavior: Behavior::InitFail,
                }),
                BTreeMap::new(),
            ),
            ErrorCode::BudgetExceeded,
        );
        assert_eq!(h.save().unwrap(), before);
    }
    let mut invalid = descriptor("test.bad");
    invalid.resources.insert("bad.key".into());
    error(
        h.install(
            Box::new(Fixture {
                desc: invalid,
                behavior: Behavior::InitFail,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::InvalidDescriptor,
    );
    let mut contract = descriptor("test.bad");
    contract.contract.patch = 1;
    error(
        h.install(
            Box::new(Fixture {
                desc: contract,
                behavior: Behavior::InitFail,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::VersionMismatch,
    );
    for i in 0..16 {
        install(&mut h, &format!("test.p{i}"), Behavior::Quiet);
    }
    let before = h.save().unwrap();
    error(
        h.install(
            Box::new(Fixture {
                desc: descriptor("test.extra"),
                behavior: Behavior::InitFail,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::BudgetExceeded,
    );
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn identifier_content_binding_and_error_utf8_limits_validate_exactly() {
    for invalid in ["", "one", ".a", "a.", "a..b", "a.b c", "a.한", "a/b.c"] {
        error(PluginId::new(invalid), ErrorCode::InvalidIdentifier);
        error(CapabilityId::new(invalid), ErrorCode::InvalidIdentifier);
    }
    let boundary = format!("a.{}", "x".repeat(126));
    assert_eq!(id(&boundary).as_str().len(), 128);
    error(
        PluginId::new(&(boundary + "x")),
        ErrorCode::InvalidIdentifier,
    );
    for invalid in ["", "with space", "/path", "한글"] {
        error(Host::new(0, invalid), ErrorCode::InvalidIdentifier);
    }
    Host::new(0, &"x".repeat(128)).unwrap();
    error(Host::new(0, &"x".repeat(129)), ErrorCode::InvalidIdentifier);
    let detail = format!("{}한글", "x".repeat(4095));
    let e = Error::new(ErrorCode::PluginFailed, detail);
    assert_eq!(e.code(), ErrorCode::PluginFailed);
    assert_eq!(e.detail().len(), 4095);
    assert!(std::str::from_utf8(e.detail().as_bytes()).is_ok());
}

#[test]
fn swallowed_over_budget_draws_poison_and_roll_back_rng() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::PoisonBudget);
    let before = h.save().unwrap();
    error(h.step(1), ErrorCode::BudgetExceeded);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn empty_descriptors_and_all_64_resource_records_are_valid_scalar_boundaries() {
    let mut h = host();
    let mut empty = descriptor("test.empty");
    empty.resources.clear();
    empty.event_kinds.clear();
    h.install(
        Box::new(Fixture {
            desc: empty,
            behavior: Behavior::Quiet,
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let mut full = descriptor("test.full");
    full.resources = (0..64).map(|i| format!("r{i}")).collect();
    h.install(
        Box::new(Fixture {
            desc: full,
            behavior: Behavior::WriteDeclared,
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let observation = observe(&h, "test.full");
    assert_eq!(observation.resources.len(), 64);
    assert!(
        observation
            .resources
            .values()
            .all(|value| *value == i64::MIN)
    );
    let bytes = h.save().unwrap();
    h.restore(&bytes).unwrap();
    h.step(1).unwrap();
    let observation = observe(&h, "test.full");
    assert!(
        observation
            .resources
            .values()
            .all(|value| *value == i64::MAX)
    );
    assert!(observe(&h, "test.empty").resources.is_empty());
}

#[test]
fn context_tick_is_current_for_init_and_target_for_step() {
    let mut h = host();
    h.step(1).unwrap();
    h.step(2).unwrap();
    install(&mut h, "test.a", Behavior::RecordTick);
    assert_eq!(observe(&h, "test.a").resources["n"], 2);
    h.step(3).unwrap();
    assert_eq!(observe(&h, "test.a").resources["n"], 3);
}

#[test]
fn save_plugin_arrays_must_be_sorted_unique_and_versions_exact() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Quiet);
    install(&mut h, "test.b", Behavior::Quiet);
    let bytes = h.save().unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let start = text.find("\"plugins\":[").unwrap() + "\"plugins\":[".len();
    let end = text.find("],\"history\":").unwrap();
    let records = &text[start..end];
    let boundary = records.find("},{\"descriptor\":").unwrap() + 1;
    let a = &records[..boundary];
    let b = &records[boundary + 1..];
    let reversed = format!("{}{b},{a}{}", &text[..start], &text[end..]);
    rejected_restore(&mut h, reversed.as_bytes(), ErrorCode::InvalidSave);
    let duplicate = format!("{}{a},{a}{}", &text[..start], &text[end..]);
    rejected_restore(&mut h, duplicate.as_bytes(), ErrorCode::InvalidSave);
    let version = replace(&bytes, "\"format_version\":1", "\"format_version\":2");
    rejected_restore(&mut h, &version, ErrorCode::SaveMismatch);
    let contract = replace(
        &bytes,
        "\"contract_version\":{\"major\":1,\"minor\":0,\"patch\":0}",
        "\"contract_version\":{\"major\":2,\"minor\":0,\"patch\":0}",
    );
    rejected_restore(&mut h, &contract, ErrorCode::SaveMismatch);
}

#[test]
fn checked_tick_and_sequence_overflows_do_not_wrap_or_publish_initialization() {
    let mut h = host();
    let bytes = h.save().unwrap();
    let max_tick = replace(&bytes, "\"tick\":0", "\"tick\":18446744073709551615");
    h.restore(&max_tick).unwrap();
    let before = h.hash().unwrap();
    error(h.step(0), ErrorCode::Overflow);
    assert_eq!(h.hash().unwrap(), before);
    let mut h = host();
    let bytes = h.save().unwrap();
    let full_sequence = replace(
        &replace(
            &bytes,
            "\"next_sequence\":0",
            "\"next_sequence\":18446744073709551615",
        ),
        "\"events_dropped\":0",
        "\"events_dropped\":18446744073709551615",
    );
    h.restore(&full_sequence).unwrap();
    let before = h.save().unwrap();
    error(
        h.install(
            Box::new(Fixture {
                desc: descriptor("test.a"),
                behavior: Behavior::Pulse,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::Overflow,
    );
    assert_eq!(h.save().unwrap(), before);
    assert!(h.describe().plugins.is_empty());
}

#[test]
fn pending_cannot_omit_current_tick_history_and_history_ticks_cannot_regress() {
    let mut h = host();
    install(&mut h, "test.a", Behavior::Pulse);
    let bytes = h.save().unwrap();
    let missing_pending = replace(
        &bytes,
        "\"pending\":[{\"tick\":0,\"owner\":\"test.a\",\"kind\":\"pulse\",\"value\":7,\"sequence\":0}]",
        "\"pending\":[]",
    );
    rejected_restore(&mut h, &missing_pending, ErrorCode::InvalidSave);
    h.step(1).unwrap();
    h.step(2).unwrap();
    let bytes = h.save().unwrap();
    // Sequence0 is changed to tick2, followed by the tick1 event: ascending
    // sequence alone is insufficient to reject an impossible commit history.
    let regressed = replace(
        &bytes,
        "\"history\":[{\"tick\":0",
        "\"history\":[{\"tick\":2",
    );
    rejected_restore(&mut h, &regressed, ErrorCode::InvalidSave);
}

#[test]
fn independent_current_tick_projection_survives_shared_history_retention_boundary() {
    let mut combined = host();
    let mut b_only = host();
    install(
        &mut combined,
        "test.a",
        Behavior::Emits { init: 0, tick: 64 },
    );
    install(
        &mut combined,
        "test.b",
        Behavior::Emits { init: 0, tick: 64 },
    );
    install(&mut b_only, "test.b", Behavior::Emits { init: 0, tick: 64 });
    let current = |host: &Host| {
        let o = observe(host, "test.b");
        (
            o.tick,
            o.resources,
            o.rng_state,
            o.events
                .into_iter()
                .filter(|e| e.tick == o.tick)
                .map(|e| (e.tick, e.owner, e.kind, e.value))
                .collect::<Vec<_>>(),
        )
    };
    for tick in 1..=3 {
        combined.step(tick).unwrap();
        b_only.step(tick).unwrap();
        assert_eq!(current(&combined), current(&b_only));
    }
    let b = observe(&combined, "test.b");
    let independent = observe(&b_only, "test.b");
    assert_eq!((b.events.len(), independent.events.len()), (128, 192));
    assert_eq!(
        (b.next_sequence, b.events_dropped, b.first_retained_sequence),
        (384, 128, 128)
    );
    combined
        .remove(combined.owner(&id("test.a")).unwrap())
        .unwrap();
    combined.step(4).unwrap();
    b_only.step(4).unwrap();
    assert_eq!(current(&combined), current(&b_only));
    let b = observe(&combined, "test.b");
    let independent = observe(&b_only, "test.b");
    assert_eq!((b.events.len(), independent.events.len()), (192, 256));
    assert_eq!((b.next_sequence, b.events_dropped), (448, 256));
    assert_eq!(
        (independent.next_sequence, independent.events_dropped),
        (256, 0)
    );
}
