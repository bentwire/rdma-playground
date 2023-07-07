//#![feature(vec_into_raw_parts)]

use async_rdma::{device::{Device, DeviceList}, RdmaBuilder, ConnectionType, QueuePairEndpointBuilder};

use clap::Parser;
use figment::{Figment, providers::{Serialized, Toml, Env, Format}};

use serde;
use tracing::level_filters::LevelFilter;

use cli::Args;
mod cli;
mod server;
mod proto;
mod rdma_types;


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

    let args: Args = Figment::new()
        .merge(Serialized::defaults(Args::parse()))
        .merge(Toml::file("/etc/rdma/rdma.toml"))
        .merge(Toml::file("rdma.toml"))
        .merge(Env::prefixed("RDMA_"))
        .extract()?;

    //let conf_str = toml::to_string_pretty(&args).unwrap();

    //tracing::info!("CONF TOML:\n{:}", conf_str);
    //let args = Args::parse();

    let server_rdma = RdmaBuilder::default()
                        //.listen("192.168.10.40:55000").await?;
                        .set_conn_type(ConnectionType::RCIBV)
                        .set_dev(&args.rdma_dev)
                        .set_port_num(args.rdma_dev_port)
                        //.set_gid_index(2)
                        .build()?;
    let server_rdma_qp = server_rdma.get_qp_endpoint();

    let client_rdma = RdmaBuilder::default()
                        .set_conn_type(ConnectionType::RCIBV)
                        .build()?;
    let client_rdma_qp = client_rdma.get_qp_endpoint();
    
    //let foo = serde_json::to_string(&client_rdma_qp);

    tracing::info!("Server Rdma: {:?}", server_rdma);
    tracing::info!("Server RdmaQP: {:?}", server_rdma_qp);

    tracing::info!("Client Rdma: {:?}", client_rdma);
    tracing::info!("Client RdmaQP: {:?}", client_rdma_qp);
    //tracing::info!("Gid: {:} {:} {:?}", client_rdma_qp.qp_num(), client_rdma_qp.lid(), client_rdma_qp.gid());
    //let rdma = Arc::new(server_rdma);

    // let serv = server_rdma.listen().await?;
    // let conn = client_rdma.new_connect("192.168.10.40:55000").await?;

    // tracing::info!("CONN? {:?} {:?}", serv, conn);


    Ok(())
}
