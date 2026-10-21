use std::{
    io::Write,
    process::{Command, Stdio},
};
#[test]
fn cli_is_headless_and_protects_locked_library() {
    let binary = env!("CARGO_BIN_EXE_cctv-view");
    let dir = std::env::temp_dir().join(format!("cctv-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cctv.db");
    let mut store = cctv_storage::Store::open(&path).unwrap();
    let command = |flag: &str| {
        let mut c = Command::new(binary);
        c.env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .arg(flag)
            .arg("--data-dir")
            .arg(&dir);
        c
    };
    assert!(Command::new(binary)
        .arg("--help")
        .env_remove("DISPLAY")
        .output()
        .unwrap()
        .status
        .success());
    let open = command("--list-views").output().unwrap();
    assert!(open.status.success());
    assert_eq!(String::from_utf8(open.stdout).unwrap().trim(), "[]");
    store
        .set_app_password("", Some("cli-test-password"))
        .unwrap();
    let blocked = command("--list-views").output().unwrap();
    assert!(!blocked.status.success());
    assert!(blocked.stdout.is_empty());
    let mut unlocked = command("--list-views")
        .arg("--unlock-stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    unlocked
        .stdin
        .take()
        .unwrap()
        .write_all(b"cli-test-password\n")
        .unwrap();
    assert!(unlocked.wait_with_output().unwrap().status.success());
    let mut wrong = command("--list-cameras")
        .arg("--unlock-stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wrong.stdin.take().unwrap().write_all(b"wrong\n").unwrap();
    let out = wrong.wait_with_output().unwrap();
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}
