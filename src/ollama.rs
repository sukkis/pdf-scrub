const ENDPOINT: &str = "http://localhost:11434/api/generate";
const PROMPT: &str = "Extract the text from this document page as Markdown. \
    Preserve all content exactly.";

pub async fn ask_ollama(image_bytes: &[u8], model: &str) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    let image_b64 = general_purpose::STANDARD.encode(image_bytes);
    let body = build_request(&image_b64, model);
    let client = reqwest::Client::new();
    let text = client
        .post(ENDPOINT)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| "ollama unavailable".to_string())?
        .text()
        .await
        .map_err(|_| "ollama unavailable".to_string())?;
    parse_response(&text)
}

fn build_request(image_b64: &str, model: &str) -> String {
    serde_json::json!({
        "model": model,
        "prompt": PROMPT,
        "images": [image_b64],
        "stream": false,
    })
    .to_string()
}

fn parse_response(json: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|_| "invalid response".to_string())?;
    if let Some(err) = v["error"].as_str() {
        return Err(format!("ollama error: {err}"));
    }
    v["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("invalid response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_response_extracts_response_field() {
        let json = r#"{"model":"mistral","response":"Page content here.","done":true}"#;
        assert_eq!(parse_response(json).unwrap(), "Page content here.");
    }

    #[test]
    fn parse_response_returns_error_on_missing_field() {
        let json = r#"{"model":"mistral","done":true}"#;
        assert!(parse_response(json).is_err());
    }

    #[test]
    fn parse_response_returns_error_on_invalid_json() {
        assert!(parse_response("not json").is_err());
    }

    #[test]
    fn build_request_contains_model() {
        let body = build_request("abc123", "mistral-small3.2:24b");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["model"], "mistral-small3.2:24b");
    }

    #[test]
    fn build_request_contains_image() {
        let body = build_request("abc123", "mistral-small3.2:24b");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["images"][0], "abc123");
    }

    #[test]
    fn build_request_is_not_streaming() {
        let body = build_request("abc123", "mistral-small3.2:24b");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["stream"], false);
    }

    #[test]
    fn build_request_contains_prompt() {
        let body = build_request("abc123", "mistral-small3.2:24b");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["prompt"], PROMPT);
    }

    #[cfg(feature = "local")]
    #[tokio::test]
    async fn ask_ollama_is_reachable() {
        // Minimal 1x1 white PNG — small enough that the model may reject it,
        // but sufficient to verify Ollama is running and responding.
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xd7, 0x63, 0xf8, 0xff, 0xff, 0x3f, 0x00, 0x05, 0xfe, 0x02, 0xfe, 0xdc, 0xcc, 0x59,
            0xe7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        match ask_ollama(png_bytes, "mistral-small3.2:24b").await {
            Ok(text) => assert!(!text.is_empty()),
            Err(e) => assert!(
                e.starts_with("ollama error:"),
                "Ollama is not reachable: {e}"
            ),
        }
    }
}
