use clap::Parser;


/// Command line arguments and config file fields.
#[derive(Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value="http://127.0.0.1:55000", value_hint = clap::ValueHint::Url)]
    pub server_url: String,
    #[arg(long, default_value="http://127.0.0.1:55000", value_hint = clap::ValueHint::Url)]
    pub manager_url: String,
    /// RDMA Device name
    #[arg(long, default_value="mlx4_0")]
    pub rdma_dev: String,
    /// RDMA Device physical port
    #[arg(long, default_value="1")]
    pub rdma_dev_port: u8,
}