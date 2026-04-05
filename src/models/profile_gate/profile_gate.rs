use futures::FutureExt;
use serde::Serialize;

use crate::constants::PROFILES_STORAGE_KEY;
use crate::models::ctx::CtxError;
use crate::runtime::msg::{Action, ActionProfileGate, Event, Internal, Msg};
use crate::runtime::{EffectFuture, Effects, Env, EnvFutureExt, Update};
use crate::types::profile_gate::{
    LocalProfile, LocalProfileId, PinConfig, ProfileGateError, ProfilesBucket,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum GateStatus {
    SelectingProfile,
    AwaitingPin {
        profile_id: LocalProfileId,
        error: Option<ProfileGateError>,
    },
    LoadingProfile(LocalProfileId),
    Inactive,
    ManagingProfiles,
    CreatingProfile,
    EditingProfile(LocalProfileId),
}

impl Default for GateStatus {
    fn default() -> Self {
        GateStatus::Inactive
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileGate {
    pub profiles_bucket: ProfilesBucket,
    pub status: GateStatus,
    pub profiles_list: Vec<LocalProfile>,
}

impl Default for ProfileGate {
    fn default() -> Self {
        ProfileGate::new(ProfilesBucket::default())
    }
}

impl ProfileGate {
    pub fn new(bucket: ProfilesBucket) -> Self {
        let status = determine_initial_status(&bucket);
        let mut gate = ProfileGate {
            profiles_bucket: bucket,
            status,
            profiles_list: vec![],
        };
        gate.refresh_profiles_list();
        gate
    }

    fn refresh_profiles_list(&mut self) {
        self.profiles_list = self
            .profiles_bucket
            .profiles_sorted()
            .into_iter()
            .cloned()
            .collect();
    }
}

fn determine_initial_status(bucket: &ProfilesBucket) -> GateStatus {
    if !bucket.settings.enabled {
        return GateStatus::Inactive;
    }

    if bucket.settings.auto_select_default && bucket.active_profile_id.is_none() {
        // Try to find a default profile with no PIN
        let default_no_pin = bucket
            .profiles
            .values()
            .find(|p| p.is_default && !p.requires_pin())
            .map(|p| p.id.clone());
        if let Some(id) = default_no_pin {
            return GateStatus::LoadingProfile(id);
        }
        return GateStatus::SelectingProfile;
    }

    if bucket.active_profile_id.is_some() {
        return GateStatus::Inactive;
    }

    GateStatus::SelectingProfile
}

impl<E: Env + 'static> Update<E> for ProfileGate {
    fn update(&mut self, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::ProfileGate(action)) => match action {
                ActionProfileGate::ShowGate => {
                    self.status = GateStatus::SelectingProfile;
                    Effects::none()
                }
                ActionProfileGate::DismissGate => {
                    self.status = GateStatus::Inactive;
                    Effects::none()
                }
                ActionProfileGate::ShowManageProfiles => {
                    self.status = GateStatus::ManagingProfiles;
                    Effects::none()
                }
                ActionProfileGate::SwitchProfile => {
                    self.status = GateStatus::SelectingProfile;
                    Effects::none()
                        .join(Effects::msg(Msg::Internal(Internal::SaveCurrentProfileData)).unchanged())
                }
                ActionProfileGate::SelectProfile { profile_id, pin } => {
                    let profile = match self.profiles_bucket.profiles.get(profile_id) {
                        Some(p) => p.clone(),
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    if profile.requires_pin() {
                        match pin {
                            None => {
                                self.status = GateStatus::AwaitingPin {
                                    profile_id: profile_id.clone(),
                                    error: None,
                                };
                                Effects::none()
                            }
                            Some(p) => {
                                let pin_config = match &profile.pin {
                                    Some(pc) => pc.clone(),
                                    None => {
                                        return Effects::msg(Msg::Event(
                                            Event::ProfileGateError(
                                                ProfileGateError::ProfileNotFound,
                                            ),
                                        ))
                                        .unchanged();
                                    }
                                };

                                let now = E::now();
                                if pin_config.is_locked_out(&now) {
                                    return Effects::msg(Msg::Event(Event::ProfileGateError(
                                        ProfileGateError::ProfileLockedOut,
                                    )))
                                    .unchanged();
                                }

                                if pin_config.verify(p) {
                                    // Correct PIN: reset failed attempts
                                    if let Some(profile_mut) =
                                        self.profiles_bucket.profiles.get_mut(profile_id)
                                    {
                                        if let Some(pc) = &mut profile_mut.pin {
                                            pc.failed_attempts = 0;
                                            pc.last_failed_at = None;
                                        }
                                    }
                                    self.status =
                                        GateStatus::LoadingProfile(profile_id.clone());
                                    Effects::none()
                                        .join(save_profiles_bucket::<E>(
                                            &self.profiles_bucket,
                                        ))
                                        .join(
                                            Effects::msg(Msg::Event(Event::ProfileSelected {
                                                profile_id: profile_id.clone(),
                                            }))
                                            .unchanged(),
                                        )
                                        .join(
                                            Effects::msg(Msg::Internal(
                                                Internal::LoadProfileData(profile_id.clone()),
                                            ))
                                            .unchanged(),
                                        )
                                } else {
                                    // Wrong PIN: increment failed attempts
                                    if let Some(profile_mut) =
                                        self.profiles_bucket.profiles.get_mut(profile_id)
                                    {
                                        if let Some(pc) = &mut profile_mut.pin {
                                            pc.failed_attempts += 1;
                                            pc.last_failed_at = Some(now);
                                        }
                                    }
                                    self.status = GateStatus::AwaitingPin {
                                        profile_id: profile_id.clone(),
                                        error: Some(ProfileGateError::PinIncorrect),
                                    };
                                    Effects::none().join(save_profiles_bucket::<E>(
                                        &self.profiles_bucket,
                                    ))
                                }
                            }
                        }
                    } else {
                        // No PIN required
                        self.status = GateStatus::LoadingProfile(profile_id.clone());
                        Effects::none()
                            .join(save_profiles_bucket::<E>(&self.profiles_bucket))
                            .join(
                                Effects::msg(Msg::Event(Event::ProfileSelected {
                                    profile_id: profile_id.clone(),
                                }))
                                .unchanged(),
                            )
                            .join(
                                Effects::msg(Msg::Internal(Internal::LoadProfileData(
                                    profile_id.clone(),
                                )))
                                .unchanged(),
                            )
                    }
                }
                ActionProfileGate::CreateProfile {
                    name,
                    avatar,
                    pin,
                    is_kids,
                    color,
                } => {
                    if let Some(pin_str) = pin {
                        if !is_valid_pin(pin_str) {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::InvalidPin,
                            )))
                            .unchanged();
                        }
                    }

                    let now = E::now();
                    let mut profile = LocalProfile::new(name.clone(), now);

                    if let Some(av) = avatar {
                        profile.avatar = av.clone();
                    }
                    profile.is_kids = *is_kids;
                    if let Some(c) = color {
                        profile.color = Some(c.clone());
                    }
                    if let Some(pin_str) = pin {
                        profile.pin = Some(PinConfig::new(pin_str));
                    }

                    let profile_id = profile.id.clone();

                    match self.profiles_bucket.add_profile(profile) {
                        Err(error) => {
                            Effects::msg(Msg::Event(Event::ProfileGateError(error))).unchanged()
                        }
                        Ok(()) => {
                            self.refresh_profiles_list();
                            Effects::none()
                                .join(save_profiles_bucket::<E>(&self.profiles_bucket))
                                .join(
                                    Effects::msg(Msg::Event(Event::ProfileCreated {
                                        profile_id,
                                    }))
                                    .unchanged(),
                                )
                        }
                    }
                }
                ActionProfileGate::UpdateProfile {
                    profile_id,
                    name,
                    avatar,
                    is_kids,
                    color,
                } => {
                    let profile_mut = match self.profiles_bucket.profiles.get_mut(profile_id) {
                        Some(p) => p,
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    if let Some(n) = name {
                        profile_mut.name = n.clone();
                    }
                    if let Some(av) = avatar {
                        profile_mut.avatar = av.clone();
                    }
                    if let Some(k) = is_kids {
                        profile_mut.is_kids = *k;
                    }
                    if let Some(c) = color {
                        profile_mut.color = Some(c.clone());
                    }

                    self.refresh_profiles_list();
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
                ActionProfileGate::DeleteProfile { profile_id } => {
                    match self.profiles_bucket.remove_profile(profile_id) {
                        Err(error) => {
                            Effects::msg(Msg::Event(Event::ProfileGateError(error))).unchanged()
                        }
                        Ok(()) => {
                            self.refresh_profiles_list();
                            Effects::none()
                                .join(save_profiles_bucket::<E>(&self.profiles_bucket))
                        }
                    }
                }
                ActionProfileGate::SetPin { profile_id, pin } => {
                    if !is_valid_pin(pin) {
                        return Effects::msg(Msg::Event(Event::ProfileGateError(
                            ProfileGateError::InvalidPin,
                        )))
                        .unchanged();
                    }

                    let profile_mut = match self.profiles_bucket.profiles.get_mut(profile_id) {
                        Some(p) => p,
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    profile_mut.pin = Some(PinConfig::new(pin));
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
                ActionProfileGate::RemovePin { profile_id } => {
                    let profile_mut = match self.profiles_bucket.profiles.get_mut(profile_id) {
                        Some(p) => p,
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    profile_mut.pin = None;
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
                ActionProfileGate::UpdateSettings(settings) => {
                    self.profiles_bucket.settings = settings.clone();
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
                ActionProfileGate::SetDefaultProfile { profile_id } => {
                    match self.profiles_bucket.set_default(profile_id) {
                        Err(error) => {
                            Effects::msg(Msg::Event(Event::ProfileGateError(error))).unchanged()
                        }
                        Ok(()) => {
                            self.refresh_profiles_list();
                            Effects::none()
                                .join(save_profiles_bucket::<E>(&self.profiles_bucket))
                        }
                    }
                }
                ActionProfileGate::LinkAccount {
                    profile_id,
                    auth_key,
                } => {
                    let profile_mut = match self.profiles_bucket.profiles.get_mut(profile_id) {
                        Some(p) => p,
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    profile_mut.auth_key = Some(auth_key.clone());
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
                ActionProfileGate::UnlinkAccount { profile_id } => {
                    let profile_mut = match self.profiles_bucket.profiles.get_mut(profile_id) {
                        Some(p) => p,
                        None => {
                            return Effects::msg(Msg::Event(Event::ProfileGateError(
                                ProfileGateError::ProfileNotFound,
                            )))
                            .unchanged();
                        }
                    };

                    profile_mut.auth_key = None;
                    profile_mut.uid = None;
                    Effects::none().join(save_profiles_bucket::<E>(&self.profiles_bucket))
                }
            },
            Msg::Internal(Internal::ProfilesBucketLoaded(bucket)) => {
                let loaded = bucket.clone().unwrap_or_default();
                let new_gate = ProfileGate::new(loaded);
                self.profiles_bucket = new_gate.profiles_bucket;
                self.status = new_gate.status;
                self.profiles_list = new_gate.profiles_list;
                Effects::none()
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn save_profiles_bucket<E: Env + 'static>(bucket: &ProfilesBucket) -> Effects {
    Effects::one(
        EffectFuture::Sequential(
            E::set_storage(PROFILES_STORAGE_KEY, Some(bucket))
                .map(|result| match result {
                    Ok(_) => Msg::Event(Event::ProfilesBucketSaved),
                    Err(error) => Msg::Event(Event::Error {
                        error: CtxError::from(error),
                        source: Box::new(Event::ProfilesBucketSaved),
                    }),
                })
                .boxed_env(),
        )
        .into(),
    )
    .unchanged()
}

fn is_valid_pin(pin: &str) -> bool {
    pin.len() >= 4 && pin.len() <= 6 && pin.chars().all(|c| c.is_ascii_digit())
}
