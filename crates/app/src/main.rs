mod cli;
fn main() {
    if let Err(error) = execute() {
        eprintln!("CCTV View: {error}");
        std::process::exit(1);
    }
}
fn execute() -> anyhow::Result<()> {
    let options = cli::Options::parse(std::env::args_os().skip(1))?;
    if options.help {
        print!("{}", cli::HELP);
        return Ok(());
    }
    if options.version {
        println!("CCTV View {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let password = if options.unlock_stdin {
        Some(cli::stdin_password()?)
    } else {
        None
    };
    if options.list_views || options.list_cameras {
        return cli::list(&options, password.as_deref());
    }
    if let Some(password) = password.as_deref() {
        anyhow::ensure!(
            cctv_storage::Store::open(options.data_path()?)?.verify_app_password(password)?,
            "Senha do aplicativo incorreta."
        );
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cctv=info".into()),
        )
        .with_target(false)
        .init();
    cctv_ui::run_with_options(cctv_ui::LaunchOptions {
        view: options.view,
        fullscreen: options.fullscreen,
        data_dir: options.data_dir,
        unlock_password: password,
    });
    Ok(())
}
