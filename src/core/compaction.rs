use crate::state::session::Message;
use serde_json::json;

pub fn apply_snip_shaper(
    history: &[Message],
    max_history: usize,
    preserve_recent: usize,
) -> Vec<Message> {
    if history.len() <= max_history {
        return history.to_vec();
    }

    println!("✂️  [Compaction Layer 2]: Context length exceeds budget. Sniping middle frames...");

    let anchor_count = 2;
    let mut compacted = Vec::new();

    // Preserve original system anchors
    if history.len() >= anchor_count {
        compacted.extend_from_slice(&history[0..anchor_count]);
    }

    // Insert localized system warning frame
    compacted.push(Message {
        role: "system".to_string(),
        content: format!(
            "[System Context Compaction Notice: {} historical turns omitted from execution view]",
            history.len() - (anchor_count + preserve_recent)
        ),
    });

    // Isolate immediate fresh interactive contexts
    if history.len() >= preserve_recent {
        let start = history.len() - preserve_recent;
        compacted.extend_from_slice(&history[start..]);
    }

    compacted
}

pub fn _apply_snip_shaper_compaction(
    history: &mut Vec<Message>,
    max_len: usize,
    preserve_recent: usize,
) {
    if history.len() <= max_len {
        return;
    }

    // Isolate context parameters securely
    let system_anchors: Vec<Message> = history.drain(0..2).collect();
    let recent_idx = history.len() - preserve_recent;
    let recent_turns: Vec<Message> = history.drain(recent_idx..).collect();

    let placeholder = Message {
        role: "system".to_string(),
        content: format!(
            "[System Context Compaction Notice: {} conversational execution history logs sniped cleanly here to reclaim context margins.]",
            history.len()
        ),
    };

    let mut new_history = Vec::new();
    new_history.extend(system_anchors);
    new_history.push(placeholder);
    new_history.extend(recent_turns);
    *history = new_history;
}

pub async fn apply_generative_auto_compact(
    history: &[Message],
    threshold: usize,
    model_uri: &str,
    model_name: &str,
) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
    if history.len() < threshold {
        return Ok(history.to_vec());
    }

    println!("✨ [Compaction Layer 5]: Triggering model summary synthesis...");
    let client = reqwest::Client::new();

    let distillation_prompt = format!(
        "You are an internal system context compression engine. \n\
        Review the following transcript of an engineering session. Output a highly dense, bulleted summary explaining:\n\
        1. What architectural discoveries or file reads were made.\n\
        2. What code was successfully modified or created.\n\
        3. What problems remain unsolved.\n\
        \n\
        Be incredibly concise. Do not include introductory boilerplate.\n\
        \n\
        Transcript to compress:\n{:?}",
        history
    );

    let payload = json!({
        "model": model_name,
        "messages": [
            { "role": "user", "content": distillation_prompt }
        ],
        "stream": false,
        "options": { "temperature": 0.7 }
    });

    let res = client.post(model_uri).json(&payload).send().await?;

    if !res.status().is_success() {
        return Err("Compaction call to backend model failed".into());
    }

    let data: serde_json::Value = res.json().await?;
    let summary_text = data["message"]["content"].as_str().unwrap_or("").trim();

    let system_anchor = history[0..1].to_vec();
    let compact_boundary = Message {
        role: "system".to_string(),
        content: format!(
            "### 📂 SystemCompactBoundaryMessage\nBelow is a compressed summary of prior actions:\n{}",
            summary_text
        ),
    };

    let mut new_history = system_anchor;
    new_history.push(compact_boundary);
    Ok(new_history)
}

#[tokio::test]
async fn test_ollama_connectivity() {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();

    let res = client.get("http://127.0.0.1:11434/").send().await;
    match res {
        Ok(response) => println!("✅ Connected successfully! Status: {}", response.status()),
        Err(e) => println!("❌ Still failing: {:?}", e),
    }
}
