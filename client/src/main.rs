extern crate argparse;
extern crate portmidi;
extern crate common;

use common::{
    ClientInitMsg,
    ServerInitMsg,
    ClientMsg,
    ServerMsg,
    MiditipState,
    MiditipEvent,
    send,
    recv,
};
use argparse::{
    ArgumentParser,
    Store,
    Print,
};
use std::net::{
    TcpStream,
    SocketAddr,
    SocketAddrV4,
    UdpSocket,
    Ipv4Addr,
};
use std::io;
use std::io::ErrorKind;
use std::time::Duration;
use portmidi::{
    PortMidiDeviceId,
    PortMidiResult,
    MidiMessage,
    InputPort,
    OutputPort,
};
use portmidi::PortMidiError::InvalidDeviceId;
use std::thread;
use std::sync::mpsc::{
    Receiver,
    Sender,
    TryRecvError,
    channel,
};

enum Event {
    NewPeerList(Vec<SocketAddr>),
    MidiMessages(Vec<MidiMessage>),
}

fn init_portmidi(input_id: PortMidiDeviceId, output_id: PortMidiDeviceId) -> PortMidiResult<(InputPort,OutputPort)> {
    try!(portmidi::initialize());

    let device = try!(portmidi::get_device_info(input_id).ok_or(InvalidDeviceId));
    println!("Opening: {}", device.name);

    let mut input = portmidi::InputPort::new(input_id, 4096);
    try!(input.open());

    let device = try!(portmidi::get_device_info(output_id).ok_or(InvalidDeviceId));
    println!("Opening: {}", device.name);

    let mut output = portmidi::OutputPort::new(output_id, 4096);
    try!(output.open());

    Ok((input,output))
}

fn from_raw_miditip_event_to_midi_msg(raw: &[u8;5]) -> MidiMessage {
    MidiMessage {
        status: raw[0],
        data1: raw[1],
        data2: raw[2],
    }
}

fn init_udp_socket() -> io::Result<UdpSocket> {
    let timeout = Duration::new(0,1);

    for p in 8000..65535 {
        if let Ok(udp_socket) = UdpSocket::bind(("0.0.0.0",p)) {
            try!(udp_socket.set_read_timeout(Some(timeout)));
            try!(udp_socket.set_write_timeout(Some(timeout)));
            return Ok(udp_socket);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other,"no port available"))
}

fn init_server_stream(server_addr: SocketAddr,udp_socket_addr: SocketAddr) -> io::Result<(Receiver<Event>,Sender<[u8;5]>,u8)> {
    let (event_tx,event_rx) = channel();
    let (thread_tx,thread_rx) = channel();

    let mut stream = try!(TcpStream::connect(server_addr));
    try!(send(&ClientInitMsg::NewPeer(udp_socket_addr.port()), &mut stream));
    try!(stream.set_read_timeout(Some(Duration::from_secs(5))));
    let mut buffer = [0u8;4096];
    let peer_id = match try!(recv(&mut buffer, &mut stream)) {
        ServerInitMsg::PeerId(id) => id,
    };
    try!(stream.set_read_timeout(Some(Duration::from_millis(10))));
    thread::spawn(move || {
        let mut miditip_state = MiditipState::new();
        let mut miditip_events: Vec<MiditipEvent> = Vec::new();
        loop {
            match recv(&mut buffer, &mut stream) {
                Ok(ServerMsg::NewPeerList(list)) => event_tx.send(Event::NewPeerList(list)).unwrap(),
                Ok(ServerMsg::MiditipState(mut server_miditip_state)) => {
                    miditip_events.retain(|event| {
                        server_miditip_state.modify(event)
                    });
                    let msgs = miditip_state.resolve(&server_miditip_state)
                        .iter()
                        .map(|&me| MidiMessage {
                            status: me[0],
                            data1: me[1],
                            data2: me[2]})
                        .collect();
                    event_tx.send(Event::MidiMessages(msgs)).unwrap();
                },
                Err(e) => {
                    match e.kind() {
                        ErrorKind::TimedOut | ErrorKind::WouldBlock => (),
                        _ => {
                            println!("recv error: {} \n kind:{:#?}",e,e.kind());
                            return;
                        }
                    }
                }
            }
            match thread_rx.try_recv() {
                Ok(miditip_event) => {
                    let miditip_event = MiditipEvent::from_array(&miditip_event);
                    miditip_state.modify(&miditip_event);
                    miditip_events.push(miditip_event.clone());
                    send(&ClientMsg::MiditipEvent(miditip_event), &mut stream).unwrap();
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return,
            }
        }
    });
    Ok((event_rx,thread_tx,peer_id))
}

struct Options {
    input: PortMidiDeviceId,
    output: PortMidiDeviceId,
    server: SocketAddr,
}

impl Options {
    fn new() -> Options {
        Options {
            input: 0,
            output: 0,
            server: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),0)),
        }
    }
}

