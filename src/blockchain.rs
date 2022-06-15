use std::{
    collections::{HashMap, HashSet},
    fs::{read, write},
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    client::BlockchainClient,
    keys,
    server::{load_nodes, BlockchainServer},
    transaction::Transaction,
    types::{Address, ClientNetworkMessage, Hash, ServerNetworkMessage},
};

enum MinerMessage {
    NewTransaction(Transaction, World),
    NewBlock(Block),
    // RemoveTransaction(Address, u128),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockChain {
    pub blocks: HashMap<Hash, Block>,

    #[serde(skip)]
    miner: Option<Sender<MinerMessage>>,
}

impl BlockChain {
    pub fn new() -> BlockChain {
        BlockChain {
            blocks: HashMap::new(),
            miner: None,
        }
    }

    pub fn save(&self) {
        let data = bincode::serialize(&self).unwrap();
        let path = Path::new("zenchain-data.bin");
        write(path, data).unwrap();
    }

    pub fn load() -> BlockChain {
        let path = Path::new("zenchain-data.bin");
        if !path.exists() {
            return BlockChain::new();
        }
        let data = read(path).unwrap();
        let chain: BlockChain = bincode::deserialize(&data).unwrap();
        chain
    }

    pub fn run(mut self, port: u16, key_name: Option<String>) {
        let (on_message_send, on_message_recv) = mpsc::channel::<ServerNetworkMessage>();
        let (return_send, return_recv) = mpsc::channel::<ClientNetworkMessage>();
        let longest_chain = BlockChain::get_longest_chain_from_network();

        BlockchainServer::run(port, on_message_send, return_recv);

        let (miner_send, miner_recv) = mpsc::channel::<MinerMessage>();

        for block in longest_chain {
            match block.is_valid(&self) {
                Ok(_) => {
                    self.blocks.insert(block.get_hash(), block);
                }
                _ => {}
            }
        }

        let chain = self.get_chain();
        let last = chain.into_iter().last();
        println!("Last block index: {}", last.as_ref().map_or(0, |b| b.index));

        let keypair = keys::load_keypair(key_name);
        let address = keys::keypair_to_address(&keypair);

        thread::spawn(move || {
            BlockChain::run_miner(miner_recv, last, address);
        });

        self.miner = Some(miner_send);

        for message in on_message_recv {
            let response = self.handle_message(message);
            return_send.send(response).unwrap();
        }
    }

    fn run_miner(channel: Receiver<MinerMessage>, parent: Option<Block>, miner: Address) {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut parent = parent;
        let mut block = Block::new(parent.as_ref(), &transactions, &miner);
        println!("Miner started");
        loop {
            match channel.try_recv() {
                Ok(message) => match message {
                    MinerMessage::NewTransaction(transaction, mut world) => {
                        println!("\nMiner got transaction: {}", transaction);
                        for t in &transactions {
                            world.update_on_transaction(&t);
                        }
                        if let Ok(()) = transaction.is_valid(&world) {
                            transactions.push(transaction);
                            block = Block::new(parent.as_ref(), &transactions, &miner);
                        }
                    }
                    MinerMessage::NewBlock(new_block) => {
                        println!(
                            "\nBlock {} mined by: {}",
                            new_block.index,
                            keys::format_address(&new_block.miner)
                        );
                        for transaction in &new_block.transactions {
                            if let Some(index) = transactions.iter().position(|t| {
                                t.sender == transaction.sender && t.index == transaction.index
                            }) {
                                transactions.remove(index);
                            }
                        }
                        parent = Some(new_block);
                        block = Block::new(parent.as_ref(), &transactions, &miner);
                    } // MinerMessage::RemoveTransaction(from, index) => {
                      //     println!(
                      //         "\nRemoving transaction {} from {} ",
                      //         index,
                      //         keys::format_address(&from)
                      //     );
                      // }
                },
                Err(err) => match err {
                    mpsc::TryRecvError::Empty => {}
                    mpsc::TryRecvError::Disconnected => {
                        println!("Miner stopped");
                        break;
                    }
                },
            }

            let successfull = block.mine(1000);
            if successfull {
                println!(
                    "\nBlock mined: {:?}. Transactions: {:?}",
                    block.index,
                    block.transactions.len()
                );
                let nodes = load_nodes();
                for node in nodes {
                    let client = BlockchainClient::new(&node);
                    let response = client.send(ServerNetworkMessage::BroadcastBlock(block.clone()));
                    println!("Broadcast block to {}. Response: {:?}", node, response);
                }
                transactions.clear();
            }
        }
    }

