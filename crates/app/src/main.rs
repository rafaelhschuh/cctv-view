fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cctv=info".into()),
        )
        .with_target(false)
        .init();
    cctv_ui::run();
}
