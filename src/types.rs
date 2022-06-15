use serde::{Deserialize, Serialize};

use crate::{block::Block, blockchain::AccountState, transaction::Transaction};

pub type Address = [u8; 16];
pub type Hash = [u8; 32];
pub type TransactionSignature = [u8; 256];
pub type TransactionData = [u8; 64];
pub type PublicKey = [u8; 294];

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerNetworkMessage {
    AccountState(Address),
    SubmitTransaction(Transaction),
    GetChain,
    BroadcastBlock(Block),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientNetworkMessage {
    AccountState(AccountState),
    Ack,
    Error(String),
    Chain(Vec<Block>),
}
