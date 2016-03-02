extern crate argparse;
extern crate portmidi;
extern crate serde_json;
extern crate common;

use common::{
    ClientMsg,
    ServerMsg,
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
use std::io::{
    Read,
    Write,
    ErrorKind,
};
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
    TryRecvError,
    channel,
};

fn init_portmidi(input_id: PortMidiDeviceId, output_id: PortMidiDeviceId) -> PortMidiResult<(InputPort,OutputPort)> {
    try!(portmidi::initialize());

    let device = try!(portmidi::get_device_info(input_id).ok_or(InvalidDeviceId));
    println!("Opening: {}", device.name);

    let mut input = portmidi::InputPort::new(input_id, 1024);
    try!(input.open());

    let device = try!(portmidi::get_device_info(output_id).ok_or(InvalidDeviceId));
    println!("Opening: {}", device.name);

    let mut output = portmidi::OutputPort::new(output_id, 1024);
    try!(output.open());

    Ok((input,output))
}

fn from_raw_to_midi_msg(raw: [u8;3]) -> MidiMessage {
    MidiMessage {
        status: raw[0],
        data1: raw[1],
        data2: raw[2],
    }
}

fn from_midi_msg_to_raw(msg: MidiMessage) -> [u8;3] {
    [msg.status,msg.data1,msg.data2]
}

fn init_udp_socket() -> io::Result<UdpSocket> {
    let timeout = Duration::new(0,1);

    for p in 49152..65535 {
        if let Ok(udp_socket) = UdpSocket::bind(("localhost",p)) {
            try!(udp_socket.set_read_timeout(Some(timeout)));
            try!(udp_socket.set_write_timeout(Some(timeout)));
            return Ok(udp_socket);
        }
    }
    Err(io::Error::new(io::ErrorKind::Other,"no port available"))
}

fn init_server_stream(server_addr: SocketAddr,udp_socket_addr: SocketAddr) -> io::Result<Receiver<ServerMsg>> {
    let (tx,rx) = channel();

    let mut server_stream = try!(TcpStream::connect(server_addr));
    let msg = serde_json::to_vec(&ClientMsg::NewPeer(udp_socket_addr)).unwrap();
    try!(server_stream.write_all(&msg));
    thread::spawn(move || {
        loop {
            let mut msg = Vec::new();
            server_stream.read_to_end(&mut msg).unwrap();
            tx.send(serde_json::from_slice(&msg).unwrap()).unwrap();
        }
    });
    Ok(rx)
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
    loop {
        match client.step() {
            Ok(_) => (),
            Err(e) => {
                println!("Error:3 updating client {}",e);
                return;
            }
        }
    }
}

struct Client {
    input: InputPort,
    output: OutputPort,
    udp_socket: UdpSocket,
    server_recv: Receiver<ServerMsg>,
    peers: Vec<SocketAddr>,
    udp_socket_addr: SocketAddr,
    midi_msg: [u8;3],
}

const RESET_MIDI: [MidiMessage;2] = [
    MidiMessage {
        status: 0b10110000,
        data1: 123,
        data2: 0,
    },
    MidiMessage {
        status: 0b10110000,
        data1: 121,
        data2: 0,
    }
];

impl Client {
    fn new(options: Options) -> io::Result<Client> {
        println!("init udp socket");
        let udp_socket = try!(init_udp_socket());
        let addr = try!(udp_socket.local_addr());
        println!("init tcp stream");
        let server_recv = try!(init_server_stream(options.server,addr));

        println!("init port midi io");
        let (input,output) = match init_portmidi(options.input,options.output) {
            Ok(d) => d,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other,e)),
        };


        Ok(Client {
            input: input,
            output: output,
            udp_socket: udp_socket,
            udp_socket_addr: addr,
            server_recv: server_recv,
            peers: vec!(),
            midi_msg: [0u8;3],
        })
    }

    fn step(&mut self) -> io::Result<()> {
        match self.server_recv.try_recv() {
            Ok(ServerMsg::NewPeerList(mut list)) => {
                println!("receive list");
                list.retain(|&addr| addr != self.udp_socket_addr);
                self.peers = list;
                println!("reset midi {:#?}",RESET_MIDI);
                for &msg in RESET_MIDI.iter() {
                    if let Err(e) = self.output.write_message(msg) {
                        return Err(io::Error::new(io::ErrorKind::Other,e));
                    }
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => return Err(io::Error::new(io::ErrorKind::Other,"server_recv disconnected")),
        }

        for _ in 0..5 {
            match self.input.read() {
                Ok(Some(event)) => {
                    self.midi_msg = from_midi_msg_to_raw(event.message);
                    for &peer in &self.peers {
                        try!(self.udp_socket.send_to(&self.midi_msg,peer));
                    }
                    if let Err(e) = self.output.write_event(event) {
                        return Err(io::Error::new(io::ErrorKind::Other,e));
                    }
                    //println!("midi event from input {:#?}",self.midi_msg);
                },
                Ok(None) => (),
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput,e)),
            }

            match self.udp_socket.recv_from(&mut self.midi_msg) {
                Ok((3,_)) => {
                    let midi_msg = from_raw_to_midi_msg(self.midi_msg);
                    if let Err(e) = self.output.write_message(midi_msg) {
                        return Err(io::Error::new(io::ErrorKind::Other,e));
                    }
                    //println!("midi message from network {:#?}",midi_msg);
                },
                Ok((_,_)) => return Err(io::Error::new(io::ErrorKind::InvalidData,"")),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::WouldBlock => (),
                        _ => return Err(e),
                    }
                }
            }
        }

        Ok(())
    }
}
