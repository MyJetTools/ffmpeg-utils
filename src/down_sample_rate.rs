use std::collections::VecDeque;

use crate::PcmSample;

pub struct DownSampleRate {
    down_scale: usize,
    samples: VecDeque<PcmSample>,
}

impl DownSampleRate {
    pub fn new(down_scale: usize) -> Self {
        Self {
            samples: VecDeque::new(),
            down_scale,
        }
    }

    pub fn extend(&mut self, data: Vec<PcmSample>) {
        self.samples.extend(data);
    }

    pub fn extend_from_slice(&mut self, data: &[PcmSample]) {
        for itm in data {
            self.samples.push_back(*itm);
        }
    }

    pub fn next(&mut self) -> Option<PcmSample> {
        if self.samples.len() < self.down_scale {
            return None;
        }

        for _ in 0..self.down_scale - 1 {
            self.samples.pop_front();
        }

        self.samples.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::DownSampleRate;

    #[test]
    fn test_same_rate() {
        let mut down_the_sample = DownSampleRate::new(1);

        down_the_sample.extend_from_slice(&[0.0.into(), 0.1.into(), 0.2.into()]);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.0);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.1);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.2);

        assert!(down_the_sample.next().is_none());
    }

    #[test]
    fn test_down_2x() {
        let mut down_the_sample = DownSampleRate::new(2);
        down_the_sample.extend_from_slice(&[0.0.into(), 0.1.into(), 0.2.into(), 0.3.into()]);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.1);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.3);

        assert!(down_the_sample.next().is_none());
    }

    #[test]
    fn test_down_3x() {
        let mut down_the_sample = DownSampleRate::new(3);

        down_the_sample.extend_from_slice(&[
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
        ]);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.2);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.5);
        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.8);

        assert!(down_the_sample.next().is_none());
    }

    #[test]
    fn test_down_2x_other_cases() {
        let mut down_the_sample = DownSampleRate::new(2);
        down_the_sample.extend_from_slice(&[0.0.into(), 0.1.into(), 0.2.into()]);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.1);

        assert!(down_the_sample.next().is_none());

        down_the_sample.extend_from_slice(&[0.3.into()]);

        assert_eq!(down_the_sample.next().unwrap().as_f32_planar(), 0.3);

        assert!(down_the_sample.next().is_none());
    }
}
