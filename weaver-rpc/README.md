To generate source files from proto [pingcap grpc](https://github.com/pingcap/grpc-rs)


    brew install protobuf
    cargo install protobuf
    cargo install grpcio-compiler
    protoc --rust_out=. --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_rust_plugin` example.proto
    

    