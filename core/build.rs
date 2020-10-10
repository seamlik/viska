fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../grpc/changelog.proto")?;
    tonic_build::compile_protos("../grpc/database.proto")?;
    tonic_build::compile_protos("../grpc/transaction.proto")?;
    tonic_build::compile_protos("../grpc/daemon.proto")?;
    tonic_build::compile_protos("../grpc/proto.proto")?;
    Ok(())
}
