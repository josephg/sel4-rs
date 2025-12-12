fn main() {
    println!("cargo:rerun-if-changed=linker32.lds");
    println!("cargo:rustc-link-arg=-Tlinker32.lds");
    println!("cargo:rustc-link-arg=-no-pie");
    println!("cargo:rustc-link-arg=-static"); // optional, see below
}
