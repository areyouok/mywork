use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::OnceLock;

static HYDRATED_PATH: OnceLock<Option<OsString>> = OnceLock::new();

pub fn hydrated_path() -> Option<OsString> {
    HYDRATED_PATH
        .get_or_init(|| {
            #[cfg(target_os = "macos")]
            {
                return build_hydrated_path(
                    std::env::var_os("PATH"),
                    login_shell_path(),
                    path_helper_path(),
                );
            }

            #[allow(unreachable_code)]
            std::env::var_os("PATH")
        })
        .clone()
}

fn build_hydrated_path(
    current_path: Option<OsString>,
    login_shell: Option<OsString>,
    path_helper: Option<OsString>,
) -> Option<OsString> {
    let login_entries = parse_path_entries(login_shell.as_deref());
    let helper_entries = parse_path_entries(path_helper.as_deref());
    let current_entries = parse_path_entries(current_path.as_deref());

    let merged = merge_unique_paths([login_entries, helper_entries, current_entries]);
    if merged.is_empty() {
        return None;
    }

    std::env::join_paths(merged).ok()
}

fn parse_path_entries(raw_path: Option<&OsStr>) -> Vec<PathBuf> {
    match raw_path {
        Some(path) => std::env::split_paths(path)
            .filter(|entry| !entry.as_os_str().is_empty())
            .collect(),
        None => Vec::new(),
    }
}

fn merge_unique_paths<const N: usize>(groups: [Vec<PathBuf>; N]) -> Vec<PathBuf> {
    let mut seen = HashSet::<OsString>::new();
    let mut merged = Vec::<PathBuf>::new();

    for paths in groups {
        for entry in paths {
            let key = entry.as_os_str().to_os_string();
            if seen.insert(key) {
                merged.push(entry);
            }
        }
    }

    merged
}

#[cfg(target_os = "macos")]
fn login_shell_path() -> Option<OsString> {
    let output = Command::new("/bin/zsh")
        .args(["-l", "-c", "printf '__MYWORK_PATH__=%s\\n' \"$PATH\""])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let shell_output = String::from_utf8_lossy(&output.stdout);
    parse_prefixed_path_output(&shell_output, "__MYWORK_PATH__=")
}

#[cfg(target_os = "macos")]
fn path_helper_path() -> Option<OsString> {
    let output = Command::new("/usr/libexec/path_helper")
        .arg("-s")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let shell_output = String::from_utf8_lossy(&output.stdout);
    parse_path_helper_output(&shell_output)
}

fn parse_prefixed_path_output(output: &str, prefix: &str) -> Option<OsString> {
    for line in output.lines().rev() {
        if let Some(value) = line.strip_prefix(prefix) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(OsString::from(trimmed));
            }
        }
    }

    None
}

fn parse_path_helper_output(output: &str) -> Option<OsString> {
    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("PATH=\"") {
            if let Some((value, _)) = rest.split_once("\";") {
                if !value.is_empty() {
                    return Some(OsString::from(value));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path_helper_output_extracts_path() {
        let output = "PATH=\"/usr/local/bin:/usr/bin:/bin\"; export PATH;\nMANPATH=\"/usr/share/man\"; export MANPATH;\n";

        let parsed = parse_path_helper_output(output);

        assert_eq!(parsed, Some(OsString::from("/usr/local/bin:/usr/bin:/bin")));
    }

    #[test]
    fn test_parse_prefixed_path_output_extracts_last_prefixed_line() {
        let output = "hello\n__MYWORK_PATH__=/usr/local/bin:/usr/bin\nnoise\n__MYWORK_PATH__=/opt/homebrew/bin:/usr/bin\n";

        let parsed = parse_prefixed_path_output(output, "__MYWORK_PATH__=");

        assert_eq!(parsed, Some(OsString::from("/opt/homebrew/bin:/usr/bin")));
    }

    #[test]
    fn test_parse_prefixed_path_output_returns_none_when_missing() {
        let output = "hello\nworld\n";

        let parsed = parse_prefixed_path_output(output, "__MYWORK_PATH__=");

        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_path_helper_output_returns_none_without_path() {
        let output = "MANPATH=\"/usr/share/man\"; export MANPATH;\n";

        let parsed = parse_path_helper_output(output);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_build_hydrated_path_prefers_login_and_dedupes() {
        let current = Some(OsString::from("/usr/bin:/bin"));
        let login = Some(OsString::from("/opt/homebrew/bin:/usr/bin:/bin"));
        let helper = Some(OsString::from("/usr/local/MacGPG2/bin:/usr/bin:/bin"));

        let hydrated = build_hydrated_path(current, login, helper);

        let entries = parse_path_entries(hydrated.as_deref());
        let expected = vec![
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
            PathBuf::from("/usr/local/MacGPG2/bin"),
        ];
        assert_eq!(entries, expected);
    }

    #[test]
    fn test_build_hydrated_path_falls_back_to_current() {
        let current = Some(OsString::from("/usr/bin:/bin"));

        let hydrated = build_hydrated_path(current.clone(), None, None);

        assert_eq!(hydrated, current);
    }

    #[test]
    fn test_build_hydrated_path_none_when_all_empty() {
        let hydrated = build_hydrated_path(None, None, None);

        assert!(hydrated.is_none());
    }
}
