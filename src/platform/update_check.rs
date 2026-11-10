//! The update requests (ADR-0011, ADR-0012, ADR-0013): one HTTPS GET to GitHub's latest-release
//! API on a background thread. The manual button starts it on request; with the switch on,
//! Settings starts it when it opens, at most once a day. Nothing else is sent: no identifier, no
//! settings, no usage data. The User-Agent names the app and its version, which GitHub's API
//! requires. Certificates are checked against the operating system's store. The same agent serves
//! the asset downloads in `platform::update_download`.

use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::app::update::{self, UpdateStatus};
use crate::app::version;

const LATEST_RELEASE: &str = "https://api.github.com/repos/sanamhub/mitibar/releases/latest";
const TIMEOUT: Duration = Duration::from_secs(15);
/// The latest-release answer is a few kilobytes; anything far larger is not it.
const BODY_LIMIT: u64 = 256 * 1024;

/// What one check got: the result, and whether GitHub answered with a release at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// GitHub returned a release tag, even if the tag itself was not a version this app reads.
    pub answered: bool,
    /// What About shows for this answer, or `Failed` when there was none.
    pub status: UpdateStatus,
}

/// Starts one update check. The app's checker runs the request on a background thread and returns
/// the channel its answer arrives on; a test hands back a filled channel, so no test touches the
/// network.
pub trait Checker {
    /// Starts one check and returns the channel that receives its answer.
    fn start(&self) -> Receiver<Answer>;
}

/// The network checker: the request of ADR-0011 point 2, on a background thread.
pub struct Network;

impl Checker for Network {
    fn start(&self) -> Receiver<Answer> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(check());
        });
        receiver
    }
}

/// One check. `answered` is true only when GitHub returned a release tag: a timeout, a refusal
/// or a body without one is not an answer, so it does not use up the day (ADR-0013).
fn check() -> Answer {
    answer(latest_tag())
}

/// The answer for a release request's result. Split from `check` so a test can feed it a result
/// without the network.
fn answer(tag: anyhow::Result<String>) -> Answer {
    match tag {
        Ok(tag) => Answer {
            answered: true,
            status: update::outcome(&version::current(), Some(&tag)),
        },
        Err(err) => {
            log::warn!("update check: {err}");
            Answer {
                answered: false,
                status: UpdateStatus::Failed,
            }
        }
    }
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

    #[test]
    fn only_a_release_tag_counts_as_an_answer() {
        let answered = answer(Ok("v9.9.9".to_owned()));
        assert!(answered.answered);
        assert_eq!(
            answered.status,
            UpdateStatus::Newer(version::Version::parse("9.9.9").expect("version"))
        );

        let junk = answer(Ok("nightly".to_owned()));
        assert!(
            junk.answered,
            "GitHub answered, but with a tag this app cannot read"
        );
        assert_eq!(junk.status, UpdateStatus::Failed);

        let offline = answer(Err(anyhow::anyhow!("timed out")));
        assert!(!offline.answered, "no answer means no day is used up");
        assert_eq!(offline.status, UpdateStatus::Failed);
    }
}
