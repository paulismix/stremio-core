use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::local_profile::{LocalProfile, LocalProfileId, MAX_PROFILES};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileGateSettings {
    pub enabled: bool,
    pub auto_select_default: bool,
    pub require_pin_on_switch: bool,
}

impl Default for ProfileGateSettings {
    fn default() -> Self {
        ProfileGateSettings {
            enabled: false,
            auto_select_default: false,
            require_pin_on_switch: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesBucket {
    pub profiles: HashMap<LocalProfileId, LocalProfile>,
    pub active_profile_id: Option<LocalProfileId>,
    pub settings: ProfileGateSettings,
}

impl Default for ProfilesBucket {
    fn default() -> Self {
        ProfilesBucket::new()
    }
}

impl ProfilesBucket {
    pub fn new() -> Self {
        ProfilesBucket {
            profiles: HashMap::new(),
            active_profile_id: None,
            settings: ProfileGateSettings::default(),
        }
    }

    pub fn with_initial_profile(profile: LocalProfile) -> Self {
        let id = profile.id.clone();
        let mut profiles = HashMap::new();
        profiles.insert(id.clone(), profile);
        ProfilesBucket {
            profiles,
            active_profile_id: Some(id),
            settings: ProfileGateSettings::default(),
        }
    }

    pub fn active_profile(&self) -> Option<&LocalProfile> {
        self.active_profile_id
            .as_ref()
            .and_then(|id| self.profiles.get(id))
    }

    pub fn active_profile_mut(&mut self) -> Option<&mut LocalProfile> {
        self.active_profile_id
            .as_ref()
            .and_then(|id| self.profiles.get_mut(id))
    }

    pub fn profiles_sorted(&self) -> Vec<&LocalProfile> {
        let mut sorted: Vec<&LocalProfile> = self.profiles.values().collect();
        sorted.sort_by(|a, b| {
            b.is_default
                .cmp(&a.is_default)
                .then(a.created_at.cmp(&b.created_at))
        });
        sorted
    }

    pub fn add_profile(&mut self, profile: LocalProfile) -> Result<(), ProfileGateError> {
        if self.profiles.len() >= MAX_PROFILES {
            return Err(ProfileGateError::MaxProfilesReached);
        }
        self.profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    pub fn remove_profile(&mut self, id: &LocalProfileId) -> Result<(), ProfileGateError> {
        if self.profiles.len() <= 1 {
            return Err(ProfileGateError::CannotRemoveLastProfile);
        }
        if !self.profiles.contains_key(id) {
            return Err(ProfileGateError::ProfileNotFound);
        }
        self.profiles.remove(id);
        // If the removed profile was active, clear active selection
        if self.active_profile_id.as_ref() == Some(id) {
            self.active_profile_id = None;
            self.auto_select_profile();
        }
        Ok(())
    }

    pub fn select_profile(&mut self, id: &LocalProfileId) -> Result<(), ProfileGateError> {
        if !self.profiles.contains_key(id) {
            return Err(ProfileGateError::ProfileNotFound);
        }
        self.active_profile_id = Some(id.clone());
        Ok(())
    }

    pub fn auto_select_profile(&mut self) {
        let default_id = self
            .profiles
            .values()
            .find(|p| p.is_default)
            .map(|p| p.id.clone());
        if let Some(id) = default_id {
            self.active_profile_id = Some(id);
            return;
        }
        // Fall back to the first profile by creation time
        let first_id = self
            .profiles
            .values()
            .min_by_key(|p| p.created_at)
            .map(|p| p.id.clone());
        self.active_profile_id = first_id;
    }

    pub fn set_default(&mut self, id: &LocalProfileId) -> Result<(), ProfileGateError> {
        if !self.profiles.contains_key(id) {
            return Err(ProfileGateError::ProfileNotFound);
        }
        // Clear existing default
        for profile in self.profiles.values_mut() {
            profile.is_default = false;
        }
        if let Some(profile) = self.profiles.get_mut(id) {
            profile.is_default = true;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ProfileGateError {
    MaxProfilesReached,
    CannotRemoveLastProfile,
    ProfileNotFound,
    PinRequired,
    PinIncorrect,
    ProfileLockedOut,
    InvalidPin,
}

impl fmt::Display for ProfileGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileGateError::MaxProfilesReached => {
                write!(f, "Maximum number of profiles reached")
            }
            ProfileGateError::CannotRemoveLastProfile => {
                write!(f, "Cannot remove the last profile")
            }
            ProfileGateError::ProfileNotFound => write!(f, "Profile not found"),
            ProfileGateError::PinRequired => write!(f, "PIN required to access this profile"),
            ProfileGateError::PinIncorrect => write!(f, "Incorrect PIN"),
            ProfileGateError::ProfileLockedOut => {
                write!(f, "Profile is locked out due to too many failed attempts")
            }
            ProfileGateError::InvalidPin => write!(f, "Invalid PIN format"),
        }
    }
}
