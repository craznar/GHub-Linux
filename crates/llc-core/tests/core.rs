use llc_core::{engine::*, *};
use uuid::Uuid;

fn macro_with(operations: Vec<Operation>) -> Macro {
    Macro {
        id: Uuid::new_v4(),
        name: "Test".into(),
        interval_ms: 10,
        operations,
    }
}
fn tap() -> Macro {
    macro_with(vec![Operation::Tap {
        outputs: vec![Output::Key(29), Output::Key(30)],
        hold_ms: 30,
    }])
}
fn binding(control: &str, action: Action) -> Binding {
    Binding {
        control: control.into(),
        secondary: false,
        bank: None,
        impulse: false,
        action,
    }
}
fn down(code: u16) -> Event {
    Event::Down {
        output: Output::Key(code),
    }
}
fn up(code: u16) -> Event {
    Event::Up {
        output: Output::Key(code),
    }
}

#[test]
fn cancellation_releases_reverse_acquisition_order_not_key_number() {
    let mut ledger = Ledger::default();
    ledger.down("owner", Output::Key(30));
    ledger.down("owner", Output::Key(29));
    assert_eq!(ledger.release("owner"), vec![up(29), up(30)]);
}

#[test]
fn scheduler_rejects_backwards_time_and_oversized_repeat() {
    let mut scheduler = Scheduler::default();
    scheduler
        .start(
            "a",
            tap().compile().unwrap(),
            Execution::Once,
            Retrigger::Ignore,
            10,
        )
        .unwrap();
    assert!(scheduler.advance(9).is_err());
    assert!(scheduler
        .start(
            "b",
            tap().compile().unwrap(),
            Execution::Once,
            Retrigger::Ignore,
            9
        )
        .is_err());
    assert!(scheduler
        .start(
            "b",
            tap().compile().unwrap(),
            Execution::Repeat {
                count: MAX_REPEAT + 1
            },
            Retrigger::Ignore,
            10
        )
        .is_err());
}

