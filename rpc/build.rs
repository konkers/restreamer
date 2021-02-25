fn main() {
    println!("cargo:rerun-if-changed=proto/obs.proto");
    tonic_build::configure()
        .compile(&["proto/obs.proto"], &["proto"])
        .unwrap();
}
