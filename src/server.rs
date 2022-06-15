use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    thread,
};

use crate::types::{ClientNetworkMessage, ServerNetworkMessage};

pub struct BlockchainServer {}

impl BlockchainServer {
    pub fn run(
        port: u16,
        on_message: Sender<ServerNetworkMessage>,
        on_return: Receiver<ClientNetworkMessage>,
    ) {
        thread::spawn(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
            println!("Running Blockchain Server on port {}", port);

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let message = BlockchainServer::read_message(&mut stream);
                        on_message.send(message).unwrap();
                        let return_message = on_return.recv().unwrap();
                        BlockchainServer::write_message(&mut stream, return_message);
                    }
                    Err(e) => {
                        println!("Server Error: {}", e);
                    }
                }
            }

            drop(listener);
        });
    }

    fn read_message(stream: &mut TcpStream) -> ServerNetworkMessage {
        let mut len_buffer = [0u8; 4];
        stream.read_exact(&mut len_buffer).unwrap();
        let len = u32::from_le_bytes(len_buffer);
        let mut buffer: Vec<u8> = vec![0u8; len as usize];
        stream.read_exact(&mut buffer).unwrap();
        let message = bincode::deserialize::<ServerNetworkMessage>(&buffer).unwrap();
        return message;
    }

    fn write_message(stream: &mut TcpStream, message: ClientNetworkMessage) {
        let buffer = bincode::serialize(&message).unwrap();
        let len_bytes = (buffer.len() as u32).to_le_bytes();
        stream.write(&len_bytes).unwrap();
        stream.write(&buffer).unwrap();
        stream.flush().unwrap();
    }
}

pub fn load_nodes() -> Vec<String> {
    let mut nodes = Vec::new();
    let nodes_file = fs::read_to_string("nodes.txt").unwrap();
    for line in nodes_file.lines() {
        nodes.push(line.to_string());
    }
    nodes
}
