# miditip - MIDI Through IP

it is still in very early development,

miditip allow to create sessions where all peers
send their midi message to each other.

You can so play jam session or repetition via internet.

#GOAL

**Efficiency**: (based on udp)
 * midi messages are delivered as quickly as possible without any abstraction over udp

**Reliability**: (based on tcp)
 * The midi state of peers and the authority are the same. (with a synchronisation time > 0)
 * The midi state of the authority is computed from midi message ordered by their timestamp

**Robustness**:
 * Peer disconnection doesn't result in endless note on.
 * (but don't return to default settings as peer can change settings _of_ all others)

#NON GOAL

**SoundFont management**: doesn't ensure that all peers have the same soundfont.

**Midi Input**: it will always need a midi input device

**Midi Output**: it will always need a midi output device

However midi output and soundfont
management may be provided by another software in
the future that will integrate miditip in its core.

#install

there is no binary available for now so here are the instruction to compile it.

to use it you'll need a midi input and output devices. Some are suggested in the [use section](#use)

###dependencies

miditip depend on:
* the portmidi library (for the client only)

  On Ubuntu / Debian:
  ```sh
  apt-get install libportmidi-dev
  ```

  On OSX (Homebrew):
  ```sh
  brew install portmidi
  ```
  On OSX, if you get a linker error `ld: library not found for -lportmidi`, either,
   - make sure you have the Xcode Command Line Tools installed, not just Xcode, or
   - make sure you have the PortMidi library in your `$LIBRARY_PATH`, e.g. for Homebrew:

     ```sh
     export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
     ```
* lzma library
  On Ubuntu / Debian:
  ```sh
  apt-get install liblzma-dev
  ```

  On other system I don't know... for any help open an issue.

###compile

it must work on the stable rust.

it has been tested on rust1.7

instructions are for Ubuntu / Debian, but you may be able to adapt on other distribution.

* install rust stable compiler available [there](https://www.rust-lang.org/downloads.html)

* get the source code

  ```sh
  git clone https://github.com/thiolliere/miditip.git
  ```

* go to the directory

  ```sh
  cd miditip/client
  ```

  (or for the server `cd miditip/server`)

* compile

  ```sh
  cargo build --release
  ```

* execute with help

  ```sh
  ./target/release/miditip -h
  ```

#use

##server

`miditip-server -h` is explicit enough

##client

to use the client you need a midi input device and a midi output device

###midi input

**vkeybd** is a minimalist midi input device that does the work if you just want to test miditip

###midi ouput

**timidity** is a multiplatform software to play midi.

Some argument are needed for real time playing.
* `-B 1,1 ` buffer fragments if the audio isn't good enouch you can use `-B x,x` with x>1
* `-f ` toggles fast enveloppes

###launch

* lauch the midi output device
  ```sh
  timidity -f -B 1.1 -iA &
  ```

* launch the midi input device
  ```sh
  vkeybd &
  ```

* look at the midi devices available
  ```sh
  miditip -d
  ```

* launch miditip an the input device 6 the output device 2 and the server 46.101.45.181:8888
  ```sh
  miditip 6 2 46.101.45.181:8888
  ```

#troubleshooting

###audio

you may also need to install `libasound2-dev` for using ALSA instead of OSS

