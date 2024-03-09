fn main() {
  prost_build::compile_protos(&["session.proto"], &["."]).unwrap();
}
