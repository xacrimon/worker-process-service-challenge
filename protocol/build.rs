fn main() {
    tonic_build::compile_protos("./api.proto").unwrap();
}