fn get_devices_display() ->  String {
    let devices = {
        portmidi::initialize().unwrap();
        let no = portmidi::count_devices();
        let devices = (0..no).filter_map(|i| portmidi::get_device_info(i))
            .collect::<Vec<_>>();
        portmidi::terminate().unwrap();
        devices
    };

    let mut display = String::new();
    display.push_str("Id  Name                 Input? Output?\n");
    display.push_str("=======================================\n");
    for d in devices.into_iter() {
        display.push_str(&*format!("{:<3} {:<20} {:<6} {:<6}\n", d.device_id, d.name, d.input, d.output));
    }
    display
}

pub fn main() {
    let mut options = Options::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("send MIDI Throught IP - client");
        ap.add_option(&["-V", "--version"],
                      Print(env!("CARGO_PKG_VERSION").to_string()), "Show version");
        ap.add_option(&["-d", "--devices"],
                      Print(get_devices_display()), "display midi devices");
        ap.refer(&mut options.input).required()
            .add_argument("input", Store,
                          "midi intput device");
        ap.refer(&mut options.output).required()
            .add_argument("output", Store,
                          "midi output device");
        ap.refer(&mut options.server).required()
            .add_argument("server", Store,
                          "server ip:port");
        ap.parse_args_or_exit();
    }

    let mut client = match Client::new(options) {
        Ok(c) => c,
        Err(e) => {
            println!("Error:2 create client {}",e);
            return;
        },
    };
    match client.run() {
        Ok(_) => (),
        Err(e) => {
            println!("Error:3 run client {}",e);
            return;
        }
    }
}

struct Client {
    peer_id: u8,
    msg_id: u8,
    input: InputPort,
    output: OutputPort,
    udp_socket: UdpSocket,
    event_receiver: Receiver<Event>,
    thread_sender: Sender<[u8;5]>,
    peers: Vec<SocketAddr>,
    miditip_msg_buffer: [u8;5],
}

impl Client {
    fn new(options: Options) -> io::Result<Client> {
        println!("init udp socket");
        let udp_socket = try!(init_udp_socket());
        let addr = try!(udp_socket.local_addr());
        println!("udp socket bind on {:?}",addr);
        println!("init tcp stream");
        let (recv,sndr,peer_id) = try!(init_server_stream(options.server,addr));

        println!("init port midi");
        let (input,output) = match init_portmidi(options.input,options.output) {
            Ok(d) => d,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other,e)),
        };


        Ok(Client {
            peer_id: peer_id,
            msg_id: 0,
            input: input,
            output: output,
            udp_socket: udp_socket,
            event_receiver: recv,
            thread_sender: sndr,
            peers: vec!(),
            miditip_msg_buffer: [0u8;5],
        })
    }

    ///! wrap the midi_message into a miditip_event for the peer
    fn miditip_event(&mut self, midi_message: MidiMessage) -> MiditipEvent {
        self.msg_id.wrapping_add(1);
        MiditipEvent {
            status: midi_message.status,
            data1: midi_message.data1,
            data2: midi_message.data2,
            peer_id: self.peer_id,
            msg_id: self.msg_id,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            match self.event_receiver.try_recv() {
                Ok(Event::NewPeerList(list)) => { self.peers = list; },
                Ok(Event::MidiMessages(midi_msgs)) => {
                    for midi_msg in midi_msgs {
                        try!(self.output.write_message(midi_msg)
                             .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                        println!("midi event from resolution {:#?}",midi_msg);
                        let miditip_msg = self.miditip_event(midi_msg).to_array();
                        try!(self.thread_sender.send(miditip_msg)
                             .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return Err(io::Error::new(io::ErrorKind::Other,"server connection failed")),
            }
            match self.input.read() {
                Ok(Some(event)) => {
                    let miditip_event = self.miditip_event(event.message);
                    let miditip_event_array = miditip_event.to_array();
                    for &peer in &self.peers {
                        try!(self.udp_socket.send_to(&miditip_event_array,peer));
                    }
                    try!(self.output.write_event(event)
                         .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                    try!(self.thread_sender.send(miditip_event_array)
                         .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                    println!("midi event from input {:#?}",miditip_event);
                },
                Ok(None) => (),
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput,e)),
            }
            match self.udp_socket.recv_from(&mut self.miditip_msg_buffer) {
                Ok((5,_addr)) => {
                    let midi_msg = from_raw_miditip_event_to_midi_msg(&self.miditip_msg_buffer);
                    try!(self.output.write_message(midi_msg)
                         .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                    try!(self.thread_sender.send(self.miditip_msg_buffer)
                         .map_err(|e| io::Error::new(io::ErrorKind::Other,e)));
                    println!("midi message from peer {:?} {:#?}",_addr,midi_msg);
                },
                Ok((_,_)) => return Err(io::Error::new(io::ErrorKind::InvalidData,"")),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::WouldBlock | ErrorKind::TimedOut => (),
                        _ => return Err(e),
                    }
                }
            }
        }
    }
}

