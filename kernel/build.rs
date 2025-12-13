  use std::path::PathBuf;

  fn main() {
      let script = PathBuf::from("linker.lds"); // adjust if in another dir
      println!("cargo:rerun-if-changed={}", script.display());
      // println!("cargo:rustc-link-arg=-T{}", script.display());
      // println!("cargo:rustc-link-arg=-no-pie"); // if you need this too
  }
