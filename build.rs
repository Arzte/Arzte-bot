extern crate built;
fn main() {
    let mut options_default = built::Options::default();
    let options = options_default
        .set_compiler(false)
        .set_ci(false)
        .set_features(false)
        .set_cfg(false);
    let src = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&options, src.as_ref(), &dst)
        .expect("Failed to acquire build-time information");
}
