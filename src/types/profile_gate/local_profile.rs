use std::fmt;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::types::profile::UID;

pub const MAX_PROFILES: usize = 8;
pub const MAX_FAILED_ATTEMPTS: u32 = 5;
pub const LOCKOUT_DURATION_SECS: i64 = 300;

const PIN_SALT: &str = "stremio-profile-gate-v1";

// TODO: replace with Env::generate_id()
fn generate_id() -> String {
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:016x}{:08x}", dur.as_secs(), dur.subsec_nanos())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalProfileId(pub String);

impl Default for LocalProfileId {
    fn default() -> Self {
        LocalProfileId(generate_id())
    }
}

impl fmt::Display for LocalProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for LocalProfileId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Avatar {
    Preset(String),
    Url(String),
    Account,
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinConfig {
    pub pin_hash: String,
    pub enabled: bool,
    pub failed_attempts: u32,
    pub last_failed_at: Option<DateTime<Utc>>,
}

impl PinConfig {
    pub fn new(raw_pin: &str) -> Self {
        PinConfig {
            pin_hash: Self::hash_pin(raw_pin),
            enabled: true,
            failed_attempts: 0,
            last_failed_at: None,
        }
    }

    fn hash_pin(raw_pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(PIN_SALT.as_bytes());
        hasher.update(b":");
        hasher.update(raw_pin.as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    pub fn verify(&self, attempt: &str) -> bool {
        self.pin_hash == Self::hash_pin(attempt)
    }

    pub fn is_locked_out(&self, now: &DateTime<Utc>) -> bool {
        if self.failed_attempts < MAX_FAILED_ATTEMPTS {
            return false;
        }
        match self.last_failed_at {
            Some(last_failed) => {
                let elapsed = now.signed_duration_since(last_failed).num_seconds();
                elapsed < LOCKOUT_DURATION_SECS
            }
            None => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalProfile {
    pub id: LocalProfileId,
    pub name: String,
    pub avatar: Avatar,
    pub pin: Option<PinConfig>,
    pub uid: UID,
    pub auth_key: Option<String>,
    pub is_kids: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub color: Option<String>,
}

impl LocalProfile {
    pub fn new(name: String, now: DateTime<Utc>) -> Self {
        LocalProfile {
            id: LocalProfileId::default(),
            name,
            avatar: Avatar::Default,
            pin: None,
            uid: None,
            auth_key: None,
            is_kids: false,
            is_default: false,
            created_at: now,
            last_used_at: now,
            color: None,
        }
    }

    pub fn new_linked(
        name: String,
        uid: UID,
        auth_key: Option<String>,
        now: DateTime<Utc>,
    ) -> Self {
        LocalProfile {
            id: LocalProfileId::default(),
            name,
            avatar: Avatar::Account,
            pin: None,
            uid,
            auth_key,
            is_kids: false,
            is_default: true,
            created_at: now,
            last_used_at: now,
            color: None,
        }
    }

    pub fn requires_pin(&self) -> bool {
        self.pin.as_ref().map(|p| p.enabled).unwrap_or(false)
    }

    pub fn storage_prefix(&self) -> String {
        format!("profile:{}", self.id)
    }
}
