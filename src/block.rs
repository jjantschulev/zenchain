use openssl::hash::{Hasher, MessageDigest};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    blockchain::{BlockChain, World},
    transaction::Transaction,
    types::{Address, Hash},
};

pub const DIFFICULTY: u32 = 2;
pub const BLOCK_REWARD: u128 = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub index: u128,
    pub prev_hash: Hash,
    pub nonce: [u8; 32],
    pub miner: Address,
    pub reward: u128,
    pub transactions: Vec<Transaction>,
    pub difficulty: u32,
}

impl Block {
    pub fn new(parent: Option<&Block>, transactions: &Vec<Transaction>, miner: &Address) -> Block {
        Block {
            index: parent.as_ref().map_or(1, |b| b.index + 1),
            prev_hash: parent.as_ref().map_or([0u8; 32], |b| b.get_hash()),
            transactions: transactions.clone(),
            difficulty: DIFFICULTY,
            reward: BLOCK_REWARD,
            miner: miner.clone(),
            nonce: [0u8; 32],
        }
    }

    pub fn get_hash(&self) -> Hash {
        let mut hasher = Hasher::new(MessageDigest::sha3_256()).unwrap();
        let data = bincode::serialize(self).unwrap();
        hasher.update(&data).unwrap();
        let hash = hasher.finish().unwrap();

        let mut hash_bytes: Hash = [0u8; 32];
        hash_bytes.copy_from_slice(&hash);
        hash_bytes
    }

    pub fn mine(&mut self, attempts: u128) -> bool {
        for _ in 0..attempts {
            self.randomize_nonce();
            let hash = self.get_hash();
            let difficulty_buffer = vec![0u8; self.difficulty as usize];
            if hash.starts_with(&difficulty_buffer) {
                return true;
            }
        }
        false
    }

    fn randomize_nonce(&mut self) {
        self.nonce = rand::thread_rng().gen::<[u8; 32]>();
    }

    pub fn is_valid(&self, blockchain: &BlockChain) -> Result<(), String> {
        if self.difficulty != DIFFICULTY {
            return Err("Invalid difficulty".to_string());
        }
        if self.reward != BLOCK_REWARD {
            return Err("Invalid reward".to_string());
        }
        let parent = if self.prev_hash == [0u8; 32] {
            match blockchain.blocks.get(&self.prev_hash) {
                Some(block) => Some(block),
                None => None,
            }
        } else {
            match blockchain.blocks.get(&self.prev_hash) {
                Some(block) => Some(block),
                None => return Err("Invalid prev_hash. Parent not found".to_string()),
            }
        };

        if self.index != parent.map_or(0, |p| p.index) + 1 {
            return Err(format!(
                "Invalid index. Should be {}, but is {}",
                parent.map_or(0, |p| p.index) + 1,
                self.index
            ));
        }

        let hash = self.get_hash();
        let difficulty_buffer = vec![0u8; self.difficulty as usize];
        if !hash.starts_with(&difficulty_buffer) {
            return Err("Invalid hash. Did you really do the work?".to_string());
        }

        let chain = blockchain.get_chain_from_leaf(self.prev_hash);
        let mut world = World::from_chain(&chain);

        for transaction in &self.transactions {
            if let Err(message) = transaction.is_valid(&world) {
                return Err(format!(
                    "Invalid transaction: {}. Error: {}",
                    transaction, message
                ));
            }
            world.update_on_transaction(&transaction);
        }
        world.update_on_block(&self);

        return Ok(());
    }
}
