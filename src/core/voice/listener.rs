// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::env;
use std::thread;
use std::path::PathBuf;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};


// Discord Dependencies -------------------------------------------------------
use discord::voice::AudioReceiver;
use discord::model::UserId;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use super::queue::{Queue, QueueEntry, QueueHandle};
use super::recorder::{Recorder, SamplePacket};
use super::super::effect::Effect;


// Types ----------------------------------------------------------------------
pub type ListenerCount = Arc<AtomicUsize>;

pub struct EmptyListenerCount;

impl EmptyListenerCount {
    pub fn create() -> ListenerCount {
        Arc::new(AtomicUsize::new(0))
    }
}

pub type RecordingState = Arc<RwLock<Option<String>>>;

pub struct DefaultRecordingState;

impl DefaultRecordingState {
    pub fn create() -> RecordingState {
        Arc::new(RwLock::new(None))
    }
}


// Voice Audio Listener -------------------------------------------------------
pub struct Listener {
    timer_thread: Option<thread::JoinHandle<()>>,
    recording_state: RecordingState,
    queue_handle: Option<QueueHandle>,
    peak_sender: Sender<Option<u32>>,
    record_sender: Sender<SamplePacket>,
    silence_threshold: u32
}

impl Listener {

    pub fn new(
        audio_queue: Queue,
        listener_count: ListenerCount,
        recording_state: RecordingState

    ) -> Listener {

        let listener_recording_state = recording_state.clone();
        let (record_sender, record_receive) = channel::<(SamplePacket)>();
        let (peak_sender, peak_receive) = channel::<(Option<u32>)>();
        let (status_sender, status_receive) = channel::<(QueueEntry)>();

        let timer = thread::spawn(move || {
            listen(
                audio_queue,
                listener_count,
                recording_state,
                peak_receive,
                status_receive,
                record_receive
            );
        });

        Listener {
            timer_thread: Some(timer),
            recording_state: listener_recording_state,
            queue_handle: Some(status_sender),
            peak_sender: peak_sender,
            record_sender: record_sender,
            silence_threshold: 0,
        }

    }

    pub fn take_handle(&mut self) -> Option<QueueHandle> {
        self.queue_handle.take()
    }

}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(timer_thread) = self.timer_thread.take() {
            self.peak_sender.send(None).ok();
            timer_thread.join().unwrap();
        }
    }
}


// Traits ---------------------------------------------------------------------
impl AudioReceiver for Listener {

    fn speaking_update(&mut self, _: u32, _: &UserId, _: bool) {}

    fn voice_packet(&mut self, ssrc: u32, seq: u16, timestamp: u32, is_stereo: bool, data: &[i16]) {

        let peak = (*data.iter().max_by_key(|s| (**s as i32).abs()).unwrap_or(&0) as i32).abs() as u32;
        if peak > self.silence_threshold * 2 {
            self.peak_sender.send(Some(peak)).ok();
        }

        if self.recording_state.read().unwrap().is_some() {
            let ms = clock_ticks::precise_time_ms();
            self.record_sender.send((
                seq,
                timestamp,
                ssrc,
                ((ms + 10) / 20) * 20,
                is_stereo,
                data.to_vec()

            )).ok();
        }

        // TODO fix threshold calculation
        self.silence_threshold = cmp::max(
            (self.silence_threshold + peak) / 2,
            2000
        );

    }

}


// Audio Listening ------------------------------------------------------------
fn listen(
    mut audio_queue: Queue,
    listener_count: ListenerCount,
    recording_state: RecordingState,
    peak_receive: Receiver<Option<u32>>,
    status_receive: Receiver<QueueEntry>,
    record_receive: Receiver<SamplePacket>
) {

    let recording_started = PathBuf::from(env::var("RECORDING_STARTED_EFFECT").unwrap_or("".into()));
    let recording_stopped = PathBuf::from(env::var("RECORDING_STOPPED_EFFECT").unwrap_or("".into()));
    let recording_limit = PathBuf::from(env::var("RECORDING_LIMIT_EFFECT").unwrap_or("".into()));
    let recording_directory = PathBuf::from(env::var("RECORDING_DIRECTORY").unwrap_or("".into()));

    let delay = Duration::from_millis(100);
    let mut silent_for_seconds: usize = 0;

    let mut is_recording = false;
    let mut recorder = Recorder::new(0);

    'outer: loop {

        // Check number of active users in channel
        let active_listeners = listener_count.load(Ordering::Relaxed);
        if active_listeners > 2 {
            silent_for_seconds += 1;

        } else {
            silent_for_seconds = 0;
        }

        // Sample Peaks
        while let Ok(option) = peak_receive.try_recv() {
            if let Some(_) = option {
                silent_for_seconds = 0;

            } else {
                break 'outer;
            }
        }

        // Status Commands
        while let Ok(status) = status_receive.try_recv() {
            if let QueueEntry::Reset = status {
                silent_for_seconds = 0;
            }
        }

        // Recording
        if let Some(ref filename) = *recording_state.read().unwrap() {

            // Start recording when value toggles
            if !is_recording {

                let mut file = recording_directory.clone();
                file.push(filename.clone());

                if recorder.start(file.to_str().unwrap()) {

                    info!("[Listener] Recording started ({}).", filename);

                    play_effect(&mut audio_queue, recording_started.clone());

                    let mut skipped = 0;
                    while let Ok(_) = record_receive.try_recv() {
                        skipped += 1;
                    }

                    info!("[Listener] Skipped {} previous packets.", skipped);
                    is_recording = true;

                } else {
                    info!("[Listener] Recording failed ({}).", filename);
                    let mut w = recording_state.write().unwrap();
                    is_recording = false;
                    *w = None;
                }

            }

            // Record samples
            if is_recording {

                while let Ok(packet) = record_receive.try_recv() {
                    recorder.receive_packet(packet);
                }

                // Mix and check for file size limit
                if recorder.mix() == false {
                    play_effect(&mut audio_queue, recording_limit.clone());
                    info!("[Listener] Recording stopped, filesize limit reached ({}).", filename);
                    is_recording = false;
                }

            }

        // Recording was stopped
        } else if is_recording {

            info!("[Listener] Recording stopped.");
            play_effect(&mut audio_queue, recording_stopped.clone());

            while let Ok(packet) = record_receive.try_recv() {
                recorder.receive_packet(packet);
            }

            recorder.mix();
            recorder.stop();

            is_recording = false

        }

        // Silence Detection
        // TODO fix silence detection
        if silent_for_seconds > 60 {
            info!("[Listener] Silence for 60 seconds detected.");
            silent_for_seconds = 0;
        }

        thread::sleep(delay);

    }

}

fn play_effect(audio_queue: &mut Queue, path: PathBuf) {
    if let Ok(mut queue) = audio_queue.lock() {
        queue.push_front(QueueEntry::EffectList(vec![
            Effect::new("effect_name".to_string(), path)

        ], 0));
    }
}

