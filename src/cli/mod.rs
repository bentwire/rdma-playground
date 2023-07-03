use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value="http://127.0.0.1:55000", value_hint = clap::ValueHint::Url)]
    pub local_url: String,
    #[arg(long, default_value="http://127.0.0.1:55000", value_hint = clap::ValueHint::Url)]
    pub remote_url: String,
    #[arg(long, default_value="mlx4_0")]
    pub rdma_dev: String,
    #[arg(long, default_value="1")]
    pub rdma_dev_port: u8,
}