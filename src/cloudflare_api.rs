use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize)]
struct UpdateDnsRequest {
    r#type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<T>,
}

#[derive(Deserialize, Debug)]
struct CloudflareError {
    code: u32,
    message: String,
}

pub async fn update_dns_record(
    api_token: &str,
    zone_id: &str,
    record_id: &str,
    domain: &str,
    ip: &str,
) -> Result<()> {
    let client = Client::new();
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );

    let request_body = UpdateDnsRequest {
        r#type: "A".to_string(),
        name: domain.to_string(),
        content: ip.to_string(),
        ttl: 1, // Auto TTL
        proxied: true,
    };

    let response: CloudflareResponse<serde_json::Value> = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?
        .json()
        .await?;

    if response.success {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to update DNS record: {:?}",
            response.errors
        ))
    }
}