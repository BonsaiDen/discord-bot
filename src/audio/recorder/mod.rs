// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::iter;
use std::thread;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};


// External Dependencies ------------------------------------------------------
use chrono;
use clock_ticks;
use vorbis_enc::OggVorbisEncoder;


// Modules --------------------------------------------------------------------
mod track;
pub use self::track::{Track, Chunk, VoicePacket};


// Discord Dependencies -------------------------------------------------------
use discord::model::UserId;
use discord::voice::AudioReceiver;


// Audio Recorder Abstraction -------------------------------------------------
pub struct Recorder {
    tracks: HashMap<u32, Track>,
    started: u64,
    chunk_duration: u32,
    write_queue: Sender<Option<Chunk>>,
    writer_thread: Option<thread::JoinHandle<()>>
}


// Public Interface -----------------------------------------------------------
impl Recorder {

    pub fn new(mut recording_path: PathBuf, chunk_duration: u32) -> Recorder {

        let (sender, receiver) = channel::<Option<Chunk>>();
        recording_path.push(format!("{}", chrono::Local::now()));

        Recorder {
            tracks: HashMap::new(),
            started: clock_ticks::precise_time_ms(),
            chunk_duration: chunk_duration,
            write_queue: sender,
            writer_thread: Some(Recorder::writer_thread(recording_path, receiver))
        }

    }

}

// Internal Interface ---------------------------------------------------------
impl Recorder {

    fn writer_thread(path: PathBuf, write_queue: Receiver<Option<Chunk>>) -> thread::JoinHandle<()> {

        thread::spawn(move || {

            fs::create_dir_all(path.clone()).expect("[AudioWiter] Failed to create recording directory.");

            info!("[AudioWiter] Created");

            let silence_buffer: Vec<i16> = iter::repeat(0).take(48000).collect();
            let mut streams: HashMap<UserId, OggVorbisEncoder> = HashMap::new();

            fn write_silence(
                stream: &mut OggVorbisEncoder,
                silence_buffer: &[i16],
                silence_millis: usize
            ) {

                let silence_samples = silence_millis * 48;
                let buffers = silence_samples / 48000;
                let remainder = silence_samples % 48000;

                for _ in 0..buffers {
                    stream.write_samples(silence_buffer).ok();
                }

                if remainder > 0 {
                    stream.write_samples(&silence_buffer[0..remainder]).ok();
                }

            }

            while let Ok(Some(chunk)) = write_queue.recv() {

                let mut stream = streams.entry(chunk.user_id).or_insert_with(|| {

                    let mut file_path = path.clone();
                    file_path.push(format!("{}.ogg", chunk.user_id));

                    let filename = file_path.to_str().unwrap();

                    info!("[AudioWriter] Creating file \"{}\"...", filename);

                    let mut encoder = OggVorbisEncoder::new(filename).expect("[AudioWiter] Failed to open ogg file for recording.");
                    encoder.initialize_with_vbr(1, 48000, 0.2).expect("[AudioWiter] Failed to initialize vorbis stream.");
                    encoder

                });

                info!(
                    "[AudioWriter] Received chunk for User#{} with {} packets @ {}ms with {}ms ({}ms silence infront)",
                    chunk.user_id,
                    chunk.packets.len(),
                    chunk.offset,
                    chunk.duration,
                    chunk.silence
                );

                if chunk.silence > 0 {
                    write_silence(&mut stream, &silence_buffer, chunk.silence as usize);
                }

                for mut packet in chunk.packets {
                    if packet.silence > 0 {
                        write_silence(&mut stream, &silence_buffer, packet.silence as usize);
                    }
                    stream.write_samples(&packet.mix_to_mono()).ok();
                }

            }

            info!("[AudioWriter] Closing streams...");
            for stream in streams.values_mut() {
                stream.close().expect("[AudioWriter] Failed to close vorbis stream.");
            }

            info!("[AudioWriter] Destroyed");

        })
    }

    fn get_track(&mut self, source_id: u32) -> &mut Track {
        let started = self.started;
        let chunk_duration = self.chunk_duration;
        let write_queue = self.write_queue.clone();
        self.tracks.entry(source_id).or_insert_with(|| {
            Track::new(
                started,
                chunk_duration,
                write_queue
            )
        })
    }

    fn flush(&mut self) {
        for track in self.tracks.values_mut() {
            track.flush();
        }
        self.write_queue.send(None).ok();
        self.writer_thread.take().unwrap().join().ok();
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
        let remainder = timestamp % 960;
        self.get_track(source_id).add_voice_packet(VoicePacket {
            sequence: sequence,
            timestamp: (timestamp - remainder) / 48,
            received: clock_ticks::precise_time_ms(),
            channels: if stereo { 2 } else { 1 },
            data: Some(data.to_vec())
        });
    }

}

