use std::env;

fn main() {
    const LIB_DIR: &str = "MONGOCRYPT_LIB_DIR";
    const STATIC: &str = "MONGOCRYPT_STATIC";
    for name in &[LIB_DIR, STATIC] {
        println!("cargo:rerun-if-env-changed={}", name);
    }

    let mut kind = "dylib";
    if let Ok(_) = env::var(STATIC) {
        kind = "static";
    }
    println!("cargo:rustc-link-lib={}=mongocrypt", kind);

    if let Ok(path) = env::var(LIB_DIR) {
        println!("cargo:rustc-link-search=native={}", path);
    }
}