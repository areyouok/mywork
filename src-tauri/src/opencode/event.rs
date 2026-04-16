use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenCodeEvent {
    #[serde(rename = "step_start")]
    StepStart {
        timestamp: u64,
        #[serde(rename = "sessionID")]
        session_id: String,
        part: serde_json::Value,
    },
    #[serde(rename = "step_finish")]
    StepFinish {
        timestamp: u64,
        #[serde(rename = "sessionID")]
        session_id: String,
        part: serde_json::Value,
    },
    #[serde(rename = "text")]
    Text {
        timestamp: u64,
        #[serde(rename = "sessionID")]
        session_id: String,
        part: TextPart,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        timestamp: u64,
        #[serde(rename = "sessionID")]
        session_id: String,
        part: ToolUsePart,
    },
}

impl OpenCodeEvent {
    pub fn session_id(&self) -> &str {
        match self {
            OpenCodeEvent::StepStart { session_id, .. } => session_id,
            OpenCodeEvent::StepFinish { session_id, .. } => session_id,
            OpenCodeEvent::Text { session_id, .. } => session_id,
            OpenCodeEvent::ToolUse { session_id, .. } => session_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
    pub time: Option<TimeInfo>,
    pub id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUsePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub tool: String,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub title: Option<String>,
    pub state: ToolState,
    pub id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolState {
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub time: Option<TimeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeInfo {
    pub start: u64,
    pub end: u64,
}

pub fn parse_session_id_from_ndjson(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<OpenCodeEvent>(trimmed) {
            return Some(event.session_id().to_string());
        }
    }
    None
}

pub fn extract_error_message(stdout: &str) -> Option<String> {
    let mut last_error: Option<String> = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(part) = value.get("part") {
                if let Some(reason) = part.get("reason").and_then(|r| r.as_str()) {
                    if reason == "error" {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                last_error = Some(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    last_error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_step_start_event() {
        let json = r#"{"type":"step_start","timestamp":1776342614875,"sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","part":{"id":"prt_d96454357001qBBEHdjJfKDi54","messageID":"msg_d964534f300141F48RpUFuMS4a","sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","snapshot":"6f7bba50bfe8ab0115936f351839f299e05ed83b","type":"step-start"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::StepStart {
                timestamp,
                session_id,
                ..
            } => {
                assert_eq!(timestamp, 1776342614875);
                assert_eq!(session_id, "ses_269bad235ffeiyHgPamGmxM3wz");
            }
            _ => panic!("Expected StepStart"),
        }
    }

    #[test]
    fn test_parse_step_finish_event() {
        let json = r#"{"type":"step_finish","timestamp":1776342619531,"sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","part":{"id":"prt_d9645555c001SCa1K0VYTHeFP7","reason":"stop","snapshot":"6f7bba50bfe8ab0115936f351839f299e05ed83b","messageID":"msg_d964543c2001gV9VKJOIJ5MN4l","sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","type":"step-finish","tokens":{"total":18548,"input":49,"output":3,"reasoning":0,"cache":{"write":0,"read":18496}},"cost":0}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::StepFinish {
                timestamp,
                session_id,
                ..
            } => {
                assert_eq!(timestamp, 1776342619531);
                assert_eq!(session_id, "ses_269bad235ffeiyHgPamGmxM3wz");
            }
            _ => panic!("Expected StepFinish"),
        }
    }

    #[test]
    fn test_parse_text_event() {
        let json = r#"{"type":"text","timestamp":1776342619484,"sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","part":{"id":"prt_d96455559001L64Z1l66KOmvKY","messageID":"msg_d964543c2001gV9VKJOIJ5MN4l","sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","type":"text","text":"hello","time":{"start":1776342619481,"end":1776342619483}}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::Text {
                timestamp,
                session_id,
                part,
            } => {
                assert_eq!(timestamp, 1776342619484);
                assert_eq!(session_id, "ses_269bad235ffeiyHgPamGmxM3wz");
                assert_eq!(part.text, "hello");
                assert_eq!(part.part_type, "text");
                let time = part.time.unwrap();
                assert_eq!(time.start, 1776342619481);
                assert_eq!(time.end, 1776342619483);
            }
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_parse_tool_use_event() {
        let json = r#"{"type":"tool_use","timestamp":1776342614909,"sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","part":{"type":"tool","tool":"bash","callID":"call_200be9f4a5c542009168e31f","state":{"status":"completed","input":{"command":"echo hello","description":"Echo hello"},"output":"hello\n","metadata":{"output":"hello\n","exit":0,"description":"Echo hello","truncated":false},"title":"Echo hello","time":{"start":1776342614905,"end":1776342614908}},"id":"prt_d96454358001ZlKjaLL4IAxIU4","sessionID":"ses_269bad235ffeiyHgPamGmxM3wz","messageID":"msg_d964534f300141F48RpUFuMS4a"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::ToolUse {
                timestamp,
                session_id,
                part,
            } => {
                assert_eq!(timestamp, 1776342614909);
                assert_eq!(session_id, "ses_269bad235ffeiyHgPamGmxM3wz");
                assert_eq!(part.tool, "bash");
                assert_eq!(part.call_id, "call_200be9f4a5c542009168e31f");
                assert_eq!(part.state.status, "completed");
                assert_eq!(part.state.input["command"], "echo hello");
                assert_eq!(part.state.output.as_deref(), Some("hello\n"));
                let time = part.state.time.unwrap();
                assert_eq!(time.start, 1776342614905);
                assert_eq!(time.end, 1776342614908);
            }
            _ => panic!("Expected ToolUse"),
        }
    }

    #[test]
    fn test_session_id_method() {
        let json = r#"{"type":"text","timestamp":1776342619484,"sessionID":"ses_test123","part":{"type":"text","text":"hi"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.session_id(), "ses_test123");
    }

    #[test]
    fn test_camel_case_mapping() {
        let json = r#"{"type":"tool_use","timestamp":1,"sessionID":"ses_cc","part":{"type":"tool","tool":"bash","callID":"call_abc","state":{"status":"running","input":{}}}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        match event {
            OpenCodeEvent::ToolUse { part, .. } => {
                assert_eq!(part.call_id, "call_abc");
            }
            _ => panic!("Expected ToolUse"),
        }
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let result = serde_json::from_str::<OpenCodeEvent>("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_string_returns_error() {
        let result = serde_json::from_str::<OpenCodeEvent>("");
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_event_type_returns_error() {
        let json = r#"{"type":"unknown","timestamp":1,"sessionID":"ses_x"}"#;
        let result = serde_json::from_str::<OpenCodeEvent>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_session_id_from_ndjson_extracts_first_session() {
        let stdout = r#"{"type":"step_start","timestamp":1,"sessionID":"ses_from_json","part":{}}
{"type":"text","timestamp":2,"sessionID":"ses_from_json","part":{"type":"text","text":"hello"}}"#;
        let sid = parse_session_id_from_ndjson(stdout);
        assert_eq!(sid, Some("ses_from_json".to_string()));
    }

    #[test]
    fn test_parse_session_id_from_ndjson_returns_none_on_no_json() {
        let stdout = "not json\nalso not json";
        let sid = parse_session_id_from_ndjson(stdout);
        assert!(sid.is_none());
    }

    #[test]
    fn test_parse_session_id_from_ndjson_skips_bad_lines() {
        let stdout = "bad line\n{\"type\":\"text\",\"timestamp\":1,\"sessionID\":\"ses_skip\",\"part\":{\"type\":\"text\",\"text\":\"hi\"}}";
        let sid = parse_session_id_from_ndjson(stdout);
        assert_eq!(sid, Some("ses_skip".to_string()));
    }

    #[test]
    fn test_parse_session_id_from_ndjson_empty_input() {
        let sid = parse_session_id_from_ndjson("");
        assert!(sid.is_none());
    }

    #[test]
    fn test_text_part_serialization_roundtrip() {
        let part = TextPart {
            part_type: "text".to_string(),
            text: "hello world".to_string(),
            time: Some(TimeInfo {
                start: 100,
                end: 200,
            }),
            id: Some("prt_123".to_string()),
            message_id: Some("msg_456".to_string()),
            session_id: Some("ses_789".to_string()),
        };
        let json = serde_json::to_string(&part).unwrap();
        let deserialized: TextPart = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text, "hello world");
        assert_eq!(deserialized.part_type, "text");
        assert_eq!(deserialized.id, Some("prt_123".to_string()));
    }

    #[test]
    fn test_tool_state_with_optional_fields() {
        let json = r#"{"status":"running","input":{"command":"ls"}}"#;
        let state: ToolState = serde_json::from_str(json).unwrap();
        assert_eq!(state.status, "running");
        assert!(state.output.is_none());
        assert!(state.metadata.is_none());
        assert!(state.time.is_none());
    }
}
