use sdl2::audio::AudioCallback;
use std::f32::consts::PI;

pub struct SineWave {
    pub phase_inc: f32,
    pub phase: f32,
    pub volume: f32,
}

impl AudioCallback for SineWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a sine wave
        for x in out.iter_mut() {
            *x = self.volume * (self.phase * 2.0 * PI).sin();
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
