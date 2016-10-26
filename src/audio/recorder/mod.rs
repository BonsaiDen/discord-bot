// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::fmt;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;
use discord::voice::AudioReceiver;

struct VoicePacket {
    sequence: u16,
    timestamp: u32,
    received: u64,
    channels: usize,
    data: Vec<i16>
}

struct Chunk {
    duration: u32,
    track_offset: u64,
    channels: usize,
    voice_packets: Vec<VoicePacket>
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

struct RecorderTrack {
    source_id: u32,
    user_id: Option<UserId>,
    chunk_duration: u32,
    voice_packets: Vec<VoicePacket>,
    start_timestamp: u64,
    oldest_packet_timestamp: Option<u32>
}

impl RecorderTrack {

    fn new(source_id: u32, chunk_duration: u32) -> RecorderTrack {
        RecorderTrack {
            source_id: source_id,
            user_id: None,
            chunk_duration: chunk_duration,
            voice_packets: Vec::new(),
            start_timestamp: 0,
            oldest_packet_timestamp: None
        }
    }

    fn set_user_id(&mut self, user_id: &UserId) {
        if self.user_id.is_none() {
            info!("{} user set", self);
            self.user_id = Some(*user_id)
        }
    }

    fn add_voice_packet(&mut self, packet: VoicePacket) {

        // Remember the oldest server timestamp for the oldest received packet
        if packet.timestamp <= self.oldest_packet_timestamp.unwrap_or(packet.timestamp) {
            self.oldest_packet_timestamp = Some(packet.timestamp);
            self.start_timestamp = packet.received;
        }

        // Insert packet
        self.voice_packets.push(packet);

        let minimal_chunk_length = (self.chunk_duration as f32 * 1.5) as u32;
        while let Some(_) = self.create_chunk(minimal_chunk_length) {
            // TODO write out?
        }

    }

    fn flush(&mut self) {
        info!("{} Flushing remaining packets...", self);
        while let Some(_) = self.create_chunk(0) {
            // TODO write out?
        }
    }

    fn create_chunk(&mut self, minimum_duration: u32) -> Option<Chunk> {

        if self.voice_packets.len() < (minimum_duration / 20) as usize {
            None

        // Also check we have at least one voice packet this simplifies the
        // logic below
        } else if self.voice_packets.is_empty() {
            None

        } else {

            // Now sort all voice packets by their sequence number
            // Each sequence increase corresponds to 20ms so there are about 21
            // minutes within the 16bit sequence number space.
            // Which is the reason why we should not run into any sorting issues
            // here.
            self.voice_packets.sort_by(|a, b| {
                if a.sequence == b.sequence {
                    cmp::Ordering::Equal

                } else if seq_is_more_recent(a.sequence, b.sequence) {
                    cmp::Ordering::Greater

                } else {
                    cmp::Ordering::Less
                }
            });

            // Get timestamp of the first packet, we'll use it to calculate the
            // actual timestamp offsets for the rest of the packets
            let packet_channels = self.voice_packets.first().unwrap().channels;
            let oldest_packet_timestamp = self.voice_packets.first().unwrap().timestamp;
            let oldest_packet_received = self.voice_packets.first().unwrap().received;

            let mut chunk_duration = 0;
            let mut chunk_packet_count = 0;

            for packet in &mut self.voice_packets {

                // Calculate offset from initial packet
                let offset = (packet.timestamp - oldest_packet_timestamp) as u32;

                // Break out once we collected `chunk_duration` packets
                if offset >= self.chunk_duration {
                    break;

                // Otherwise append packet to the chunk and correct packet timestamp
                // with the `real timestamp` and the `offset` form the first packet
                } else {
                    chunk_duration = offset + 20; // Use the end of the packet
                    packet.timestamp = oldest_packet_timestamp + offset;
                    chunk_packet_count += 1;
                }

            }

            // Create a new chunk
            let chunk = Chunk {
                track_offset: oldest_packet_received - self.start_timestamp,
                duration: chunk_duration,
                channels: packet_channels,
                voice_packets: self.voice_packets.drain(0..chunk_packet_count).collect()
            };

            info!("{} Created chunk {}", self, chunk);

            Some(chunk)

        }

    }

}


impl fmt::Display for RecorderTrack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(user_id) = self.user_id {
            write!(
                f,
                "[Track for User#{}({}) {} packets]",
                user_id,
                self.source_id,
                self.voice_packets.len()
            )

        } else {
            write!(
                f,
                "[Track for ? ({}) {} packets]",
                self.source_id,
                self.voice_packets.len()
            )
        }
    }
}

// Audio Recorder Abstraction -------------------------------------------------
pub struct Recorder {
    tracks: HashMap<u32, RecorderTrack>,
    chunk_duration: u32
}


// Public Interface -----------------------------------------------------------
impl Recorder {

    pub fn new(chunk_duration: u32) -> Recorder {
        Recorder {
            tracks: HashMap::new(),
            chunk_duration: chunk_duration
        }
    }

}

// Internal Interface ---------------------------------------------------------
impl Recorder {

    fn get_track(&mut self, source_id: u32) -> &mut RecorderTrack {
        let chunk_duration = self.chunk_duration;
        self.tracks.entry(source_id).or_insert_with(|| {
            RecorderTrack::new(source_id, chunk_duration)
        })
    }

    fn flush(&mut self) {
        for track in self.tracks.values_mut() {
            track.flush();
        }
    }

}


// Traits ---------------------------------------------------------------------
impl Drop for Recorder {
    fn drop(&mut self) {
        self.flush();
    }
}


// Recorder Receiver Implementation -------------------------------------------
impl AudioReceiver for Recorder {

    fn speaking_update(&mut self, source_id: u32, user_id: &UserId, _: bool) {
        self.get_track(source_id).set_user_id(user_id);
    }

    fn voice_packet(
        &mut self,
        source_id: u32,
        sequence: u16,
        timestamp: u32,
        stereo: bool,
        data: &[i16]
    ) {
        self.get_track(source_id).add_voice_packet(VoicePacket {
            sequence: sequence,
            timestamp: timestamp / 48,
            received: clock_ticks::precise_time_ms(),
            channels: if stereo {
                2
            } else {
                1
            },
            data: data.to_vec()
        });
    }

}


// Helpers --------------------------------------------------------------------
const MAX_SEQ_NUMBER: u16 = 65535;

pub fn seq_is_more_recent(a: u16, b: u16) -> bool {
    (a > b) && (a - b <= MAX_SEQ_NUMBER / 2) ||
    (b > a) && (b - a >  MAX_SEQ_NUMBER / 2)
}

