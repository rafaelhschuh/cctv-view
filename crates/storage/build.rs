fn main() {
    println!("cargo:rerun-if-changed=src/secret.c");
    let lib = pkg_config::Config::new()
        .probe("libsecret-1")
        .expect("Install libsecret-1-dev (see docs/BUILD.md)");
    let mut build = cc::Build::new();
    build.file("src/secret.c");
    for p in lib.include_paths {
        build.include(p);
    }
    build.compile("cctv_secret");
}
