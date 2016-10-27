// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Audio Chunk Abstraction ----------------------------------------------------
pub struct VoicePacket {
    pub sequence: u16,
    pub timestamp: u32,
    pub received: u64,
    pub channels: usize,
    pub data: Vec<i16>
}

pub struct Chunk {
    pub duration: u32,
    pub track_offset: u64,
    pub channels: usize,
    pub voice_packets: Vec<VoicePacket>
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[Chunk {}ms ({} offset) ({} channels) ({} packets)]",
            self.duration,
            self.track_offset,
            self.channels,
            self.voice_packets.len()
        )
    }
}

