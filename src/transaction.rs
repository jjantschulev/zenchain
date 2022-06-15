use std::fmt::Display;

use crate::{
    blockchain::World,
    client::BlockchainClient,
    keys,
    types::{Address, PublicKey, ServerNetworkMessage, TransactionData, TransactionSignature},
};

use openssl::{
    hash::MessageDigest,
    memcmp,
    pkey::PKey,
    rsa::Rsa,
    sign::{Signer, Verifier},
};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub amount: u128,
    pub index: u128,
    pub sender: Address,
    pub recipient: Address,

    #[serde(with = "BigArray")]
    pub signature: TransactionSignature,
    #[serde(with = "BigArray")]
    pub public_key: PublicKey,
}

impl Transaction {
    pub fn is_signature_valid(&self) -> bool {
        let data = Transaction::transaction_data_bytes(
            &self.sender,
            &self.recipient,
            self.amount,
            self.index,
        );
        let rsa = Rsa::public_key_from_der(&self.public_key).unwrap();
        let pkey = PKey::from_rsa(rsa.clone()).unwrap();

        let mut verifier = Verifier::new(MessageDigest::sha3_256(), &pkey).unwrap();
        verifier.update(&data).unwrap();
        let valid = verifier.verify(&self.signature).unwrap();

        let address = keys::keypair_to_address(&rsa);
        let is_sender = memcmp::eq(&address, &self.sender);

        return valid && is_sender;
    }

    pub fn is_valid(&self, account_states: &World) -> Result<(), String> {
        if !self.is_signature_valid() {
            return Err("Invalid signature".to_string());
        }
        let account_state = account_states.get_account_state(&self.sender);
        if account_state.balance < self.amount {
            return Err("Insufficient balance".to_string());
        }
        if account_state.transaction_index + 1 != self.index {
            return Err("Invalid transaction index".to_string());
        }
        return Ok(());
    }

    pub fn send(to: &str, amount: u128, client: &BlockchainClient) {
        let rsa = keys::load_keypair(None);
        let sender = keys::keypair_to_address(&rsa);
        let recipient = keys::parse_address(&to);

        let private_key = PKey::from_rsa(rsa).unwrap();

        let state = client.account_state(sender);
        let index = state.transaction_index + 1;

        let transaction_data =
            Transaction::transaction_data_bytes(&sender, &recipient, amount, index);

        let mut signer = Signer::new(MessageDigest::sha3_256(), &private_key).unwrap();
        signer.update(&transaction_data).unwrap();
        let signature = signer.sign_to_vec().unwrap();

        let mut signature_hash: TransactionSignature = [0u8; 256];
        signature_hash.copy_from_slice(&signature[..256]);

        let public_key_vec = private_key.public_key_to_der().unwrap();
        let mut public_key: PublicKey = [0u8; 294];
        public_key.copy_from_slice(&public_key_vec[..294]);

        let transaction = Transaction {
            amount,
            index,
            public_key,
            recipient,
            sender,
            signature: signature_hash,
        };

        println!(
            "Transaction SEND {} $ZEN to: {}",
            amount,
            keys::format_address(&recipient)
        );

        println!(
            "Transaction Signature Valid?: {:?}",
            transaction.is_signature_valid()
        );

        let result = client.send(ServerNetworkMessage::SubmitTransaction(transaction));
        println!("Transaction result: {:?}", result);
    }

    fn transaction_data_bytes(
        from: &Address,
        to: &Address,
        amount: u128,
        index: u128,
    ) -> TransactionData {
        let mut data = [0u8; 64];
        data[0..16].copy_from_slice(&from[..]);
        data[16..32].copy_from_slice(&to[..]);
        data[32..48].copy_from_slice(&amount.to_le_bytes());
        data[48..64].copy_from_slice(&index.to_le_bytes());
        data
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction: {} $ZEN  {} ==> {}",
            self.amount,
            keys::format_address(&self.sender),
            keys::format_address(&self.recipient)
        )
    }
}
