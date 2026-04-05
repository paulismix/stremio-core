use super::local_profile::LocalProfileId;

pub fn profile_key(id: &LocalProfileId, suffix: &str) -> String {
    format!("profile:{}:{}", id, suffix)
}

pub mod keys {
    pub const PROFILE: &str = "profile";
    pub const LIBRARY_RECENT: &str = "library_recent";
    pub const LIBRARY: &str = "library";
    pub const STREAMS: &str = "streams";
    pub const NOTIFICATIONS: &str = "notifications";
    pub const SEARCH_HISTORY: &str = "search_history";
    pub const STREAMING_SERVER_URLS: &str = "streaming_server_urls";
    pub const DISMISSED_EVENTS: &str = "dismissed_events";

    pub const ALL: &[&str] = &[
        PROFILE,
        LIBRARY_RECENT,
        LIBRARY,
        STREAMS,
        NOTIFICATIONS,
        SEARCH_HISTORY,
        STREAMING_SERVER_URLS,
        DISMISSED_EVENTS,
    ];
}

pub fn all_profile_keys(id: &LocalProfileId) -> Vec<String> {
    keys::ALL
        .iter()
        .map(|suffix| profile_key(id, suffix))
        .collect()
}
