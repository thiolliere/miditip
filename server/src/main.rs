extern crate argparse;
extern crate common;

use common::{
    ServerInitMsg,
    ServerMsg,
    ClientInitMsg,
    ClientMsg,
    MiditipState,
    send,
    recv,
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
    TcpStream,
};
use std::io;
use std::io::ErrorKind;
use std::collections::HashMap;
use std::sync::mpsc::{
    Receiver,
    channel,
    TryRecvError,
};
use std::thread;
use std::time::Duration;

struct Peer {
    id: u8,
    addr: SocketAddr,
    stream: TcpStream,
}

struct Server {
    buffer: [u8;4096],
    peers: HashMap<u8,Peer>,
    miditip_state: MiditipState,
    listener: Receiver<(TcpStream,SocketAddr)>,
}

impl Server {
    fn new(addr: SocketAddr) -> io::Result<Server> {
        let tcp_listener = try!(TcpListener::bind(addr));
        let (listener_tx,listener_rx) = channel();
        thread::spawn(move || {
            while let Ok((stream,addr)) = tcp_listener.accept() {
                listener_tx.send((stream,addr)).unwrap();
            }
        });

        Ok(Server {
            buffer: [0u8;4096],
            peers: HashMap::new(),
            miditip_state: MiditipState::new(),
            listener: listener_rx,
        })
    }

    fn unused_peer_id(&self) -> Option<u8> {
        for i in 0..255 {
            if !self.peers.contains_key(&i) {
                return Some(i);
            }
        }
        if !self.peers.contains_key(&255) {
            Some(255)
        } else {
            None
        }

    }

    fn accept_peer(&mut self, mut stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
        try!(stream.set_read_timeout(Some(Duration::from_secs(2))));
        let port = match try!(recv(&mut self.buffer, &mut stream)) {
            ClientInitMsg::NewPeer(port) => port,
        };
        let id = try!(self.unused_peer_id().ok_or(io::Error::new(io::ErrorKind::Other,"FullSession")));
        try!(send(&ServerInitMsg::PeerId(id), &mut stream));
        let udp_socket_addr = match addr {
            SocketAddr::V4(addr) => SocketAddr::V4(SocketAddrV4::new( *addr.ip(), port)),
            SocketAddr::V6(addr) => SocketAddr::V6(SocketAddrV6::new( *addr.ip(), port, addr.flowinfo(), addr.scope_id())),
        };
        try!(stream.set_read_timeout(Some(Duration::from_millis(20))));

        let peer = Peer {
            id: id,
            addr: udp_socket_addr,
            stream: stream,
        };
        self.peers.insert(peer.id,peer);
        self.send_peer_list();
        Ok(())
    }

    fn remove_peer(&mut self, id: u8) {
        println!("remove peer: {}",id);
        self.peers.remove(&id);
        self.send_peer_list();
    }

    fn send_peer_list(&mut self) {
        let peer_list: Vec<SocketAddr> = self.peers.values()
            .map(|peer| peer.addr)
            .collect();
        println!("send new peer list: {:#?}",peer_list);
        let mut tokill = Vec::new();
        for (_,peer) in &mut self.peers {
            let mut other_peer_list = peer_list.clone();
            other_peer_list.retain(|&addr| addr != peer.addr);
            match send(&ServerMsg::NewPeerList(other_peer_list), &mut peer.stream) {
                Ok(()) => (),
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
                    _ => tokill.push(peer.id),
                },
            }
        }
        for id in tokill {
            self.remove_peer(id);
        }
    }

    fn run(&mut self) -> io::Result<()> {
        let mut counter_timer = 0;
        loop {
            match self.listener.try_recv() {
                Ok((stream,addr)) => {
                    match self.accept_peer(stream,addr) {
                        Ok(()) => println!("peer accepted"),
                        Err(e) => println!("peer refused: {}",e),
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return Err(io::Error::new(io::ErrorKind::ConnectionAborted,"listener channel disconnected")),
            }
            let mut tokill = Vec::new();
            for (_,peer) in &mut self.peers {
                match recv(&mut self.buffer, &mut peer.stream) {
                    Ok(msg) => match msg {
                        ClientMsg::MiditipEvent(event) => {self.miditip_state.modify(&event);},
                    },
                    Err(e) => match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
                        _ => tokill.push(peer.id),
                    },
                }
            }
            for id in tokill {
                self.remove_peer(id);
            }

            let mut tokill = Vec::new();
            if counter_timer > 10 {
                counter_timer = 0;
                for (_,peer) in &mut self.peers {
                    match send(&ServerMsg::MiditipState(self.miditip_state.clone()),&mut peer.stream) {
                        Ok(()) => (),
                        Err(_e) => {
                            println!("error while send miditipState: {}",_e);
                            tokill.push(peer.id);
                        }
                    }
                }
            } else {
                counter_timer += 1;
            }
            for id in tokill {
                self.remove_peer(id);
            }
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

