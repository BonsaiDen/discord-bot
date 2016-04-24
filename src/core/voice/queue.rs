// STD Dependencies -----------------------------------------------------------
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::collections::VecDeque;


// Internal Dependencies ------------------------------------------------------
use super::super::Effect;


// Voice Audio Queue Abstraction ----------------------------------------------
pub enum QueueEntry {
    EffectList(Vec<Effect>, usize),
    QueuedEffectList(Vec<Effect>, usize),
    Reset
}

pub type QueueHandle = Sender<QueueEntry>;

pub type Queue = Arc<Mutex<VecDeque<QueueEntry>>>;

pub struct EmptyQueue;

impl EmptyQueue {
    pub fn create() -> Queue {
        Arc::new(Mutex::new(VecDeque::new()))
    }
}

