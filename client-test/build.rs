fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["../proto-test/helloworld.proto", "../proto-test/echo.proto"],
            &["../proto-test"],
        )
        .unwrap();
}
