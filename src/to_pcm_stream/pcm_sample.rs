#[derive(Debug, Clone, Copy)]
pub struct PcmSample(f32);

impl PcmSample {
    pub fn new(value: f32) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> f32 {
        self.0
    }

    pub fn to_i16(&self) -> i16 {
        (self.0.clamp(-1.0, 1.0) * 32767.0) as i16
    }
}
