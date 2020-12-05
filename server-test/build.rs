fn main() {
    tonic_build::configure()
        .build_client(false)
        .compile(&["../proto-test/helloworld.proto"], &["../proto-test"])
        .unwrap();
}
