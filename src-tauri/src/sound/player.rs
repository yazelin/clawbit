pub struct SoundPlayer {
    muted: bool,
}

impl SoundPlayer {
    pub fn new() -> Self {
        Self { muted: false }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn play_completion(&self) {
        if self.muted { return; }
        // TODO: Add actual sound playback with rodio once WAV assets are added
    }

    pub fn play_error(&self) {
        if self.muted { return; }
        // TODO: Add actual sound playback with rodio once WAV assets are added
    }
}
