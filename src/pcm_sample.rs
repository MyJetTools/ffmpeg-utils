#[derive(Debug, Clone, Copy)]
pub struct PcmSample(f32);

impl PcmSample {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn as_f32_planar(&self) -> f32 {
        self.0
    }

    pub fn as_i16_pcm(&self) -> i16 {
        (self.0.clamp(-1.0, 1.0) * 32767.0) as i16
    }
}

impl Into<PcmSample> for f32 {
    fn into(self) -> PcmSample {
        PcmSample(self)
    }
}
