//! the midi state structure hold:
//! * for each channel:
//!   * notes ON
//!   * instrument
//!   * controllers status

pub const CHANNEL_SIZE: usize = 257;
pub const MIDI_STATE_SIZE: usize = 16*257;

struct ChannelState {
    instrument: u8,
    controllers: [u8;128],
    /// the first bit indicate OFF/ON the 7 later the pitch
    notes: [u8;128],
}

impl ChannelState {
    fn new() -> ChannelState {
        ChannelState {
            instrument: 0,
            controllers: [0u8;128],
            notes: [0u8;128],
        }
    }
    //pub fn into_array(&self) -> [u8;CHANNEL_SIZE] {
    //    let mut array = [0u8;CHANNEL_SIZE];
    //    let mut i = 0;
    //    array[i] = self.instrument;
    //    i+=1;
    //    for j in 0..128 {
    //        array[i] = self.controllers[j];
    //        i+=1;
    //    }
    //    for j in 0..128 {
    //        array[i] = self.notes[j];
    //        i+=1;
    //    }
    //    array
    //}
    //pub fn from_array(array: &[u8;CHANNEL_SIZE]) -> ChannelState {
    //    let mut state = ChannelState::new();
    //    let mut i = 0;
    //    state.instrument = array[i];
    //    i+=1;
    //    for j in 0..128 {
    //        state.controllers[j] = array[i];
    //        i+=1;
    //    }
    //    for j in 0..128 {
    //        state.notes[j] = array[i];
    //        i+=1;
    //    }
    //    state
    //}
}

pub struct MidiState {
    channels: [ChannelState;16],
}

impl MidiState {
    pub fn new() -> MidiState {
        let channels = [
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new(),
            ChannelState::new()];

        MidiState {
            channels: channels,
        }
    }
    //pub fn into_array(&self) -> [u8;MIDI_STATE_SIZE] {
    //    let mut array = [0u8;MIDI_STATE_SIZE];
    //    let mut i = 0;
    //    for j in 0..16 {
    //        let channel = self.channels[j].into_array();
    //        for k in 0..CHANNEL_SIZE {
    //            array[i] = channel[k];
    //            i+=1;
    //        }
    //    }
    //    array
    //}
    //pub fn from_array(array: &[u8;MIDI_STATE_SIZE]) -> MidiState {
    //    let mut state = MidiState::new();
    //    let mut i = 0;
    //    for j in 0..16 {
    //        let mut channel = [0u8;CHANNEL_SIZE];
    //        for k in 0..CHANNEL_SIZE {
    //            channel[k] = array[i];
    //            i+=1;
    //        }
    //        state.channels[j] = ChannelState::from_array(&channel);
    //    }
    //    state
    //}

    //pub fn modify(&mut self, event: MidiEvent) {
    //}
}
