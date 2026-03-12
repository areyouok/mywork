use serde::Serialize;
use tauri::ipc::Channel;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PoCEvent {
    Message { text: String },
    Progress { percent: u32 },
    Done,
}

#[tauri::command]
pub async fn test_channel_stream(channel: Channel<PoCEvent>) -> Result<(), String> {
    channel
        .send(PoCEvent::Message {
            text: "Starting stream test...".to_string(),
        })
        .map_err(|e| e.to_string())?;

    for i in (0..=100).step_by(20) {
        channel
            .send(PoCEvent::Progress { percent: i })
            .map_err(|e| e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    channel.send(PoCEvent::Done).map_err(|e| e.to_string())?;

    Ok(())
}
