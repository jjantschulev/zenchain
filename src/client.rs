use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    blockchain::AccountState,
    types::{Address, ClientNetworkMessage, ServerNetworkMessage},
};

pub struct BlockchainClient {
    address: String,
}

impl BlockchainClient {
    pub fn new(address: &str) -> BlockchainClient {
        BlockchainClient {
            address: address.to_owned(),
        }
    }

    fn connect(&self) -> TcpStream {
        let stream = TcpStream::connect(&self.address).unwrap();
        return stream;
    }

    pub fn account_state(&self, address: Address) -> AccountState {
        let mut stream = self.connect();
        BlockchainClient::write_message(&mut stream, ServerNetworkMessage::AccountState(address));
        match BlockchainClient::read_message(&mut stream) {
            ClientNetworkMessage::AccountState(state) => state,
            msg => panic!("Unexpected message: {:?}", msg),
        }
    }

    pub fn send(&self, message: ServerNetworkMessage) -> ClientNetworkMessage {
        let mut stream = self.connect();
        BlockchainClient::write_message(&mut stream, message);
        BlockchainClient::read_message(&mut stream)
    }

    fn read_message(stream: &mut TcpStream) -> ClientNetworkMessage {
        let mut len_buffer = [0u8; 4];
        stream.read_exact(&mut len_buffer).unwrap();
        let len = u32::from_le_bytes(len_buffer);
        let mut buffer: Vec<u8> = vec![0u8; len as usize];
        stream.read_exact(&mut buffer).unwrap();
        let message = bincode::deserialize::<ClientNetworkMessage>(&buffer).unwrap();
        return message;
    }

    fn write_message(stream: &mut TcpStream, message: ServerNetworkMessage) {
        let buffer = bincode::serialize(&message).unwrap();
        let len_bytes = (buffer.len() as u32).to_le_bytes();
        stream.write(&len_bytes).unwrap();
        stream.write(&buffer).unwrap();
        stream.flush().unwrap();
    }
}
