use crate::types::profile_gate::{storage_keys, LocalProfileId};

#[test]
fn profile_key_format() {
    let id = LocalProfileId("test123".to_string());
    assert_eq!(
        storage_keys::profile_key(&id, "library"),
        "profile:test123:library"
    );
}

#[test]
fn all_keys_count() {
    let id = LocalProfileId("abc".to_string());
    let keys = storage_keys::all_profile_keys(&id);
    assert_eq!(keys.len(), storage_keys::keys::ALL.len());
}

#[test]
fn keys_unique() {
    let id = LocalProfileId("x".to_string());
    let keys = storage_keys::all_profile_keys(&id);
    let unique: std::collections::HashSet<_> = keys.iter().collect();
    assert_eq!(keys.len(), unique.len());
}

#[test]
fn different_profiles_different_keys() {
    let id1 = LocalProfileId("aaa".to_string());
    let id2 = LocalProfileId("bbb".to_string());
    let k1 = storage_keys::profile_key(&id1, "library");
    let k2 = storage_keys::profile_key(&id2, "library");
    assert_ne!(k1, k2);
}
