use clap::Parser;
use zenchain::blockchain::BlockChain;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(short, long, value_parser)]
    port: u16,

    #[clap(short, long, value_parser)]
    key: Option<String>,
}

fn main() {
    run_node();
}

fn run_node() {
    let cli = Cli::parse();

    println!("Running zenchain node.");

    let chain = BlockChain::load();

    chain.run(cli.port, cli.key);
}
