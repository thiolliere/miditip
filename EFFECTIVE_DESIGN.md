###Client:

two thread:

main thread:
* poll event from a kind of window inspired by piston.
* event are:
  * InputEvent(MidiEvent)
  * ResolutionMidiMessage([u8;5])
  * NetworkMidiMessage([u8;5])
  * NewPeerList(list)
  * ServerConnectionLost
  * InternalError(InternalError)

second thread:
* receive all event to the midi output and keep the miditip state
* manage tcp stream send event and return if fail to write or read
* send miti event to server via tcp
* receive server miditip state and modify and then resolve

###Server

main thread:

* poll event
* event are:
  * NewPeer(addr,stream)
  * RemovePeer(addr)
  * MiditipEvent(miditip event)
  * SendMidiState

###Common

no more ServerInitMsg and ClientInitMsg,
things sendable through tcp implement `to_vec`

###Tcp connection

the client udp socket addr is ipv4.
it send its port on [u8;2]

the server receive the port and compute the addr from its accept addr
and send back the peer identifier
