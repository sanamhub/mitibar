//! The update requests (ADR-0011, ADR-0012): when the user presses Check for updates, one HTTPS GET
//! to GitHub's latest-release API on a background thread. Nothing else is sent: no identifier, no
//! settings, no usage data. The User-Agent names the app and its version, which GitHub's API
//! requires. Certificates are checked against the operating system's store. The same agent serves
//! the asset downloads in `platform::update_download`.

use std::sync::Arc;
use std::time::Duration;

use crate::app::update::{self, UpdateStatus};
use crate::app::version;

const LATEST_RELEASE: &str = "https://api.github.com/repos/sanamhub/mitibar/releases/latest";
const TIMEOUT: Duration = Duration::from_secs(15);
/// The latest-release answer is a few kilobytes; anything far larger is not it.
const BODY_LIMIT: u64 = 256 * 1024;

/// Asks GitHub for the latest release on a new thread and hands the result to `done` there.
pub fn spawn(done: impl FnOnce(UpdateStatus) + Send + 'static) {
    std::thread::spawn(move || {
        let tag = match latest_tag() {
            Ok(tag) => Some(tag),
            Err(err) => {
                log::warn!("update check: {err}");
                None
            }
        };
        done(update::outcome(&version::current(), tag.as_deref()));
    });
}

fn latest_tag() -> anyhow::Result<String> {
    let body = agent(TIMEOUT)
        .get(LATEST_RELEASE)
        .header("Accept", "application/vnd.github+json")
        .call()?
        .into_body()
        .with_config()
        .limit(BODY_LIMIT)
        .read_to_string()?;
    tag_name(&body).ok_or_else(|| anyhow::anyhow!("no tag_name in the answer"))
}

/// The agent every update request uses (ADR-0011, ADR-0012): ureq with rustls (the `ring`
/// provider) and certificates checked by the operating system's verifier, HTTPS only, redirects
/// followed, no compression or cookies. `timeout` bounds the whole request.
pub(crate) fn agent(timeout: Duration) -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::Rustls)
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(timeout))
        .user_agent(format!("MitiBar/{}", version::current()))
        .https_only(true)
        .build()
        .into()
}

/// The `tag_name` of a GitHub release object.
fn tag_name(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("tag_name")?.as_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_tag_of_a_release_object() {
        let body = r#"{"tag_name":"v1.1.0","name":"MitiBar 1.1.0","draft":false}"#;
        assert_eq!(tag_name(body).as_deref(), Some("v1.1.0"));
    }

    /// Talks to GitHub, so it runs only on request: `cargo test -- --ignored live_`.
    #[test]
    #[ignore = "needs the internet"]
    fn live_github_names_a_release_tag() {
        let tag = latest_tag().expect("GitHub answered");
        assert!(crate::app::version::Version::parse(&tag).is_some(), "{tag}");
    }

    #[test]
    fn an_error_object_or_junk_has_no_tag() {
        assert_eq!(tag_name(r#"{"message":"Not Found"}"#), None);
        assert_eq!(tag_name("<html>rate limited</html>"), None);
        assert_eq!(tag_name(r#"{"tag_name":7}"#), None);
    }
}
