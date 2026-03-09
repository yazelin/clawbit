use crate::models::Emotion;
use serde::Serialize;

const DECAY_FACTOR: f64 = 0.92;
const HAPPY_THRESHOLD: f64 = 0.6;
const SAD_THRESHOLD: f64 = 0.45;
const SOB_THRESHOLD: f64 = 0.9;
const FLOOR: f64 = 0.01;

#[derive(Debug, Clone, Serialize)]
pub struct EmotionState {
    pub happy_score: f64,
    pub sad_score: f64,
}

impl EmotionState {
    pub fn new() -> Self { Self { happy_score: 0.0, sad_score: 0.0 } }

    pub fn apply(&mut self, emotion: &str, intensity: f64) {
        match emotion {
            "happy" => self.happy_score = (self.happy_score + intensity).min(1.0),
            "sad" => self.sad_score = (self.sad_score + intensity).min(1.0),
            _ => {}
        }
    }

    pub fn decay(&mut self) {
        self.happy_score *= DECAY_FACTOR;
        self.sad_score *= DECAY_FACTOR;
        if self.happy_score < FLOOR { self.happy_score = 0.0; }
        if self.sad_score < FLOOR { self.sad_score = 0.0; }
    }

    pub fn current_emotion(&self) -> Emotion {
        if self.sad_score >= SOB_THRESHOLD { Emotion::Sob }
        else if self.sad_score >= SAD_THRESHOLD { Emotion::Sad }
        else if self.happy_score >= HAPPY_THRESHOLD { Emotion::Happy }
        else { Emotion::Neutral }
    }
}
