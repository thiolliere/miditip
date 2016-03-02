#VRAC (french)

###command-line interface

arguments:
  * -i, --input=DEVICE: midi input device, midi messages will be send over the network and also played locally.
  * -o, --output=DEVICE: midi output device,
  * -s, --socket=PORT: udp/tcp socket port,
  * -p, --peer="ip:port": peer to request introduction,

###udp protocol:

* message:
  * id of the later message: i32 // the packet id 0 contain message 0 to -n
  * message[id]: Midi
  * message[id-1]: Midi 
  * message[id-2]: Midi 
  * ...
  * message[id-n]: Midi 


###tcp protocol

* request introduction
* answer introduction request: (channel,peers)
* introduce itself
* request the state (all critical state variable: instruement, ..it's all?)
* answer state request: (id,state)
* quit

#restart

move to a centralized architecture:

the authority
the peers

#timidiy 

in order to use timidiy efficiently for real time playing some parameters:
* -B 1,1 buffer fragments
* -f toggles fast enveloppes

#steps

* server
* client can call the server to reset a peer (so the peer have to resent all controller setttings)
* client store their midi state and resend the setting when they receive reset msg
* client check if some udp packet are lost ask a reset if any
