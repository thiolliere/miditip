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
    SocketAddrV6,
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
                if let Ok((mut stream,peer_addr)) = tcp_listener.accept() {
                    println!("new stream {:?}",peer_addr);

                    let mut msg = [0u8;1024];

                    if let Ok(size) = stream.read(&mut msg) {
                        let (msg,_) = msg.split_at(size);
                        let res: Result<ClientMsg,serde_json::Error> = serde_json::from_slice(&msg);
                        println!("stream msg: {:?}",res);
                        if let Ok(ClientMsg::NewPeer(udp_socket_addr)) = res {
                            let udp_port = match udp_socket_addr {
                                SocketAddr::V4(addr) => addr.port(),
                                SocketAddr::V6(addr) => addr.port(),
                            };
                            let udp_socket_addr = match peer_addr {
                                SocketAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new( *addr.ip(), udp_port)),
                                SocketAddr::V6(addr) => SocketAddr::V6(SocketAddrV6::new( *addr.ip(), udp_port, addr.flowinfo(), addr.scope_id())),
                            };
                            println!("udp socket :{}",udp_socket_addr);
                            if let Ok(mut stream_b) = stream.try_clone() {

                                let (peer_tx,peer_rx) = channel();
                                let server_tx_b = server_tx.clone();
                                let server_tx_bb = server_tx.clone();

                                thread::spawn(move || {
                                    while let Ok(msg) = peer_rx.recv() {
                                        println!("send msg to peer: {:?}",msg);
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
                    self.peers.remove(&addr);
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
        println!("send new peer list: {:#?}",server_msg);
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

    let mut server = match Server::new(addr) {
        Ok(server) => server,
        Err(e) => {
            println!("Error: {}",e);
            return;
        }
    };
    if let Err(e) = server.run() {
        println!("Error: {}",e);
    }
}