#[test]
fn defaults_are_passive_and_roundtrip() {
    let config = Config::default();
    assert!(!config.login_activation);
    let bytes = serde_json::to_vec(&config).unwrap();
    Config::parse(&bytes).unwrap();
    assert_eq!(
        config.profiles[0].resolve("g502.back", 1, true),
        Action::PassThrough
    );
}
#[test]
fn rejects_future_schema_unknown_fields_and_oversize() {
    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["schema_version"] = 2.into();
    assert!(Config::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    value["schema_version"] = 1.into();
    value["shell"] = "bad".into();
    assert!(Config::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(Config::parse(&vec![b' '; MAX_CONFIG_BYTES + 1]).is_err());
}
#[test]
fn rejects_unbalanced_and_duplicate_presses() {
    let key = Output::Key(29);
    for operations in [
        vec![Operation::Down { output: key }],
        vec![Operation::Up { output: key }],
        vec![
            Operation::Down { output: key },
            Operation::Down { output: key },
            Operation::Up { output: key },
        ],
    ] {
        assert!(macro_with(operations).compile().is_err());
    }
}
#[test]
fn macro_bounds() {
    for ms in [0, MAX_DELAY_MS + 1] {
        assert!(macro_with(vec![Operation::Wait { ms }]).compile().is_err());
    }
    assert!(macro_with(vec![Operation::Wait { ms: MAX_DELAY_MS }])
        .compile()
        .is_ok());
    assert!(macro_with(vec![Operation::Wait { ms: 1 }; MAX_STORED_OPS])
        .compile()
        .is_ok());
    assert!(
        macro_with(vec![Operation::Wait { ms: 1 }; MAX_STORED_OPS + 1])
            .compile()
            .is_err()
    );
    assert!(macro_with(vec![Operation::Repeat {
        count: 0,
        body: vec![Operation::Wait { ms: 1 }]
    }])
    .compile()
    .is_err());
    let body = vec![Operation::Wait { ms: 1 }; 11];
    assert!(macro_with(vec![Operation::Repeat {
        count: MAX_REPEAT,
        body
    }])
    .compile()
    .is_err());
    assert!(macro_with(vec![Operation::Wait { ms: MAX_DELAY_MS }; 7])
        .compile()
        .is_err());
    let mut body = vec![Operation::Wait { ms: 1 }];
    for _ in 0..5 {
        body = vec![Operation::Repeat { count: 1, body }];
    }
    assert!(macro_with(body).compile().is_err());
}
#[test]
fn physical_modifier_survives_macro_and_cancellation() {
    let mut scheduler = Scheduler::default();
    assert_eq!(
        scheduler.ledger.down("physical", Output::Key(29)),
        Some(down(29))
    );
    scheduler
        .start(
            "mouse",
            tap().compile().unwrap(),
            Execution::WhileHeld,
            Retrigger::Ignore,
            0,
        )
        .unwrap();
    assert_eq!(scheduler.advance(0).unwrap().0, vec![down(30)]);
    assert_eq!(scheduler.cancel_source("mouse", true), vec![up(30)]);
    assert_eq!(scheduler.ledger.held_count(), 1);
    assert_eq!(
        scheduler.ledger.up("physical", Output::Key(29)),
        Some(up(29))
    );
}
#[test]
fn once_ignores_release_but_while_held_cancels_wait() {
    for (mode, cancels) in [(Execution::Once, false), (Execution::WhileHeld, true)] {
        let mut scheduler = Scheduler::default();
        scheduler
            .start(
                "source",
                tap().compile().unwrap(),
                mode,
                Retrigger::Ignore,
                0,
            )
            .unwrap();
        assert_eq!(scheduler.advance(0).unwrap().0.len(), 2);
        let events = scheduler.cancel_source("source", true);
        assert_eq!(!events.is_empty(), cancels);
        if !cancels {
            assert_eq!(scheduler.advance(30).unwrap().0, vec![up(30), up(29)]);
        }
        assert_eq!(scheduler.ledger.held_count(), 0);
    }
}
#[test]
fn toggle_and_retrigger_semantics() {
    let mut s = Scheduler::default();
    let p = tap().compile().unwrap();
    s.start("a", p.clone(), Execution::Toggle, Retrigger::Ignore, 0)
        .unwrap();
    s.advance(0).unwrap();
    assert_eq!(
        s.start("a", p.clone(), Execution::Toggle, Retrigger::Ignore, 1)
            .unwrap(),
        vec![up(30), up(29)]
    );
    s.start("a", p.clone(), Execution::Once, Retrigger::Ignore, 1)
        .unwrap();
    s.advance(1).unwrap();
    s.start("a", p.clone(), Execution::Once, Retrigger::Restart, 2)
        .unwrap();
    assert_eq!(s.active_count(), 1); // running binding's ignore policy wins
    s.cancel_all();
    s.start("a", p.clone(), Execution::Once, Retrigger::Restart, 2)
        .unwrap();
    s.advance(2).unwrap();
    assert_eq!(
        s.start("a", p, Execution::Once, Retrigger::Restart, 3)
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn concurrent_limit_and_rate_limit_release_everything() {
    let mut s = Scheduler::default();
    for n in 0..MAX_INSTANCES {
        s.start(
            &n.to_string(),
            tap().compile().unwrap(),
            Execution::Once,
            Retrigger::Ignore,
            0,
        )
        .unwrap();
    }
    assert!(s
        .start(
            "overflow",
            tap().compile().unwrap(),
            Execution::Once,
            Retrigger::Ignore,
            0
        )
        .is_err());
    s.advance(0).unwrap();
    s.cancel_all();
    assert_eq!(s.ledger.held_count(), 0);
    let ops = vec![Operation::Repeat {
        count: 2000,
        body: vec![
            Operation::Down {
                output: Output::Key(29),
            },
            Operation::Up {
                output: Output::Key(29),
            },
        ],
    }];
    s.start(
        "burst",
        macro_with(ops).compile().unwrap(),
        Execution::Once,
        Retrigger::Ignore,
        1,
    )
    .unwrap();
    let (_, faults) = s.advance(1).unwrap();
    assert!(!faults.is_empty());
    assert_eq!(s.ledger.held_count(), 0);
    assert_eq!(s.active_count(), 0);
}
#[test]
fn bank_and_layer_changes_preserve_captured_release() {
    let mut c = Config::default();
    c.profiles[0].bindings = vec![
        binding("g815.key.29", Action::LayerHold),
        binding(
            "g502.back",
            Action::Hold {
                outputs: vec![Output::Key(30)],
            },
        ),
        Binding {
            secondary: true,
            ..binding(
                "g502.back",
                Action::Hold {
                    outputs: vec![Output::Key(31)],
                },
            )
        },
    ];
    let mut e = Engine::new(c).unwrap();
    e.press("keyboard", "g815.key.29", None, false, 0).unwrap();
    assert_eq!(
        e.press("mouse", "g502.back", None, false, 0).unwrap(),
        vec![down(31)]
    );
    e.release("keyboard", "g815.key.29");
    e.select_bank(3).unwrap();
    assert_eq!(e.release("mouse", "g502.back"), vec![up(31)]);
    assert!(!e.secondary());
}
#[test]
fn disconnect_clears_only_source_and_profile_transition_rearms_on_release() {
    let mut c = Config::default();
    c.profiles[0].bindings.push(binding(
        "g502.back",
        Action::Hold {
            outputs: vec![Output::Key(29)],
        },
    ));
    let profile = c.selected_profile;
    let mut e = Engine::new(c).unwrap();
    e.press("keyboard", "g815.key.29", Some(Output::Key(29)), false, 0)
        .unwrap();
    assert!(e
        .press("mouse", "g502.back", None, false, 0)
        .unwrap()
        .is_empty());
    assert!(e.disconnect("mouse").is_empty());
    e.press("mouse", "g502.back", None, false, 0).unwrap();
    assert!(e.select_profile(profile).unwrap().is_empty());
    assert!(e
        .press("mouse", "g502.back", None, false, 0)
        .unwrap()
        .is_empty());
    e.release("mouse", "g502.back");
    assert_eq!(e.release("keyboard", "g815.key.29"), vec![up(29)]);
}
#[test]
fn bypass_is_latched_and_repeat_does_not_trigger() {
    let mut c = Config::default();
    c.profiles[0].bindings.push(binding(
        "g502.back",
        Action::Hold {
            outputs: vec![Output::Key(29)],
        },
    ));
    let mut e = Engine::new(c).unwrap();
    assert!(e
        .press("mouse", "g502.back", None, true, 0)
        .unwrap()
        .is_empty());
    e.press("mouse", "g502.back", None, false, 0).unwrap();
    assert_eq!(e.set_bypass(true), vec![up(29)]);
    e.release("mouse", "g502.back");
    assert!(e.bypass);
}
#[test]
fn impulse_and_bank_validation() {
    let mut c = Config::default();
    c.profiles[0].bindings.push(Binding {
        impulse: true,
        ..binding("g502.wheel_up", Action::LayerHold)
    });
    assert!(c.validate().is_err());
    c.profiles[0].bindings = vec![Binding {
        bank: Some(1),
        ..binding("g502.back", Action::Disabled)
    }];
    assert!(c.validate().is_err());
}
#[test]
fn atomic_revision_backup_and_corrupt_primary_are_preserved() {
    let root = std::env::temp_dir().join(format!("llc-test-{}", Uuid::new_v4()));
    let path = root.join("config.json");
    let a = persistence::save(&path, &Config::default(), 0).unwrap();
    assert_eq!(a.revision, 1);
    assert!(matches!(
        persistence::save(&path, &a, 0),
        Err(Error::Revision(1))
    ));
    let b = persistence::save(&path, &a, 1).unwrap();
    assert_eq!(b.revision, 2);
    assert_eq!(
        persistence::read(&path.with_extension("json.bak"))
            .unwrap()
            .revision,
        1
    );
    std::fs::write(&path, b"corrupt").unwrap();
    let (_, backup) = persistence::recover(&path).unwrap();
    assert!(backup);
    assert!(persistence::save(&path, &a, 1).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"corrupt");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path.with_extension("json.bak"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
