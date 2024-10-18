fn main() {
    cc::Build::new()
        .file("fast_transport/EMD_wrapper.cpp")
        .compile("fast_transport");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("EMD_wrap")
        .clang_arg("-xc++") // https://github.com/rust-lang/rust-bindgen/issues/1855
        .generate()
        .expect("Couldn't generate bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
