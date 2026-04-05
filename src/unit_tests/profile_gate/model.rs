use crate::{
    constants::PROFILES_STORAGE_KEY,
    models::{
        ctx::Ctx,
        profile_gate::{GateStatus, ProfileGate},
    },
    runtime::{
        msg::{Action, ActionProfileGate},
        Runtime, RuntimeAction,
    },
    types::{
        events::DismissedEventsBucket,
        library::LibraryBucket,
        notifications::NotificationsBucket,
        profile::Profile,
        profile_gate::{LocalProfile, LocalProfileId, PinConfig, ProfilesBucket},
        search_history::SearchHistoryBucket,
        server_urls::ServerUrlsBucket,
        streams::StreamsBucket,
    },
    unit_tests::{TestEnv, STORAGE},
};

use chrono::Utc;
use stremio_derive::Model;

#[derive(Model, Clone)]
#[model(TestEnv)]
struct TestModel {
    ctx: Ctx,
    profile_gate: ProfileGate,
}

fn make_test_model(bucket: ProfilesBucket) -> TestModel {
    let ctx = Ctx::new(
        Profile::default(),
        LibraryBucket::default(),
        StreamsBucket::default(),
        ServerUrlsBucket::new::<TestEnv>(None),
        NotificationsBucket::new::<TestEnv>(None, vec![]),
        SearchHistoryBucket::default(),
        DismissedEventsBucket::default(),
    );
    TestModel {
        ctx,
        profile_gate: ProfileGate::new(bucket),
    }
}

#[test]
fn gate_inactive_when_disabled() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let bucket = ProfilesBucket::new(); // settings.enabled = false by default
    let model = make_test_model(bucket);
    assert_eq!(model.profile_gate.status, GateStatus::Inactive);
}

#[test]
fn gate_shows_on_startup_when_enabled() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let p = LocalProfile::new("Alice".to_string(), Utc::now());
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    assert_eq!(model.profile_gate.status, GateStatus::SelectingProfile);
}

#[test]
fn show_gate_action() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let p = LocalProfile::new("Alice".to_string(), Utc::now());
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::DismissGate),
        })
    });
    assert_eq!(
        runtime.model().unwrap().profile_gate.status,
        GateStatus::Inactive
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::ShowGate),
        })
    });
    assert_eq!(
        runtime.model().unwrap().profile_gate.status,
        GateStatus::SelectingProfile
    );
}

#[test]
fn select_profile_no_pin() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let p = LocalProfile::new("Alice".to_string(), Utc::now());
    let id = p.id.clone();
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::SelectProfile {
                profile_id: id.clone(),
                pin: None,
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert_eq!(
        model.profile_gate.status,
        GateStatus::LoadingProfile(id)
    );
}

#[test]
fn select_pin_profile_no_pin_shows_awaiting() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let mut p = LocalProfile::new("Alice".to_string(), Utc::now());
    p.pin = Some(PinConfig::new("1234"));
    let id = p.id.clone();
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::SelectProfile {
                profile_id: id.clone(),
                pin: None,
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert!(matches!(
        model.profile_gate.status,
        GateStatus::AwaitingPin { error: None, .. }
    ));
}

#[test]
fn select_pin_profile_correct_pin() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let mut p = LocalProfile::new("Alice".to_string(), Utc::now());
    p.pin = Some(PinConfig::new("1234"));
    let id = p.id.clone();
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::SelectProfile {
                profile_id: id.clone(),
                pin: Some("1234".to_string()),
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert_eq!(
        model.profile_gate.status,
        GateStatus::LoadingProfile(id)
    );
}

#[test]
fn select_pin_profile_wrong_pin() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let mut p = LocalProfile::new("Alice".to_string(), Utc::now());
    p.pin = Some(PinConfig::new("1234"));
    let id = p.id.clone();
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::SelectProfile {
                profile_id: id.clone(),
                pin: Some("9999".to_string()),
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert!(matches!(
        model.profile_gate.status,
        GateStatus::AwaitingPin { error: Some(_), .. }
    ));
}

#[test]
fn create_profile() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let model = make_test_model(ProfilesBucket::new());
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::CreateProfile {
                name: "Alice".to_string(),
                avatar: None,
                pin: None,
                is_kids: false,
                color: None,
            }),
        })
    });
    let model = runtime.model().unwrap();
    assert_eq!(model.profile_gate.profiles_bucket.profiles.len(), 1);
    // Verify persisted to storage
    let storage = STORAGE.read().unwrap();
    let raw = storage
        .get(PROFILES_STORAGE_KEY)
        .expect("bucket should be in storage");
    let saved: ProfilesBucket = serde_json::from_str(raw).unwrap();
    assert_eq!(saved.profiles.len(), 1);
}

#[test]
fn create_profile_invalid_pin() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let model = make_test_model(ProfilesBucket::new());
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::CreateProfile {
                name: "Alice".to_string(),
                avatar: None,
                pin: Some("abc".to_string()), // invalid: not digits
                is_kids: false,
                color: None,
            }),
        })
    });
    let model = runtime.model().unwrap();
    // Profile should NOT have been created due to invalid PIN
    assert_eq!(model.profile_gate.profiles_bucket.profiles.len(), 0);
}

#[test]
fn profiles_persisted_to_storage() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let model = make_test_model(ProfilesBucket::new());
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::CreateProfile {
                name: "Bob".to_string(),
                avatar: None,
                pin: None,
                is_kids: false,
                color: None,
            }),
        })
    });
    let storage = STORAGE.read().unwrap();
    assert!(
        storage.contains_key(PROFILES_STORAGE_KEY),
        "bucket should be persisted to storage"
    );
}

#[test]
fn dismiss_gate_sets_inactive() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let p = LocalProfile::new("Alice".to_string(), Utc::now());
    bucket.add_profile(p).unwrap();
    let model = make_test_model(bucket);
    // Initially SelectingProfile because enabled=true and no active_profile_id
    assert_eq!(model.profile_gate.status, GateStatus::SelectingProfile);
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::DismissGate),
        })
    });
    assert_eq!(
        runtime.model().unwrap().profile_gate.status,
        GateStatus::Inactive
    );
}

#[test]
fn select_nonexistent_profile_does_not_crash() {
    let _env_mutex = TestEnv::reset().expect("Should lock TestEnv");
    let model = make_test_model(ProfilesBucket::new());
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(model, vec![], 1000);
    let fake_id = LocalProfileId("does_not_exist".to_string());
    // Dispatching a select for a nonexistent profile should emit an error event, not crash
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::ProfileGate(ActionProfileGate::SelectProfile {
                profile_id: fake_id,
                pin: None,
            }),
        })
    });
    // Status should remain unchanged (Inactive for disabled gate)
    assert_eq!(
        runtime.model().unwrap().profile_gate.status,
        GateStatus::Inactive
    );
}
