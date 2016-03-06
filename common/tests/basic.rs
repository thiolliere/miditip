extern crate common;


#[test]
fn test_send_recv() {
    use common::*;
    use std::net::*;

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),8866));
    let listener = TcpListener::bind(addr).unwrap();
    let mut client_stream = TcpStream::connect(addr).unwrap();
    let (mut server_stream,_) = listener.accept().unwrap();

    send(&ServerInitMsg::PeerId(10), &mut server_stream).unwrap();
    let mut buffer = [0u8;1024];
    assert!(match recv(&mut buffer, &mut client_stream).unwrap() {
        ServerInitMsg::PeerId(10) => true,
        _ => false,
    });

    send(&ServerMsg::NewPeerList(vec!()), &mut server_stream).unwrap();
    let mut buffer = [0u8;1024];
    assert!(match recv(&mut buffer, &mut client_stream).unwrap() {
        ServerMsg::NewPeerList(list) => list.len() == 0,
        _ => false,
    });

    let mut miditip_state = MiditipState::new();
    miditip_state.modify(&MiditipEvent {
        status: 144,
        data1: 59,
        data2: 126,
        peer_id: 0,
        msg_id: 1,
    });

    miditip_state.modify(&MiditipEvent {
        status: 128,
        data1: 59,
        data2: 0,
        peer_id: 0,
        msg_id: 2,
    });

    send(&ServerMsg::MiditipState(miditip_state), &mut server_stream).unwrap();
    let mut buffer = [0u8;1024];
    assert!(match recv(&mut buffer, &mut client_stream).unwrap() {
        ServerMsg::MiditipState(_) => true,
        _ => false,
    });
}
