//! Release versions, for the two places one copy of the app meets another: the login item that
//! may point at an older copy, and a second copy started while the first runs. Pure.

use std::cmp::Ordering;

/// A `MAJOR.MINOR.PATCH[-PRERELEASE]` version, ordered by the Semantic Versioning 2.0 rules: a
/// pre-release sorts before its release, and pre-release identifiers compare numerically when
/// both are numbers and as text otherwise. Build metadata after `+` is ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Vec<String>,
}

/// This copy's version.
pub fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Version {
        major: 0,
        minor: 0,
        patch: 0,
        pre: Vec::new(),
    })
}

impl Version {
    /// Parses `1.2.3`, `1.2.3-beta.2` or `v1.2.3` (a release tag). `None` for anything else.
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim();
        let text = text.strip_prefix('v').unwrap_or(text);
        let text = text.split('+').next()?;
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, pre.split('.').map(str::to_owned).collect()),
            None => (text, Vec::new()),
        };
        let mut parts = core.split('.').map(str::parse::<u64>);
        let version = Version {
            major: parts.next()?.ok()?,
            minor: parts.next()?.ok()?,
            patch: parts.next()?.ok()?,
            pre,
        };
        if parts.next().is_some() || version.pre.iter().any(String::is_empty) {
            return None;
        }
        Some(version)
    }

    /// The version without its pre-release part, which is all a Windows file version holds.
    pub fn release(&self) -> Version {
        Version {
            pre: Vec::new(),
            ..self.clone()
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre.join("."))?;
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (self.pre.is_empty(), other.pre.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => compare_pre(&self.pre, &other.pre),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_pre(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let order = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => x.cmp(y),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    #[test]
    fn parses_releases_pre_releases_and_tags() {
        assert_eq!(v("1.2.3").to_string(), "1.2.3");
        assert_eq!(v("v1.2.3").to_string(), "1.2.3");
        assert_eq!(v("0.2.0-beta.2").to_string(), "0.2.0-beta.2");
        assert_eq!(v("1.0.0+build.5").to_string(), "1.0.0");
        for bad in ["", "1", "1.2", "1.2.3.4", "1.x.3", "1.2.3-", "1.2.3-a..b"] {
            assert_eq!(Version::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn orders_by_the_semver_rules() {
        let ordered = [
            "0.2.0-beta.2",
            "0.2.0",
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
            "1.0.1",
            "1.1.0",
            "2.0.0",
        ];
        for pair in ordered.windows(2) {
            assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        }
        assert_eq!(v("1.0.0").cmp(&v("v1.0.0")), Ordering::Equal);
    }

    #[test]
    fn release_drops_the_pre_release() {
        assert_eq!(v("1.1.0-beta.1").release(), v("1.1.0"));
    }

    #[test]
    fn the_current_version_is_the_package_version() {
        assert_eq!(current().to_string(), env!("CARGO_PKG_VERSION"));
    }
}
