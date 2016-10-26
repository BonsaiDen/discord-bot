// STD Dependencies -----------------------------------------------------------
use std::sync::mpsc::Receiver;
use std::collections::VecDeque;


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;
use discord::voice::AudioReceiver;


// Recorder Commands ----------------------------------------------------------
pub enum RecorderCommand {
    Start,
    Pause,
    Stop
}


// Audio Recorder Abstraction -------------------------------------------------
pub struct Recorder {
    id: u64,
    command_queue: Receiver<RecorderCommand>,
    command_buffer: VecDeque<RecorderCommand>
}


// Recorder Receiver Implementation -------------------------------------------
impl AudioReceiver for Recorder {

    fn speaking_update(
        &mut self,
        ssrc: u32,
        user_id: &UserId,
        _: bool
    ) {

    }

    fn voice_packet(
        &mut self,
        ssrc: u32,
        sequence: u16,
        timestamp: u32,
        stereo: bool,
        data: &[i16]
    ) {

    }

}

