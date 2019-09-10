fn main() {
    prost_build::compile_protos(&["src/proto/db_items.proto"], &["src/"]).unwrap();
}
