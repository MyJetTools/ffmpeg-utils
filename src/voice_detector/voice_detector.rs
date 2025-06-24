use std::collections::VecDeque;

use crate::*;

#[derive(Debug, Clone, Copy)]
pub enum DetectionModel {
    Silence,
    VoiceStarted { silence_detected: Option<usize> },
}

impl Default for DetectionModel {
    fn default() -> Self {
        Self::Silence
    }
}

pub struct VoiceDetector {
    sample_rate: u32,
    samples_amount_20ms: usize,
    current_stream: Vec<PcmSample>,
    pos: usize,
    out_put: VecDeque<Vec<PcmSample>>,
    silence_threshold: f32,
    silence_duration_in_samples: usize,
    mode: DetectionModel,
    prev_chunk: Option<Vec<PcmSample>>,
}

impl VoiceDetector {
    pub fn new(silence_threshold: f32) -> Self {
        Self {
            sample_rate: 0,
            samples_amount_20ms: 360,
            current_stream: vec![],
            out_put: VecDeque::new(),
            pos: 0,
            silence_threshold,
            silence_duration_in_samples: 0,
            mode: Default::default(),
            prev_chunk: None,
        }
    }

    pub fn is_silence(&self) -> bool {
        match self.mode {
            DetectionModel::Silence => true,
            _ => false,
        }
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn create_output(&mut self, to: usize) {
        println!("Creating output: {}", to);

        let extra_silence = self.samples_amount_20ms * 10;

        let chunk = self
            .current_stream
            .drain(..to - self.silence_duration_in_samples + extra_silence);

        let new_item = match self.prev_chunk.take() {
            Some(mut result) => {
                result.extend(chunk);
                result
            }
            None => chunk.collect(),
        };

        self.current_stream
            .drain(..self.silence_duration_in_samples - extra_silence);

        self.out_put.push_back(new_item);
        self.pos = 0;
        self.mode = Default::default();
        self.prev_chunk = None;
    }

    fn delete_first_chunk_of_silence(&mut self, to: usize) {
        let chunk: Vec<_> = self.current_stream.drain(..to).collect();
        self.prev_chunk = Some(chunk);
        self.pos = 0;
        self.mode = Default::default()
    }

    fn detect_silence(&mut self) {
        loop {
            let end_of_chunk = self.pos + self.samples_amount_20ms;

            if end_of_chunk > self.current_stream.len() {
                break;
            }

            let chunk_20_ms = &self.current_stream[self.pos..end_of_chunk];

            let is_silence = is_silence(chunk_20_ms, self.silence_threshold);

            match self.mode {
                DetectionModel::Silence => {
                    if is_silence {
                        self.delete_first_chunk_of_silence(end_of_chunk);
                        continue;
                    } else {
                        println!("Voice detected");
                        self.mode = DetectionModel::VoiceStarted {
                            silence_detected: None,
                        }
                    }
                }
                DetectionModel::VoiceStarted { silence_detected } => match silence_detected {
                    Some(silence_pos) => {
                        if is_silence {
                            if self.pos - silence_pos >= self.silence_duration_in_samples {
                                self.create_output(end_of_chunk);
                                continue;
                            }
                        } else {
                            self.mode = DetectionModel::VoiceStarted {
                                silence_detected: None,
                            }
                        }
                    }
                    None => {
                        if is_silence {
                            self.mode = DetectionModel::VoiceStarted {
                                silence_detected: Some(self.pos),
                            }
                        }
                    }
                },
            }

            self.pos = end_of_chunk;
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        if self.sample_rate == sample_rate {
            return;
        }

        println!("Set SampleRate: {}", sample_rate);

        self.sample_rate = sample_rate;
        self.samples_amount_20ms = (sample_rate as f32 * 0.02) as usize;
        self.silence_duration_in_samples = (sample_rate * 3) as usize;
    }

    pub fn try_get_chunk(&mut self) -> Option<Vec<PcmSample>> {
        self.out_put.pop_back()
    }

    pub fn append_frames(&mut self, samples: &[PcmSample]) {
        self.current_stream.extend_from_slice(samples);
        self.detect_silence();
    }
}

fn is_silence(chunk: &[PcmSample], threshold: f32) -> bool {
    let mut sum = 0.0;

    for itm in chunk {
        let v = itm.as_f32_planar();
        sum += v * v;
    }

    //  println!("Sum: {}", sum);

    sum < threshold
}

#[cfg(test)]
mod test {
    use crate::voice_detector::VoiceDetector;

    #[test]
    fn test() {
        let mut sd = VoiceDetector::new(50.0);

        sd.set_sample_rate(48000);

        assert_eq!(sd.samples_amount_20ms, 960)
    }
}
