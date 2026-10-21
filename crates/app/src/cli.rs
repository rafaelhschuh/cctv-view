use anyhow::{bail, ensure, Context, Result};
use std::{
    ffi::OsString,
    io::{IsTerminal, Read},
    path::PathBuf,
};

#[derive(Default)]
pub struct Options {
    pub view: Option<String>,
    pub fullscreen: bool,
    pub data_dir: Option<PathBuf>,
    pub list_views: bool,
    pub list_cameras: bool,
    pub unlock_stdin: bool,
    pub help: bool,
    pub version: bool,
}
impl Options {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self> {
        let mut out = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--view") => {
                    out.view = Some(
                        args.next()
                            .context("Falta o nome ou ID da View.")?
                            .into_string()
                            .map_err(|_| anyhow::anyhow!("Nome de View inválido."))?,
                    )
                }
                Some("--data-dir") => {
                    out.data_dir = Some(PathBuf::from(
                        args.next().context("Falta o diretório de dados.")?,
                    ))
                }
                Some("--fullscreen") => out.fullscreen = true,
                Some("--list-views") => out.list_views = true,
                Some("--list-cameras") => out.list_cameras = true,
                Some("--unlock-stdin") => out.unlock_stdin = true,
                Some("--help" | "-h") => out.help = true,
                Some("--version" | "-V") => out.version = true,
                _ => bail!("Opção desconhecida. Use --help."),
            }
        }
        ensure!(
            !(out.list_views && out.list_cameras),
            "Escolha uma listagem por execução."
        );
        ensure!(
            !(out.list_views || out.list_cameras) || (out.view.is_none() && !out.fullscreen),
            "Não combine listagem com abertura de View."
        );
        Ok(out)
    }
    pub fn data_path(&self) -> Result<PathBuf> {
        if let Some(dir) = self
            .data_dir
            .clone()
            .or_else(|| std::env::var_os("CCTV_VIEW_DATA_DIR").map(PathBuf::from))
        {
            std::fs::create_dir_all(&dir)?;
            Ok(dir.join("cctv.db"))
        } else {
            cctv_storage::Store::default_path()
        }
    }
}
pub fn stdin_password() -> Result<String> {
    ensure!(
        !std::io::stdin().is_terminal(),
        "Envie a senha por pipe com --unlock-stdin ou use a tela de desbloqueio."
    );
    let mut bytes = vec![];
    std::io::stdin().take(1027).read_to_end(&mut bytes)?;
    let text = String::from_utf8(bytes).context("Entrada de senha inválida.")?;
    let value = text.trim_end_matches(['\r', '\n']).to_string();
    ensure!(value.len() <= 1024, "Senha excede o limite.");
    Ok(value)
}
pub fn list(options: &Options, password: Option<&str>) -> Result<()> {
    let store = cctv_storage::Store::open(options.data_path()?)?;
    if store.has_app_password()? {
        ensure!(
            password
                .map(|p| store.verify_app_password(p))
                .transpose()?
                .unwrap_or(false),
            "Biblioteca bloqueada. Desbloqueie pela interface ou use --unlock-stdin."
        );
    }
    let rows: Vec<_> = if options.list_views {
        store.views()?.into_iter().map(|v|serde_json::json!({"id":v.id,"name":v.name,"columns":v.columns,"rows":v.rows,"panels":v.panels.len()})).collect()
    } else {
        store
            .cameras()?
            .into_iter()
            .map(|c| serde_json::json!({"id":c.id,"name":c.name}))
            .collect()
    };
    println!("{}", serde_json::to_string_pretty(&rows)?);
    Ok(())
}
pub const HELP:&str="CCTV View — monitor nativo de câmeras\n\nUso:\n  cctv-view [--view NOME_OU_ID] [--fullscreen] [--data-dir DIRETÓRIO]\n  cctv-view --list-views | --list-cameras [--data-dir DIRETÓRIO]\n  cctv-view --unlock-stdin [outras opções]\n  cctv-view --help | --version\n\n--view abre uma View existente pelo nome exato ou ID.\n--fullscreen inicia somente com os streams organizados.\nAs listagens JSON funcionam sem display e omitem URLs/credenciais.\n--unlock-stdin lê a senha do aplicativo por pipe, nunca por argumento.\nA senha é definida em Preferências → Senha do aplicativo.\n";
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Options> {
        Options::parse(args.iter().map(OsString::from))
    }
    #[test]
    fn parses_launch_and_rejects_ambiguous_commands() {
        let o = parse(&[
            "--view",
            "Portaria",
            "--fullscreen",
            "--data-dir",
            "/tmp/cctv-cli",
        ])
        .unwrap();
        assert_eq!(o.view.as_deref(), Some("Portaria"));
        assert!(o.fullscreen);
        assert!(parse(&["--view"]).is_err());
        assert!(parse(&["--unknown"]).is_err());
        assert!(parse(&["--list-views", "--fullscreen"]).is_err());
        assert!(parse(&["--list-views", "--list-cameras"]).is_err());
    }
}
