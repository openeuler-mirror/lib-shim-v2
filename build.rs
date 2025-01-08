use ttrpc_codegen::Codegen;
use ttrpc_codegen::ProtobufCustomize;

fn main() {
    let protobuf_customized = ProtobufCustomize::default().gen_mod_rs(false);

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
        .rust_protobuf_customize(protobuf_customized.clone())
        .run()
        .expect("Codegen failed");
}
