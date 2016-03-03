#DESIGN

##GOAL

**Efficiency**: (based on udp)
 * midi messages are delivered as quickly as possible without any abstraction over udp

**Reliability**: (based on tcp)
 * The midi state of peers and the authority are the same. (with a synchronisation time > 0)
 * The midi state of the authority is computed from midi message ordered by their timestamp

**Robustness**:
 * Peer disconnection doesn't result in endless note on.
 * (but don't return to default settings as peer can change settings _of_ all others)

##THOUGHT

* no channel/peer attribution?
* no soundfont management?
* no built in midi output, a build in one would avoid the communication between processus but it doesn't seem to worth it.
* no built in midi input. Idem output.
* some midi message from some midi input devices are redondant. Does it worth it to remove those for more efficiency?
* when a note played from a synth it can result in several midi messages. Does it worth it to combine them in a unique udp packet?
* no session policy like some people heard others but not all and not in duplex ?

##ARCHITECTURE

centralized authority.

```
###OLD udp protocol:

* message:
  * id of the later message: i32 // the packet id 0 contain message 0 to -n
  * message[id]: Midi
  * message[id-1]: Midi
  * message[id-2]: Midi
  * ...
  * message[id-n]: Midi


###OLD tcp protocol

* request introduction
* answer introduction request: (channel,peers)
* introduce itself
* request the state (all critical state variable: instruement, ..it's all?)
* answer state request: (id,state)
* quit
```

##Midi output suggestions

* **timidiy**: some parameters for real time playing:
  * `-B 1,1` buffer fragments
  * `-f` toggles fast enveloppes

##TODO

* client and server in the same git
* midi state structure
* error detection and resolution:
  (inspired by real time multiplayer game method)
  * a thread receive every midi event (from input after being send through udp, from peers after being send to midi outpun).
  * it send to the authority the midi event through tcp (bundle into packet)
  * it receive authority midi state every tick:
    the midi snapshot contain the current midi state and for each peers the last timestamp computed.
  * on recv:
    it add to the snapshot all local midi event with timestamp > those contained by the snapshot.
	if local midi state is different from snapshot recomputation then it resolve the difference by
	sending some event to the local output

