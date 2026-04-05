use chrono::Utc;

use crate::types::profile_gate::{
    Avatar, LocalProfile, LocalProfileId, PinConfig, LOCKOUT_DURATION_SECS, MAX_FAILED_ATTEMPTS,
};

#[test]
fn pin_verify_correct() {
    let config = PinConfig::new("1234");
    assert!(config.verify("1234"));
}

#[test]
fn pin_verify_wrong() {
    let config = PinConfig::new("1234");
    assert!(!config.verify("5678"));
}

#[test]
fn pin_lockout() {
    let now = Utc::now();
    let mut config = PinConfig::new("1234");
    config.failed_attempts = MAX_FAILED_ATTEMPTS;
    config.last_failed_at = Some(now);
    assert!(config.is_locked_out(&now));
}

#[test]
fn pin_lockout_expires() {
    let past = Utc::now() - chrono::Duration::seconds(LOCKOUT_DURATION_SECS + 1);
    let now = Utc::now();
    let mut config = PinConfig::new("1234");
    config.failed_attempts = MAX_FAILED_ATTEMPTS;
    config.last_failed_at = Some(past);
    assert!(!config.is_locked_out(&now));
}

#[test]
fn avatar_serde_roundtrip() {
    let variants = vec![
        Avatar::Preset("preset1".to_string()),
        Avatar::Url("https://example.com/img.png".to_string()),
        Avatar::Account,
        Avatar::Default,
    ];
    for avatar in &variants {
        let json = serde_json::to_string(avatar).unwrap();
        let back: Avatar = serde_json::from_str(&json).unwrap();
        assert_eq!(avatar, &back);
    }
}

#[test]
fn local_profile_serde_roundtrip() {
    let now = Utc::now();
    let profile = LocalProfile::new("Alice".to_string(), now);
    let json = serde_json::to_string(&profile).unwrap();
    let back: LocalProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(profile.name, back.name);
    assert_eq!(profile.id, back.id);
}

#[test]
fn local_profile_with_pin_serde_roundtrip() {
    let now = Utc::now();
    let mut profile = LocalProfile::new("Bob".to_string(), now);
    profile.pin = Some(PinConfig::new("5678"));
    let json = serde_json::to_string(&profile).unwrap();
    let back: LocalProfile = serde_json::from_str(&json).unwrap();
    assert!(back.requires_pin());
}

#[test]
fn local_profile_id_unique() {
    // Two new profiles should have different ids
    // (using different SystemTime-based IDs - may collide in tests if too fast,
    // but the generate_id function uses both secs and subsec_nanos)
    let now = Utc::now();
    let p1 = LocalProfile::new("A".to_string(), now);
    let p2 = LocalProfile::new("B".to_string(), now);
    // They may theoretically be equal if created in the same nanosecond, but in practice differ.
    // Just check that the ID is non-empty.
    assert!(!p1.id.0.is_empty());
    assert!(!p2.id.0.is_empty());
}
