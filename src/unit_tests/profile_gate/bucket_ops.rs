use chrono::Utc;

use crate::types::profile_gate::{
    LocalProfile, LocalProfileId, ProfileGateError, ProfilesBucket, MAX_PROFILES,
};

fn make_profile(name: &str) -> LocalProfile {
    LocalProfile::new(name.to_string(), Utc::now())
}

#[test]
fn add_profile_ok() {
    let mut bucket = ProfilesBucket::new();
    let p = make_profile("Alice");
    assert!(bucket.add_profile(p).is_ok());
    assert_eq!(bucket.profiles.len(), 1);
}

#[test]
fn add_profile_max_reached() {
    let mut bucket = ProfilesBucket::new();
    for i in 0..MAX_PROFILES {
        bucket
            .add_profile(make_profile(&format!("User{}", i)))
            .unwrap();
    }
    let result = bucket.add_profile(make_profile("One too many"));
    assert_eq!(result, Err(ProfileGateError::MaxProfilesReached));
}

#[test]
fn remove_last_fails() {
    let mut bucket = ProfilesBucket::with_initial_profile(make_profile("Only"));
    let id = bucket.profiles.keys().next().unwrap().clone();
    let result = bucket.remove_profile(&id);
    assert_eq!(result, Err(ProfileGateError::CannotRemoveLastProfile));
}

#[test]
fn remove_active_clears_selection() {
    let p1 = make_profile("Alice");
    let p2 = make_profile("Bob");
    let id1 = p1.id.clone();
    let mut bucket = ProfilesBucket::new();
    bucket.add_profile(p1).unwrap();
    bucket.add_profile(p2).unwrap();
    bucket.active_profile_id = Some(id1.clone());
    bucket.remove_profile(&id1).unwrap();
    // Active profile should be cleared or auto-selected to a different profile
    assert_ne!(bucket.active_profile_id, Some(id1));
}

#[test]
fn remove_nonexistent_fails() {
    // Need 2 profiles so it's not the "last profile" check
    let mut bucket = ProfilesBucket::new();
    bucket.add_profile(make_profile("Alice")).unwrap();
    bucket.add_profile(make_profile("Bob")).unwrap();
    let fake_id = LocalProfileId("does_not_exist".to_string());
    let result = bucket.remove_profile(&fake_id);
    assert_eq!(result, Err(ProfileGateError::ProfileNotFound));
}

#[test]
fn select_nonexistent_fails() {
    let mut bucket = ProfilesBucket::new();
    let fake_id = LocalProfileId("nope".to_string());
    let result = bucket.select_profile(&fake_id);
    assert_eq!(result, Err(ProfileGateError::ProfileNotFound));
}

#[test]
fn set_default_clears_others() {
    let p1 = make_profile("Alice");
    let p2 = make_profile("Bob");
    let id1 = p1.id.clone();
    let id2 = p2.id.clone();
    let mut bucket = ProfilesBucket::new();
    bucket.add_profile(p1).unwrap();
    bucket.add_profile(p2).unwrap();
    bucket.set_default(&id1).unwrap();
    assert!(bucket.profiles[&id1].is_default);
    assert!(!bucket.profiles[&id2].is_default);
    bucket.set_default(&id2).unwrap();
    assert!(!bucket.profiles[&id1].is_default);
    assert!(bucket.profiles[&id2].is_default);
}

#[test]
fn profiles_sorted_default_first() {
    let mut p1 = make_profile("Alice");
    let p2 = make_profile("Bob");
    p1.is_default = true;
    let mut bucket = ProfilesBucket::new();
    bucket.add_profile(p1.clone()).unwrap();
    bucket.add_profile(p2).unwrap();
    let sorted = bucket.profiles_sorted();
    assert_eq!(sorted[0].id, p1.id);
}
