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

I would love to make it completely peer to peer. but it is way more difficult...
