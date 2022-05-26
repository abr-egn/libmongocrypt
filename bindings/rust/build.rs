use std::env;

fn main() {
    const LIB_DIR: &str = "MONGOCRYPT_LIB_DIR";
    const STATIC: &str = "MONGOCRYPT_STATIC";
    for var in &[LIB_DIR, STATIC] {
        println!("cargo:rerun-if-env-changed={}", var);
    }

    let is_static = env::var(STATIC).is_ok();
    let (name, kind) = if is_static {
        ("mongocrypt-static", "static")
    } else {
        ("mongocrypt", "dylib")
    };

    println!("cargo:rustc-link-lib={}={}", kind, name);
    if is_static {
        println!("cargo:rustc-link-lib=static=bson-static");
        println!("cargo:rustc-link-lib=static=kms_message-static");
    }
    if let Ok(path) = env::var(LIB_DIR) {
        println!("cargo:rustc-link-search=native={}", path);
    }
}