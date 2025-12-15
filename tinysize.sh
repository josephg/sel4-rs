cargo build -p kernel --release
strip target/x86_64-unknown-none/release/kernel
ls -l target/x86_64-unknown-none/release/kernel
