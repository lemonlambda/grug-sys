use std::{env, path::PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    cc::Build::new()
        .file("grug/grug.c")
        .include("grug")
        .compile("grug");

    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=grug");

    let bindings = bindgen::Builder::default()
        .header("grug/grug.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("bindgen failed");

    let out = PathBuf::from(out_dir);
    bindings.write_to_file(out.join("bindings.rs")).unwrap();

    println!("cargo:rustc-link-arg=-rdynamic");
}
