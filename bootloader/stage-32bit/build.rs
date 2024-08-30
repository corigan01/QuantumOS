use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../linkerscripts/i686-quantum_loader.ld");
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path
            .join("../linkerscripts/i686-quantum_loader.ld")
            .display()
    )
}
