use clap::{Parser, Subcommand};

use zenchain::{client::BlockchainClient, keys, transaction::Transaction};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
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
    let client = BlockchainClient::new("localhost:8888");

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
            Transaction::send(to, *amount, &client);
        }
        Commands::Balance => {
            let address = keys::keypair_to_address(&keys::load_keypair(None));
            let balance = client.account_state(address);
            println!("Account Balance   : {:?} $ZEN", balance.balance);
            println!("Transaction Index : {:?}", balance.transaction_index);
        }
    }
}
