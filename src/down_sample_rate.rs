use crate::PcmSample;

pub struct DownSampleRate {
    down_scale: usize,
    samples: Vec<PcmSample>,
    pos: usize,
}

impl DownSampleRate {
    pub fn new(samples: Vec<PcmSample>, down_scale: usize) -> Self {
        Self {
            samples,
            pos: 0,
            down_scale,
        }
    }
}

impl Iterator for DownSampleRate {
    type Item = PcmSample;

    fn next(&mut self) -> Option<Self::Item> {
        for _ in 0..self.down_scale - 1 {
            if self.pos >= self.samples.len() {
                return None;
            }

            self.pos += 1;
        }

        if self.pos >= self.samples.len() {
            return None;
        }

        let result = self.samples[self.pos];
        self.pos += 1;

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DownSampleRate, PcmSample};

    #[test]
    fn test_same_rate() {
        let samples: Vec<PcmSample> = vec![0.0.into(), 0.1.into(), 0.2.into()];

        let mut down_the_sample = DownSampleRate::new(samples, 1);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.0);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.1);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.2);

        assert!(down_the_sample.next().is_none());
    }

    #[test]
    fn test_down_2x() {
        let samples: Vec<PcmSample> = vec![0.0.into(), 0.1.into(), 0.2.into(), 0.3.into()];

        let mut down_the_sample = DownSampleRate::new(samples, 2);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.1);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.3);

        assert!(down_the_sample.next().is_none());
    }

    #[test]
    fn test_down_3x() {
        let samples: Vec<PcmSample> = vec![
            0.0.into(),
            0.1.into(),
            0.2.into(),
            0.3.into(),
            0.4.into(),
            0.5.into(),
            0.6.into(),
            0.7.into(),
            0.8.into(),
            0.9.into(),
        ];

        let mut down_the_sample = DownSampleRate::new(samples, 3);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.2);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.5);
        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.8);

        assert!(down_the_sample.next().is_none());
    }
}
