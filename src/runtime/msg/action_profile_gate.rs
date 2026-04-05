use serde::Deserialize;

use crate::types::profile_gate::{Avatar, LocalProfileId, ProfileGateSettings};

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "action", content = "args")]
pub enum ActionProfileGate {
    ShowGate,
    DismissGate,
    ShowManageProfiles,

    #[serde(rename_all = "camelCase")]
    SelectProfile {
        profile_id: LocalProfileId,
        #[serde(default)]
        pin: Option<String>,
    },

    SwitchProfile,

    #[serde(rename_all = "camelCase")]
    CreateProfile {
        name: String,
        #[serde(default)]
        avatar: Option<Avatar>,
        #[serde(default)]
        pin: Option<String>,
        #[serde(default)]
        is_kids: bool,
        #[serde(default)]
        color: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    UpdateProfile {
        profile_id: LocalProfileId,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        avatar: Option<Avatar>,
        #[serde(default)]
        is_kids: Option<bool>,
        #[serde(default)]
        color: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    DeleteProfile { profile_id: LocalProfileId },

    #[serde(rename_all = "camelCase")]
    SetPin {
        profile_id: LocalProfileId,
        pin: String,
    },

    #[serde(rename_all = "camelCase")]
    RemovePin { profile_id: LocalProfileId },

    UpdateSettings(ProfileGateSettings),

    #[serde(rename_all = "camelCase")]
    SetDefaultProfile { profile_id: LocalProfileId },

    #[serde(rename_all = "camelCase")]
    LinkAccount {
        profile_id: LocalProfileId,
        auth_key: String,
    },

    #[serde(rename_all = "camelCase")]
    UnlinkAccount { profile_id: LocalProfileId },
}
