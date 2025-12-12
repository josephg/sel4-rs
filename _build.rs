use std::process::Stdio;
use std::{path::PathBuf, process::Command};

fn main() {
    // 1) Build boot32
    let status = Command::new("cargo")
        .args(&[
            "build",
            "-p", "multiboot",
            "--target", "i686-none.json",
            "--release",
        ])
        .arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem")
    .env_remove("RUSTFLAGS")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to build multiboot stub");
    assert!(status.success(), "Build multiboot failed");

    // 2) Tell Rustc to link libboot32.a
    let outdir = PathBuf::from("target/i686-none/release");
    println!("cargo:rustc-link-search=native={}", outdir.display());
    println!("cargo:rustc-link-lib=static=boot32");
}
