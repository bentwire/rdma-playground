use std::sync::Arc;

use async_rdma::{device::{Device, DeviceList}, RdmaBuilder, ConnectionType};
use clap::Parser;

use tracing::level_filters::LevelFilter;

use cli::Args;
mod cli;
mod proto;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .pretty()
        .with_ansi(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_max_level(LevelFilter::DEBUG)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let dev_list = DeviceList::available()?;

    for dev in dev_list.as_slice() {
        tracing::info!("Dev: {:?}", dev);
    }

    let args = Args::parse();

    let server_rdma = RdmaBuilder::default()
                        .set_conn_type(ConnectionType::RCIBV)
                        .set_dev(&args.rdma_dev)
                        .set_port_num(args.rdma_dev_port)
                        .build()?;
    let server_rdma_qp = server_rdma.get_qp_endpoint();

    tracing::info!("Rdma: {:?}", server_rdma);
    tracing::info!("RdmaQP: {:?}", server_rdma_qp);

    let client_rdma = RdmaBuilder::default()
                        .set_conn_type(ConnectionType::RCIBV)
                        .build()?;
    let client_rdma_qp = client_rdma.get_qp_endpoint();

    tracing::info!("Rdma: {:?}", client_rdma);
    tracing::info!("RdmaQP: {:?}", client_rdma_qp);
    tracing::info!("Gid: {:} {:} {:?}", client_rdma_qp.qp_num(), client_rdma_qp.lid(), client_rdma_qp.gid());
    let rdma = Arc::new(server_rdma);



    Ok(())
}
