use clap::{Parser, Subcommand};

use zenchain::{client::BlockchainClient, keys, server, transaction::Transaction};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, value_parser)]
    node: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Keys {
        #[clap(subcommand)]
        keys: KeyCommands,
    },
    GetAddress,
    Balance,
    Send {
        #[clap(short, long, value_parser)]
        to: String,
        #[clap(short, long, value_parser)]
        amount: u128,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    Generate {
        #[clap(value_parser)]
        name: String,
    },
    Delete {
        #[clap(value_parser)]
        name: String,
    },
    SetDefault {
        #[clap(value_parser)]
        name: String,
    },
    List,
}

fn main() {
    let cli = Cli::parse();
    let client = BlockchainClient::new(&cli.node.unwrap_or("localhost:8888".to_string()));

    let all_clients = server::load_nodes()
        .iter()
        .map(|node| BlockchainClient::new(node))
        .collect::<Vec<_>>();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Keys { keys } => match keys {
            KeyCommands::Generate { name } => keys::generate_keypair(name.to_string()),
            KeyCommands::List => keys::list_keypairs(),
            KeyCommands::Delete { name } => keys::delete_key(name.to_string()),
            KeyCommands::SetDefault { name } => keys::set_default_keypair(name.to_string()),
        },
        Commands::GetAddress => {
            println!(
                "Your address is: \n{}",
                keys::format_address(&keys::keypair_to_address(&keys::load_keypair(None)))
            );
        }
        Commands::Send { to, amount } => {
            for client in &all_clients {
                if let Err(msg) = Transaction::send(to, *amount, &client) {
                    println!("Node {} Error: {}", client.address, msg);
                } else {
                    println!("Transaction sent to node: {}", client.address);
                }
            }
            println!("Sent {} $ZEN to: {}", amount, to);
        }
        Commands::Balance => {
            let address = keys::keypair_to_address(&keys::load_keypair(None));
            match client.account_state(address) {
                Ok(balance) => {
                    println!("Account Balance   : {:?} $ZEN", balance.balance);
                    println!("Transaction Index : {:?}", balance.transaction_index);
                }
                Err(err) => println!("Error: {:?}", err),
            }
        }
    }
}
