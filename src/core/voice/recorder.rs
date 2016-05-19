// STD Dependencies -----------------------------------------------------------
use std;
use std::cmp;
use std::iter;
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use super::util::{compress, seq_is_more_recent};
use super::encoder::OggVorbisEncoder;


// Types ----------------------------------------------------------------------
pub type SamplePacket = (u16, u32, u32, u64, bool, Vec<i16>);
type File = std::io::BufWriter<std::fs::File>;


// Audio Recorder -------------------------------------------------------------
pub struct Recorder {
    last_receive: u64,
    packet_buffer: Vec<SamplePacket>,
    writer: Option<OggVorbisEncoder>,
    period: usize
}

impl Recorder {

    pub fn new() -> Recorder {
        Recorder {
            last_receive: clock_ticks::precise_time_ms(),
            packet_buffer: Vec::new(),
            writer: None,
            period: 0
        }
    }

    pub fn start(&mut self, filename: &str) {
        if let Ok(mut encoder) = OggVorbisEncoder::new(filename) {
            encoder.init_vbr(1, 48000, 0.2);
            self.last_receive = clock_ticks::precise_time_ms();
            self.writer = Some(encoder);
        }
    }

    pub fn receive_packet(&mut self, packet: SamplePacket) {
        self.packet_buffer.push(packet);
    }

    pub fn mix(&mut self) {
        self.mix_buffer(false);
    }

    pub fn stop(&mut self) {
        self.mix_buffer(true);
        self.writer.take().unwrap();
        // TODO we're not closing this out here as currently causes segfaults
        //writer.close();
    }

    fn mix_buffer(&mut self, flush: bool) {

        if let Some(mut writer) = self.writer.as_mut() {

            let len = self.packet_buffer.len();

            // Write periodically to avoid rendering very long periods of silence
            self.period += 1;

            if len >= 32 || flush || self.period > 8 {

                self.period = 0;

                // Sort buffer by timestamp
                self.packet_buffer.sort_by(|a, b| a.1.cmp(&b.1));

                // Get sorted packets to write
                let packets: Vec<_> = if flush {
                    self.packet_buffer.drain(0..len / 2).collect()

                } else {
                    self.packet_buffer.drain(0..len).collect()
                };

                if !packets.is_empty() {
                    self.last_receive = mix_packets(
                        &mut writer, packets, self.last_receive
                    );
                }

            }

        }

    }

}


// Helpers --------------------------------------------------------------------
fn mix_packets(
    mut writer: &mut OggVorbisEncoder,
    packets: Vec<SamplePacket>,
    last_receive: u64

) -> u64 {

    // Split sample packets by source IDs
    let mut min_receive: u64 = u64::max_value();
    let mut max_receive: u64 = 0;

    // Split packets according to their sources
    let mut sources: HashMap<u32, (u32, u64, Vec<SamplePacket>)> = HashMap::new();
    for p in packets.into_iter() {

        let mut source = sources.entry(p.2).or_insert_with(|| {
            (u32::max_value(), u64::max_value(), Vec::new())
        });

        // Calculate overall timings
        min_receive = cmp::min(min_receive, p.3);
        max_receive = cmp::max(max_receive, p.3);

        // Calculate Source specific timings
        source.0 = cmp::min(source.0, p.1);
        source.1 = cmp::min(source.1, p.3);
        source.2.push(p);

    }

    // Calculate silence between two packet slices
    let silence_samples = (cmp::max((min_receive as i64 - last_receive as i64) - 20, 0) * 48) as usize;

    // Sample mixing
    let max_sample_value: f32 = i16::max_value() as f32;
    let mut max_sample_index = 0;

    // Create buffer with initial silence
    let mut buffer: Vec<i16> = iter::repeat(0).take(silence_samples).collect();

    for (_, (min, p_min_receive, mut packets)) in sources.into_iter() {

        // Order source packets
        packets.sort_by(|a, b| {
            if a.0 == b.0 {
                cmp::Ordering::Equal

            } else if seq_is_more_recent(a.0, b.0) {
                cmp::Ordering::Greater

            } else {
                cmp::Ordering::Less
            }
        });

        let mut first_packet = true;
        let mut receive_offset = 0;

        for &(_, timestamp, _, _, _, ref samples) in &packets {

            // Get index into mixing buffer
            let sample_offset = (timestamp - min) as usize;

            // Add time offset base on the actual receive time of the minimal packet
            if first_packet {
                receive_offset = ((p_min_receive - min_receive) as usize) - sample_offset / 48;
                first_packet = false;
            }

            // Calculate final offset into mixing buffer
            let sample_index = silence_samples + sample_offset + receive_offset * 48;

            // Extend mixing buffer if required
            let required_samples = sample_index + samples.len();
            if required_samples > buffer.len() {
                let missing_buffer_samples = required_samples - buffer.len();
                buffer.extend(iter::repeat(0).take(missing_buffer_samples).collect::<Vec<i16>>());
            }

            // Mix samples into buffer
            for i in 0..samples.len() {
                let s = buffer[sample_index + i] as f32 + samples[i] as f32;
                buffer[sample_index + i] = (compress(s / max_sample_value, 0.6) * max_sample_value) as i16;
                max_sample_index = cmp::max(sample_index + i, max_sample_index);
            }

        }

    }

    // Write sample buffer
    //max_sample_index = max_sample_index - (max_sample_index % 8);
    if max_sample_index > 0 {
        writer.write(&buffer[0..max_sample_index]);
    }

    max_receive

}

