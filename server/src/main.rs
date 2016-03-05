extern crate argparse;
extern crate serde_json;
extern crate common;

use common::{
    ServerInitMsg,
    ServerMsg,
    ClientInitMsg,
    ClientMsg,
    MiditipState,
    MiditipEvent,
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
    ErrorKind,
};
use std::collections::HashMap;
use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
    TryRecvError,
};
use std::thread;
use std::time::Duration;
use std::sync::{ Arc, Mutex };

enum Event {
    RemovePeer(u8),
    NewPeer(Peer),
    MiditipEvents(Vec<MiditipEvent>),
    Second,
}

struct Peer {
    id: u8,
    addr: SocketAddr,
    sender: Sender<ServerMsg>,
}

struct Server {
    server_rx: Receiver<Event>,
    peers: HashMap<u8,Peer>,
    peer_id_used: Arc<Mutex<Vec<u8>>>,
    miditip_state: MiditipState,
}

impl Server {
    fn new(addr: SocketAddr) -> io::Result<Server> {
        let tcp_listener = try!(TcpListener::bind(addr));
        let peer_id_used = Arc::new(Mutex::new(Vec::new()));
        let peer_id_used_clone = peer_id_used.clone();

        let (server_tx,server_rx) = channel();

        {
            let server_tx = server_tx.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    server_tx.send(Event::Second).unwrap();
                }
            });
        }

        thread::spawn(move || {
            loop {
                if let Ok((mut stream,peer_addr)) = tcp_listener.accept() {
                    println!("new stream {:?}",peer_addr);

                    let mut msg = [0u8;1024];

                    if let Err(_) = stream.set_read_timeout(Some(Duration::from_secs(5))) {
                        println!("fail to set read timemout");
                        continue;
                    }
                    if let Ok(size) = stream.read(&mut msg) {
                        let (msg,_) = msg.split_at(size);
                        let res: Result<ClientInitMsg,serde_json::Error> = serde_json::from_slice(&msg);
                        println!("stream msg: {:?}",res);
                        if let Ok(ClientInitMsg::NewPeer(udp_socket_addr)) = res {
                            let peer_id = {
                                let mut list = peer_id_used.lock().unwrap();
                                let mut id = None;
                                for i in 0..255 {
                                    if !list.contains(&i) {
                                        list.push(i);
                                        id = Some(i);
                                        break;
                                    }
                                }
                                id
                            };
                            if let Some(peer_id) = peer_id {
                                let msg = ServerInitMsg::PeerId(peer_id);
                                if let Err(_) = stream.write_all(&serde_json::to_vec(&msg).expect("serde json fail1")) {
                                    println!("fail to set write server init msg");
                                    continue;
                                }
                                let udp_port = match udp_socket_addr {
                                    SocketAddr::V4(addr) => addr.port(),
                                    SocketAddr::V6(addr) => addr.port(),
                                };
                                let udp_socket_addr = match peer_addr {
                                    SocketAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new( *addr.ip(), udp_port)),
                                    SocketAddr::V6(addr) => SocketAddr::V6(SocketAddrV6::new( *addr.ip(), udp_port, addr.flowinfo(), addr.scope_id())),
                                };
                                println!("udp socket :{}",udp_socket_addr);
                                let (peer_tx,peer_rx) = channel();
                                let server_tx_b = server_tx.clone();

                                if let Err(_) = stream.set_read_timeout(Some(Duration::from_millis(10))) {
                                    println!("fail to set read timemout");
                                    continue;
                                }

                                let peer = Peer {
                                    id: peer_id,
                                    addr: udp_socket_addr,
                                    sender: peer_tx,
                                };

                                thread::spawn(move || {
                                    let mut buffer = [0u8;1024];

                                    loop {
                                        match peer_rx.try_recv() {
                                            Ok(ServerMsg::MiditipState(miditip_state)) => {
                                                //TODO
                                                //if let Err(_) = stream.write_all(&serde_json::to_vec(&ServerMsg::MiditipState(miditip_state)).expect("serde json fail2")) {
                                                //    server_tx_b.send(Event::RemovePeer(peer_id)).unwrap();
                                                //    //do not break: it is the responsability of the server
                                                //    //to end the channel
                                                //}
                                            },
                                            Ok(ServerMsg::NewPeerList(list)) => {
                                                if let Err(_) = stream.write_all(&serde_json::to_vec(&ServerMsg::NewPeerList(list)).expect("serde json fail3")) {
                                                    server_tx_b.send(Event::RemovePeer(peer_id)).unwrap();
                                                    //do not break: it is the responsability of the server
                                                    //to end the channel
                                                }
                                            }
                                            Err(TryRecvError::Empty) => (),
                                            Err(TryRecvError::Disconnected) => return,
                                        }
                                        match stream.read(&mut buffer) {
                                            Ok(size) => {
                                                let (msg,_) = buffer.split_at(size);
                                                let res: Result<ClientMsg,serde_json::Error> = serde_json::from_slice(&msg);
                                                match res {
                                                    Ok(ClientMsg::MiditipEvents(events)) => server_tx_b.send(Event::MiditipEvents(events)).unwrap(),
                                                    Err(_) => server_tx_b.send(Event::RemovePeer(peer_id)).unwrap(),
                                                }
                                            },
                                            Err(e) => {
                                                match e.kind() {
                                                    ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
                                                    _ => server_tx_b.send(Event::RemovePeer(peer_id)).unwrap(),
                                                    //do not break: it is the responsability of the server
                                                    //to end the channel
                                                }
                                            }
                                        }
                                    }
                                });
                                if let Err(_) = server_tx.send(Event::NewPeer(peer)) {
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
            peer_id_used: peer_id_used_clone,
            miditip_state: MiditipState::new(),
        })
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            match self.server_rx.recv() {
                Ok(Event::NewPeer(peer)) => {
                    println!("new peer: {}",peer.id);
                        self.peers.insert(peer.id,peer);
                        self.send_peer_list();
                },
                Ok(Event::RemovePeer(peer_id)) => {
                    println!("remove peer: {}",peer_id);
                    {
                        let mut list = self.peer_id_used.lock().unwrap();
                        list.retain(|&id| id != peer_id);
                        self.peers.remove(&peer_id);

                    }
                    self.send_peer_list();
                },
                Ok(Event::MiditipEvents(events)) => {
                    for event in events {
                        self.miditip_state.modify(event);
                    }
                },
                Ok(Event::Second) => {
                    for peer in self.peers.values() {
                        peer.sender.send(ServerMsg::MiditipState(self.miditip_state.clone())).unwrap();
                    }
                },
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other,"server_rx disconnected")),
            }
        }
    }

    fn send_peer_list(&mut self) {
        let peer_list: Vec<SocketAddr> = self.peers.values()
            .map(|peer| peer.addr)
            .collect();
        println!("send new peer list: {:#?}",peer_list);
        for peer in self.peers.values() {
            let mut other_peer_list = peer_list.clone();
            other_peer_list.retain(|&addr| addr != peer.addr);
            peer.sender.send(ServerMsg::NewPeerList(other_peer_list)).unwrap();
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

