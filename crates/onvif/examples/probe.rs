use cctv_core::Credentials;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: probe DEVICE_ENDPOINT"))?;
    let creds = Credentials {
        username: std::env::var("CCTV_TEST_USER").unwrap_or_default(),
        password: std::env::var("CCTV_TEST_PASSWORD").unwrap_or_default(),
    };
    let client = cctv_onvif::Client::new(&endpoint, creds)?;
    let info = client.inspect().await?;
    anyhow::ensure!(!info.profiles.is_empty(), "No profiles");
    let profile = &info.profiles[0];
    let target = info
        .capabilities
        .ptz_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("PTZ missing"))?;
    client
        .continuous_move(target, &profile.token, 0.2, 0.1, 0.)
        .await?;
    client.stop(target, &profile.token).await?;
    let presets = client.presets(target, &profile.token).await?;
    anyhow::ensure!(!presets.is_empty(), "No presets");
    client
        .goto_preset(target, &profile.token, &presets[0].token)
        .await?;
    client
        .set_preset(target, &profile.token, "Local test")
        .await?;
    client.stop(target, &profile.token).await?;
    println!(
        "ONVIF information, capabilities, profiles, URI, PTZ and presets verified with fixture"
    );
    Ok(())
}
