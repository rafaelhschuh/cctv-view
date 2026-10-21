fn main() {
    println!("cargo:rerun-if-changed=src/decoder.c");
    let mut build = cc::Build::new();
    build.file("src/decoder.c").flag_if_supported("-std=c11");
    for name in [
        "libavformat",
        "libavcodec",
        "libavutil",
        "libswscale",
        "libswresample",
        "libpulse-simple",
    ] {
        let lib = pkg_config::Config::new()
            .probe(name)
            .expect("Install libav*-dev, libsw*-dev, libpulse-dev; see docs/BUILD.md");
        for p in lib.include_paths {
            build.include(p);
        }
    }
    build.compile("cctv_decoder");
}
