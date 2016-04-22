// STD Dependencies -----------------------------------------------------------
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;


// Voice Audio Queue Abstraction ----------------------------------------------
pub enum QueueEntry {
    EffectList(Vec<PathBuf>),
    DelayedEffectList(Vec<PathBuf>, usize),
    SilenceRequest,
    Reset
}

pub type QueueHandle = Sender<QueueEntry>;

pub type Queue = Arc<Mutex<Vec<QueueEntry>>>;

pub struct EmptyQueue;

impl EmptyQueue {
    pub fn new() -> Queue {
        Arc::new(Mutex::new(Vec::new()))
    }
}

