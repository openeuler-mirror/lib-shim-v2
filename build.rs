use ttrpc_codegen::Codegen;

fn main() {
    Codegen::new()
        .out_dir("src/protocols")
        .inputs(&[
            "src/protocols/protos/shim.proto",
            "src/protocols/protos/google/protobuf/any.proto",
            "src/protocols/protos/google/protobuf/empty.proto",
            "src/protocols/protos/gogoproto/gogo.proto",
            "src/protocols/protos/google/protobuf/timestamp.proto",
            "src/protocols/protos/github.com/containerd/containerd/api/types/mount.proto",
            "src/protocols/protos/github.com/containerd/containerd/api/types/task/task.proto",
        ])
        .include("src/protocols/protos")
        .rust_protobuf()
        .run()
        .expect("Codegen failed");
}
