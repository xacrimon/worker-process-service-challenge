mod cli;
mod client;

use cli::Opts;
use structopt::StructOpt;
use tonic::transport::{Certificate, Identity};

const CLIENT_CERT: &[u8] = include_bytes!("../../data/client.pem");
const CLIENT_KEY: &[u8] = include_bytes!("../../data/client.key");
const SERVER_CA_CERT: &[u8] = include_bytes!("../../data/server_ca.pem");

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();
}
