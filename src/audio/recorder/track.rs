// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::sync::mpsc::Sender;


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;


// Internal Dependencies ------------------------------------------------------
use ::audio::mixer::compress;


// Audio Track Implementation -------------------------------------------------
pub struct Track {
    user_id: Option<UserId>,
    chunk_duration: u32,
    write_queue: Sender<Option<Chunk>>,
    started: u64,
    voice_packets: Vec<VoicePacket>,
    start_timestamp: Option<u32>,
    last_chunk_timestamp: u32
}

impl Track {

    pub fn new(
        started: u64,
        chunk_duration: u32,
        write_queue: Sender<Option<Chunk>>

    ) -> Track {
        Track {
            user_id: None,
            chunk_duration: chunk_duration,
            write_queue: write_queue,
            started: started,
            voice_packets: Vec::new(),
            last_chunk_timestamp: 0,
            start_timestamp: None
        }
    }

    pub fn set_user_id(&mut self, user_id: &UserId) {
        self.user_id = Some(*user_id) ;
    }

    pub fn add_voice_packet(&mut self, packet: VoicePacket) {
        self.voice_packets.push(packet);
        self.write_chunks(1000);
    }

    pub fn flush(&mut self) {
        self.write_chunks(0);
    }

    fn write_chunks(&mut self, split_duration: u32) {
        if self.user_id.is_some() {
            while let Some(chunk) = self.get_chunk(split_duration) {
                self.write_queue.send(Some(chunk)).ok();
            }
        }
    }

    fn get_chunk(&mut self, split_duration: u32) -> Option<Chunk> {

        let min_split_packets = (((split_duration / 20) as f32) * 1.5) as usize;
        if !self.voice_packets.is_empty() && self.voice_packets.len() > min_split_packets {

            // Sort voice packets
            self.voice_packets.sort_by(|a, b| {
                if a.sequence == b.sequence {
                    cmp::Ordering::Equal

                } else if seq_is_more_recent(a.sequence, b.sequence) {
                    cmp::Ordering::Greater

                } else {
                    cmp::Ordering::Less
                }
            });

            // Extract timing from oldest voice packet in chunk
            let (oldest_received, oldest_timestamp) = {
                let oldest = self.voice_packets.first().unwrap();
                (oldest.received, oldest.timestamp)
            };
            assert!(oldest_received > self.started);

            if let Some(start_timestamp) = self.start_timestamp {
                assert!(start_timestamp < oldest_timestamp);

            } else {
                // Convert local u64 receival into remote timestamp value
                self.last_chunk_timestamp = oldest_timestamp - (oldest_received - self.started) as u32;
                self.start_timestamp = Some(oldest_timestamp);
            }

            // Collect voice packets for the requested chunk duration
            let mut last_packet_end = 0;
            let mut chunk_packets = Vec::new();
            for packet in &mut self.voice_packets {

                // Calculate offset from initial packet
                let offset = (packet.timestamp - oldest_timestamp) as u32;

                // Break out once we collected `chunk_duration` packets
                if offset >= self.chunk_duration {
                    break;

                } else {
                    // There tends to be too much silence detected between
                    // adjacents the packets which produces a lot of stutter
                    // in the recording, so we want to "smooth" it out by removing
                    // up to 80ms of additional silence
                    let silence = cmp::max(((offset - last_packet_end) as i32) - 80, 0) as u32;
                    last_packet_end = offset + 20;
                    chunk_packets.push(ChunkPacket {
                        silence: silence,
                        offset: offset,
                        channels: packet.channels,
                        data: packet.data.take().expect("Missing sample data for voice packet.")
                    });
                }

            }

            // Remove consumed voice packets
            self.voice_packets.drain(0..chunk_packets.len()).count();

            // Create chunk from collected voice packets
            let chunk = Chunk {
                user_id: self.user_id.unwrap(),
                silence: cmp::max(((oldest_timestamp - self.last_chunk_timestamp) as i32) - 80, 0) as u32,
                duration: last_packet_end,
                offset: oldest_timestamp - self.start_timestamp.unwrap(),
                packets: chunk_packets
            };

            self.last_chunk_timestamp += chunk.silence + chunk.duration;

            Some(chunk)

        } else {
            None
        }

    }

}

pub struct Chunk {
    pub user_id: UserId,
    pub silence: u32,
    pub duration: u32,
    pub offset: u32,
    pub packets: Vec<ChunkPacket>
}

pub struct ChunkPacket {
    pub silence: u32,
    pub offset: u32,
    pub channels: usize,
    pub data: Vec<i16>
}

impl ChunkPacket {

    pub fn mix_to_mono(&mut self) -> &[i16] {

        let channels = self.channels;
        let mono_samples = self.data.len() / channels;
        let max_sample_value = i16::max_value() as f32;

        for e in 0..mono_samples {
            let i = e * channels;
            if channels == 2 {
                let s = (self.data[i] + self.data[i + 1]) as f32;
                self.data[e] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;

            } else {
                self.data[e] = self.data[i];
            }
        }

        &self.data[0..mono_samples]

    }

}

pub struct VoicePacket {
    pub sequence: u16,
    pub timestamp: u32,
    pub received: u64,
    pub channels: usize,
    pub data: Option<Vec<i16>>
}


// Helpers --------------------------------------------------------------------
const MAX_SEQ_NUMBER: u16 = 65535;

fn seq_is_more_recent(a: u16, b: u16) -> bool {
    (a > b) && (a - b <= MAX_SEQ_NUMBER / 2) ||
    (b > a) && (b - a >  MAX_SEQ_NUMBER / 2)
}

