fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        println!("cargo:rustc-link-search=packages/windows-release");
        println!("cargo:rustc-link-lib=tirra-win");
    } else {
        println!("cargo:rerun-if-changed=build.rs");
    }
}
