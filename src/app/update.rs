//! The Check for updates result in Settings, About (ADR-0011). Pure: the network call and the JSON
//! live in `platform::update_check`.

use crate::app::version::Version;

/// Where the release pages live. The Download button opens a page under this address only.
pub const RELEASES: &str = "https://github.com/sanamhub/mitibar/releases";

/// What the About section shows next to the button. Before the first press the window shows its
/// default, code 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The request is out.
    Checking,
    /// This copy is the latest release, or newer than it (a test build).
    Latest,
    /// A newer release exists.
    Newer(Version),
    /// GitHub could not be reached or answered something unexpected.
    Failed,
}

impl UpdateStatus {
    /// The code `SettingsWindow::update-status` reads.
    pub fn code(&self) -> i32 {
        match self {
            UpdateStatus::Checking => 1,
            UpdateStatus::Latest => 2,
            UpdateStatus::Newer(_) => 3,
            UpdateStatus::Failed => 4,
        }
    }

    /// The newer version as text, or empty.
    pub fn version_text(&self) -> String {
        match self {
            UpdateStatus::Newer(version) => version.to_string(),
            _ => String::new(),
        }
    }
}

/// The result for this copy, `current`, given the latest release tag GitHub named.
pub fn outcome(current: &Version, latest_tag: Option<&str>) -> UpdateStatus {
    match latest_tag.and_then(Version::parse) {
        Some(latest) if latest > *current => UpdateStatus::Newer(latest),
        Some(_) => UpdateStatus::Latest,
        None => UpdateStatus::Failed,
    }
}

/// The release page of `version`. It is built from the parsed version, never taken from the
/// response, so the button can only open this project's releases.
pub fn release_page(version: &Version) -> String {
    format!("{RELEASES}/tag/v{version}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    #[test]
    fn a_newer_tag_is_offered() {
        let status = outcome(&v("1.0.0"), Some("v1.1.0"));
        assert_eq!(status, UpdateStatus::Newer(v("1.1.0")));
        assert_eq!(status.code(), 3);
        assert_eq!(status.version_text(), "1.1.0");
    }

    #[test]
    fn the_same_or_an_older_tag_is_up_to_date() {
        assert_eq!(outcome(&v("1.0.0"), Some("v1.0.0")), UpdateStatus::Latest);
        assert_eq!(
            outcome(&v("1.1.0-beta.1"), Some("v1.0.0")),
            UpdateStatus::Latest
        );
    }

    #[test]
    fn a_pre_release_copy_is_offered_its_release() {
        assert_eq!(
            outcome(&v("1.1.0-beta.1"), Some("v1.1.0")),
            UpdateStatus::Newer(v("1.1.0"))
        );
    }

    #[test]
    fn a_missing_or_strange_tag_is_a_failed_check() {
        assert_eq!(outcome(&v("1.0.0"), None), UpdateStatus::Failed);
        assert_eq!(outcome(&v("1.0.0"), Some("nightly")), UpdateStatus::Failed);
        assert_eq!(
            outcome(&v("1.0.0"), Some("v2.0.0/../../evil")),
            UpdateStatus::Failed
        );
    }

    #[test]
    fn the_download_page_is_always_this_projects_release() {
        assert_eq!(
            release_page(&v("1.1.0")),
            "https://github.com/sanamhub/mitibar/releases/tag/v1.1.0"
        );
    }
}
