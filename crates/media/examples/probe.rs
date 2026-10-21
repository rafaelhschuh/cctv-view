//! Native pipeline probe for the reproducible local simulator; never prints the supplied URI.
use cctv_core::{Camera, Credentials};
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("Usage: probe URL [--expect-failure]"))?;
    let flags: Vec<_> = args.collect();
    let failure = flags.iter().any(|a| a == "--expect-failure");
    let camera = Camera {
        name: "Protocol fixture".into(),
        stream_url: url,
        timeout_ms: 1000,
        hardware_decode: false,
        tls_verify: !flags.iter().any(|a| a == "--no-verify-tls"),
        no_credentials: flags.iter().any(|a| a == "--no-credentials"),
        ..Default::default()
    };
    let credentials = Credentials {
        username: std::env::var("CCTV_TEST_USER").unwrap_or_default(),
        password: std::env::var("CCTV_TEST_PASSWORD").unwrap_or_default(),
    };
    let result = cctv_media::connection_test(&camera, credentials);
    if failure {
        anyhow::ensure!(result.is_err(), "Expected a connection failure");
        println!("Expected failure verified");
    } else {
        result?;
        println!("Native video frame received");
    }
    Ok(())
}
