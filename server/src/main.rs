extern crate argparse;
extern crate serde_json;
extern crate common;

use common::{
    ServerMsg,
    ClientMsg,
};
use argparse::{
    ArgumentParser, 
    Store,
    Print,
};
use std::net::{
    SocketAddr,
    SocketAddrV4,
    Ipv4Addr,
    TcpListener,
};
use std::io;
use std::io::{
    Write,
    Read,
};
use std::collections::HashMap;
use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
};
use std::thread;

enum Event {
    RemovePeer(SocketAddr),
    NewPeer(SocketAddr,Sender<ServerMsg>),
}

struct Server {
    server_rx: Receiver<Event>,
    peers: HashMap<SocketAddr,Sender<ServerMsg>>,
}

impl Server {
    fn new(addr: SocketAddr) -> io::Result<Server> {
        let tcp_listener = try!(TcpListener::bind(addr));

        let (server_tx,server_rx) = channel();
        thread::spawn(move || {
            loop {
                if let Ok((mut stream,_)) = tcp_listener.accept() {

                    let mut msg = [0u8;1024];

                    if let Ok(size) = stream.read(&mut msg) {
                        let (msg,_) = msg.split_at(size);
                        let res: Result<ClientMsg,serde_json::Error> = serde_json::from_slice(&msg);
                        if let Ok(ClientMsg::NewPeer(udp_socket_addr)) = res {
                            if let Ok(mut stream_b) = stream.try_clone() {

                                let (peer_tx,peer_rx) = channel();
                                let server_tx_b = server_tx.clone();
                                let server_tx_bb = server_tx.clone();

                                thread::spawn(move || {
                                    while let Ok(msg) = peer_rx.recv() {
                                        if let Err(_) = stream.write_all(&serde_json::to_vec(&msg).unwrap()) {
                                            server_tx_b.send(Event::RemovePeer(udp_socket_addr)).unwrap();
                                            //do not break: it is the responsability of the server
                                            //to end the channel
                                        }
                                    }
                                });
                                thread::spawn(move || {
                                    let mut msg = [0u8;1];
                                    while let Ok(()) = stream_b.read_exact(&mut msg) {
                                    }
                                    server_tx_bb.send(Event::RemovePeer(udp_socket_addr)).unwrap();
                                });
                                if let Err(_) = server_tx.send(Event::NewPeer(udp_socket_addr,peer_tx)) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Server {
            server_rx: server_rx,
            peers: HashMap::new(),
        })
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            match self.server_rx.recv() {
                Ok(Event::NewPeer(addr,peer_tx)) => {
                    println!("new peer: {}",addr);
                    if self.peers.contains_key(&addr) {
                        self.peers.remove(&addr);
                    }
                    self.peers.insert(addr,peer_tx);
                    self.send_peer_list();
                },
                Ok(Event::RemovePeer(addr)) => {
                    println!("remove peer: {}",addr);
                    self.peers.remove(&addr);
                    self.send_peer_list();
                },
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other,"server_rx disconnected")),
            }
        }
    }

    fn send_peer_list(&mut self) {
        let server_msg = ServerMsg::NewPeerList(self.peers.keys()
                                         .map(|addr| *addr)
                                         .collect());
        for peer_tx in self.peers.values() {
            peer_tx.send(server_msg.clone()).unwrap();
        }
    }
}

pub fn main() {
    let mut addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),0));
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("send MIDI Throught IP - server");
        ap.add_option(&["-V", "--version"],
                      Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");
        ap.refer(&mut addr).required()
            .add_argument("SOCKET_ADDR", Store,
                          "address to bind the server on");
        ap.parse_args_or_exit();
    }

    let mut server = Server::new(addr).unwrap();
    server.run().unwrap();
}

