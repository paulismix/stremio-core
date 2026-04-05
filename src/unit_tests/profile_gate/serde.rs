use crate::runtime::msg::ActionProfileGate;
use crate::types::profile_gate::{LocalProfileId, ProfilesBucket};

#[test]
fn action_select_profile_deserialize() {
    let json = serde_json::json!({
        "action": "SelectProfile",
        "args": {
            "profileId": "abc123"
        }
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::SelectProfile { .. }));
}

#[test]
fn action_create_profile_deserialize() {
    let json = serde_json::json!({
        "action": "CreateProfile",
        "args": {
            "name": "Alice"
        }
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::CreateProfile { .. }));
}

#[test]
fn action_set_pin_deserialize() {
    let json = serde_json::json!({
        "action": "SetPin",
        "args": {
            "profileId": "xyz",
            "pin": "1234"
        }
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::SetPin { .. }));
}

#[test]
fn profiles_bucket_roundtrip() {
    let mut bucket = ProfilesBucket::new();
    bucket.settings.enabled = true;
    let json = serde_json::to_string(&bucket).unwrap();
    let back: ProfilesBucket = serde_json::from_str(&json).unwrap();
    assert_eq!(back.settings.enabled, true);
}

#[test]
fn action_select_profile_with_pin_deserialize() {
    let json = serde_json::json!({
        "action": "SelectProfile",
        "args": {
            "profileId": "abc123",
            "pin": "1234"
        }
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    match action {
        ActionProfileGate::SelectProfile { profile_id, pin } => {
            assert_eq!(profile_id, LocalProfileId("abc123".to_string()));
            assert_eq!(pin, Some("1234".to_string()));
        }
        _ => panic!("Expected SelectProfile"),
    }
}

#[test]
fn action_delete_profile_deserialize() {
    let json = serde_json::json!({
        "action": "DeleteProfile",
        "args": {
            "profileId": "some-id"
        }
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::DeleteProfile { .. }));
}

#[test]
fn action_dismiss_gate_deserialize() {
    let json = serde_json::json!({
        "action": "DismissGate",
        "args": null
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::DismissGate));
}

#[test]
fn action_show_gate_deserialize() {
    let json = serde_json::json!({
        "action": "ShowGate",
        "args": null
    });
    let action: ActionProfileGate = serde_json::from_value(json).unwrap();
    assert!(matches!(action, ActionProfileGate::ShowGate));
}
