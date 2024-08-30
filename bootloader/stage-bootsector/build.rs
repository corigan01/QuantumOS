use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed=init.s");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../linkerscripts/i386-quantum_bootsector.ld");
    println!(
        "cargo:rustc-link-arg-bins=--script={}",
        local_path
            .join("../linkerscripts/i386-quantum_bootsector.ld")
            .display()
    )
}
