pub fn parse_session_id(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Session ID:")
            || trimmed.starts_with("Session:")
            || trimmed.starts_with("session:")
        {
            let mut parts = trimmed.splitn(2, ':');
            let _ = parts.next();
            if let Some(value) = parts.next() {
                let session_id = value.trim();
                if !session_id.is_empty() {
                    return Some(session_id.to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::parse_session_id;

    #[test]
    fn parses_session_format() {
        let output = "Session: sess_abc123";
        assert_eq!(parse_session_id(output), Some("sess_abc123".to_string()));
    }

    #[test]
    fn parses_session_id_format() {
        let output = "Session ID: sess_def456";
        assert_eq!(parse_session_id(output), Some("sess_def456".to_string()));
    }

    #[test]
    fn parses_lowercase_format() {
        let output = "session: sess_xyz789";
        assert_eq!(parse_session_id(output), Some("sess_xyz789".to_string()));
    }

    #[test]
    fn returns_none_when_no_session_line() {
        let output = "no session line";
        assert_eq!(parse_session_id(output), None);
    }
}
