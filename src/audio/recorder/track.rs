// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::cmp;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;


// Internal Dependencies ------------------------------------------------------
use super::{AudioWriter, Chunk, OggWriter, VoicePacket};


// Audio Recording Track Implementation ---------------------------------------
pub struct Track {
    source_id: u32,
    user_id: Option<UserId>,
    offset: u64,
    chunk_duration: u32,
    recording_directory: PathBuf,
    track_file: Option<File>,
    writer: Option<Box<AudioWriter>>,
    voice_packets: Vec<VoicePacket>,
    start_timestamp: u64,
    oldest_packet_timestamp: Option<u32>
}


// Public Interface -----------------------------------------------------------
impl Track {

    pub fn new(
        source_id: u32,
        chunk_duration: u32,
        offset: u64,
        recording_directory: PathBuf

    ) -> Track {
        Track {
            source_id: source_id,
            user_id: None,
            offset: offset,
            chunk_duration: chunk_duration,
            recording_directory: recording_directory,
            track_file: None,
            writer: None,
            voice_packets: Vec::new(),
            start_timestamp: 0,
            oldest_packet_timestamp: None
        }
    }

    pub fn set_user_id(&mut self, user_id: &UserId) {
        if self.user_id.is_none() {
            info!("{} user set", self);
            self.user_id = Some(*user_id)
        }
    }

    pub fn add_voice_packet(
        &mut self,
        sequence: u16,
        timestamp: u32,
        received: u64,
        channels: usize,
        data: &[i16]
    ) {

        let packet = VoicePacket {
            sequence: sequence,
            timestamp: timestamp / 48,
            received: clock_ticks::precise_time_ms(),
            channels: channels,
            data: data.to_vec()
        };

        // Remember the oldest server timestamp for the oldest received packet
        if packet.timestamp <= self.oldest_packet_timestamp.unwrap_or(packet.timestamp) {
            self.oldest_packet_timestamp = Some(packet.timestamp);
            self.start_timestamp = packet.received;
        }

        // Insert packet
        self.voice_packets.push(packet);

        let minimal_chunk_length = (self.chunk_duration as f32 * 1.5) as u32;
        while let Some(chunk) = self.create_chunk(minimal_chunk_length) {
            self.write_chunk(chunk);
        }

    }

    pub fn flush(&mut self) {
        info!("{} Flushing remaining packets...", self);
        while let Some(chunk) = self.create_chunk(0) {
            self.write_chunk(chunk);
        }
    }

}


// Internal Interface ---------------------------------------------------------
impl Track {

    fn create_chunk(&mut self, minimum_duration: u32) -> Option<Chunk> {

        // Make sure we got a user ID for the track file
        if self.user_id.is_none() {
            None

        // Require a minimum amount of voice packets
        } else if self.voice_packets.len() < (minimum_duration / 20) as usize {
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

    fn write_chunk(&mut self, chunk: Chunk) {

        // Append to opened file
        if self.track_file.is_some() {
            if let Some(file) = self.track_file.as_mut() {
                if let Some(writer) = self.writer.as_mut() {
                    writer.write_chunk(file, chunk);
                }
            }

        // Create file if it does not exist yet
        } else if self.user_id.is_some() {

            let mut path = self.recording_directory.clone();
            if let Ok(_) = fs::create_dir_all(path.clone()) {

                path.push(
                    format!("{}.{}.track", self.user_id.unwrap(), self.source_id)
                );

                info!("{} Opening track file \"{:?}\"...", self, path);

                match File::create(path) {
                    Ok(mut file) => {
                        let mut writer = Box::new(OggWriter);
                        writer.write_header(&mut file, self);
                        writer.write_chunk(&mut file, chunk);
                        self.writer = Some(writer);
                        self.track_file = Some(file);
                    },
                    Err(err) => warn!("Failed to open track file: {:?}", err)
                }

            }

        }

    }

}

// Traits ---------------------------------------------------------------------
impl fmt::Display for Track {
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


// Helpers --------------------------------------------------------------------
const MAX_SEQ_NUMBER: u16 = 65535;

pub fn seq_is_more_recent(a: u16, b: u16) -> bool {
    (a > b) && (a - b <= MAX_SEQ_NUMBER / 2) ||
    (b > a) && (b - a >  MAX_SEQ_NUMBER / 2)
}

