use std::env;

fn main() {
    const LIB_DIR: &str = "MONGOCRYPT_LIB_DIR";
    const STATIC: &str = "MONGOCRYPT_STATIC";
    const NAME: &str = "MONGOCRYPT_LIB_NAME";
    for var in &[LIB_DIR, STATIC, NAME] {
        println!("cargo:rerun-if-env-changed={}", var);
    }

    let name = if let Ok(s) = env::var(NAME) {
        s
    } else {
        "mongocrypt".to_string()
    };
    let kind = if let Ok(_) = env::var(STATIC) {
        "static"
    } else {
        "dylib"
    };

    println!("cargo:rustc-link-lib={}={}", kind, name);
    if let Ok(path) = env::var(LIB_DIR) {
        println!("cargo:rustc-link-search=native={}", path);
    }
}