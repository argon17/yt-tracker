pub fn send_update(
    client: &reqwest::blocking::Client,
    webhook_url: &str,
    message: &str,
) -> anyhow::Result<()> {
    let body = serde_json::json!({ "content": message });
    client.post(webhook_url).json(&body).send()?;
    Ok(())
}
