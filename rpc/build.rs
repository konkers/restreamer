fn main() {
    tonic_build::configure()
        .compile(&["proto/obs.proto"], &["proto"])
        .unwrap();
}
