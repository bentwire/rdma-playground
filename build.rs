fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
         .build_server(true)
         .build_client(true)
         .build_transport(true)
         .out_dir("src/proto")
         .include_file("mod.rs")
         .compile(
             &["proto/rdma.proto"],
             &["proto"],
         )?;
    Ok(())
 }