    pub fn handle_message(&mut self, message: ServerNetworkMessage) -> ClientNetworkMessage {
        match message {
            ServerNetworkMessage::AccountState(address) => ClientNetworkMessage::AccountState(
                World::from_chain(&self.get_chain())
                    .get_account_state(&address)
                    .clone(),
            ),
            ServerNetworkMessage::SubmitTransaction(transaction) => {
                match self.submit_transaction(transaction) {
                    Ok(_) => ClientNetworkMessage::Ack,
                    Err(err) => ClientNetworkMessage::Error(err),
                }
            }
            ServerNetworkMessage::GetChain => ClientNetworkMessage::Chain(self.get_chain()),
            ServerNetworkMessage::BroadcastBlock(block) => match block.is_valid(&self) {
                Ok(_) => {
                    let chain = self.get_chain();
                    self.blocks.insert(block.get_hash(), block.clone());
                    let chain_after = self.get_chain();

                    if chain_after.len() > chain.len() {
                        if let Some(ref miner) = self.miner {
                            miner.send(MinerMessage::NewBlock(block)).unwrap();
                        }
                    }

                    self.save();
                    ClientNetworkMessage::Ack
                }
                Err(err) => ClientNetworkMessage::Error(err),
            },
        }
    }

    fn submit_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        let world = World::from_chain(&self.get_chain());
        transaction.is_valid(&world)?;
        match self.miner {
            Some(ref channel) => channel
                .send(MinerMessage::NewTransaction(transaction, world))
                .unwrap(),
            None => return Err("Transaction channel not initialized".to_string()),
        }
        Ok(())
    }

    pub fn get_chain(&self) -> Vec<Block> {
        let mut leaves: HashSet<Hash> = self.blocks.clone().into_keys().collect();

        for (_, block) in &self.blocks {
            leaves.remove(&block.prev_hash);
        }

        let longest_chain = leaves
            .into_iter()
            .map(|l| self.get_chain_from_leaf(l))
            .fold(Vec::new(), |longest, chain| {
                if chain.len() > longest.len() {
                    chain
                } else {
                    longest
                }
            });

        longest_chain
    }

    pub fn get_chain_from_leaf(&self, leaf: Hash) -> Vec<Block> {
        let mut chain = Vec::new();
        let mut current_hash = leaf;
        if current_hash == [0u8; 32] {
            return chain;
        }
        loop {
            let block = self.blocks.get(&current_hash).unwrap();
            chain.push(block.clone());
            current_hash = block.prev_hash;
            if current_hash == [0u8; 32] {
                break;
            }
        }
        chain.reverse();
        chain
    }

    pub fn get_longest_chain_from_network() -> Vec<Block> {
        let nodes = load_nodes();
        let mut longest_chain = Vec::new();
        for node in nodes {
            let client = BlockchainClient::new(&node);
            let chain = client.send(ServerNetworkMessage::GetChain);
            match chain {
                Ok(msg) => match msg {
                    ClientNetworkMessage::Chain(chain) => {
                        println!("Got chain with len={} from {}: ", chain.len(), node);
                        if chain.len() > longest_chain.len() {
                            longest_chain = chain;
                        }
                    }
                    _ => {}
                },
                Err(msg) => println!("Node {} error: {:?}", node, msg),
            }
        }
        longest_chain
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountState {
    pub address: Address,
    pub balance: u128,
    pub transaction_index: u128,
}

impl AccountState {
    pub fn new(address: Address) -> AccountState {
        AccountState {
            address,
            balance: 0,
            transaction_index: 0,
        }
    }

    pub fn update_on_block(&mut self, block: &Block) {
        if block.miner == self.address {
            self.balance += block.reward;
        }
    }

    pub fn update_on_transaction(&mut self, transaction: &Transaction) {
        if transaction.sender == self.address {
            self.balance -= transaction.amount;
            self.transaction_index += 1;
        }
        if transaction.recipient == self.address {
            self.balance += transaction.amount;
        }
    }
}

/*

when we receive a transaction, check if valid and add to current block

When we recieve a block, add it to our hashmap and add it as a leaf
check to remove leafs that this block is a child of



*/

#[derive(Debug, Clone)]
pub struct World {
    accounts: HashMap<Address, AccountState>,
}

impl World {
    pub fn new() -> World {
        World {
            accounts: HashMap::new(),
        }
    }

    pub fn get_account_state(&self, address: &Address) -> AccountState {
        match self.accounts.get(address) {
            Some(account) => account.clone(),
            None => AccountState::new(address.clone()),
        }
    }

    fn get_account_state_mut(&mut self, address: &Address) -> &mut AccountState {
        if self.accounts.get(address).is_none() {
            let account = AccountState::new(address.clone());
            self.accounts.insert(address.clone(), account);
        }
        self.accounts.get_mut(address).unwrap()
    }

    pub fn update_on_transaction(&mut self, transaction: &Transaction) {
        let sender = self.get_account_state_mut(&transaction.sender);
        sender.update_on_transaction(transaction);
        let recipient = self.get_account_state_mut(&transaction.recipient);
        recipient.update_on_transaction(transaction);
    }
    pub fn update_on_block(&mut self, block: &Block) {
        let miner = self.get_account_state_mut(&block.miner);
        miner.update_on_block(block);
    }

    pub fn from_chain(chain: &Vec<Block>) -> World {
        let mut world = World::new();
        for block in chain {
            world.update_on_block(block);
            for transaction in &block.transactions {
                world.update_on_transaction(&transaction);
            }
        }
        world
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}
