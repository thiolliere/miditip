#Mititip

MIDI Throught IP.
early development. not 0.1 yet, go on your own way.

###test

first you'll need to install:
* rustc1.6 available [there](https://www.rust-lang.org/downloads.html)
* a midi output like `timidity`
* a midi input like `vkeybd`
* the portmidi library:

  On Ubuntu / Debian:
  ```sh
  apt-get install libportmidi-dev
  ```
  and you may also need libasound2-dev for using ALSA instead of OSS
  
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


###steps

* build miditip:
  * `git clone https://github.com/thiolliere/miditip.git`
  * `cd miditip`
  * `cargo build --release`
  * `./target/release/miditip -h`

* launch:
  * `timidity -f -B 1.1 -iA &` note that if the sound if too bad you can change -B x,x with x>1
  * `vkeybd &`
  * `miditip -d` to see the midi devices available
  * `miditip 6 2 46.101.45.181:8888` argument are `INPUT OUTPUT SERVER_IP:SERVER_PORT`
 
