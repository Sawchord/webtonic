fn main() {
    tonic_build::compile_protos("../old/proto/helloworld.proto").unwrap();
}
