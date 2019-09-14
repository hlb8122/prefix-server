fn main() {
    tower_grpc_build::Config::new()
        .enable_server(true)
        .build(&["proto/prefix_server.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
    println!("cargo:rerun-if-changed=proto/prefix_server.proto");
}
