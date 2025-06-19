use rust_extensions::{StrOrString, TaskCompletion, UnsafeValue};
use tokio::sync::Mutex;

use crate::{AudioCodec, PcmSample};

use super::ToPcmCommand;

lazy_static::lazy_static! {
     static ref TO_PCM_STREAM_TASKS: Mutex<Option<tokio::sync::mpsc::Sender<ToPcmCommand>>> = {
        Default::default()
    };
}

pub struct BoxedUnsafe {
    codec: UnsafeValue<u8>,
    sample_rate: UnsafeValue<u32>,
}

pub struct ToPcmStream {
    last_sample_number: usize,
    temp_folder: StrOrString<'static>,
    values: Box<BoxedUnsafe>,
}

impl ToPcmStream {
    pub fn new() -> Self {
        Self {
            last_sample_number: 0,
            temp_folder: "/dev/shm".into(),
            values: BoxedUnsafe {
                codec: UnsafeValue::new(0),
                sample_rate: UnsafeValue::new(0),
            }
            .into(),
        }
    }

    pub fn set_temp_folder(mut self, temp_folder: impl Into<StrOrString<'static>>) -> Self {
        let result = rust_extensions::file_utils::format_path(temp_folder);
        let mut result = result.to_string();

        let last = *result.as_bytes().last().unwrap();
        if last == b'/' {
            result.pop();
        }

        self.temp_folder = result.into();
        self
    }

    pub fn get_codec(&self) -> AudioCodec {
        let value = self.values.codec.get_value();
        AudioCodec::from_u8(value)
    }

    pub fn get_sample_rate(&self) -> Option<u32> {
        let value = self.values.sample_rate.get_value();
        if value == 0 {
            return None;
        }

        Some(value)
    }

    pub async fn feed_source_data(&mut self, chunk: &[u8]) -> Vec<PcmSample> {
        let temp_file_name = rust_extensions::SortableId::generate();

        let temp_file_name = format!("{}/{}.mp4", self.temp_folder, temp_file_name);

        tokio::fs::write(temp_file_name.as_str(), chunk)
            .await
            .unwrap();

        let mut task_completing = TaskCompletion::new();

        let awaiter = task_completing.get_awaiter();

        let mut tasks = TO_PCM_STREAM_TASKS.lock().await;

        let tx = match &*tasks {
            Some(tasks_ref) => tasks_ref,
            None => {
                let (tx, rx) = tokio::sync::mpsc::channel(1024);

                tokio::spawn(async move {
                    to_pcm_stream_event_loop(rx).await;
                });

                *tasks = Some(tx);

                tasks.as_ref().unwrap()
            }
        };

        let _ = tx
            .send(ToPcmCommand {
                file: temp_file_name,
                last_sample_data: self.last_sample_number,
                task_completion: task_completing,
            })
            .await;

        let result = awaiter.get_result().await;

        let result = result.unwrap();

        self.last_sample_number = result.last_sample_no;

        if result.codec.is_some() {
            self.values.codec.set_value(result.codec.to_u8());
        }

        if result.sample_rate > 0 {
            self.values.sample_rate.set_value(result.sample_rate);
        }

        result.data
    }
}

async fn to_pcm_stream_event_loop(mut rx: tokio::sync::mpsc::Receiver<ToPcmCommand>) {
    while let Some(mut command) = rx.recv().await {
        let result = execute_data(command.file.as_str(), command.last_sample_data);
        command.task_completion.set_ok(result);
        let _ = tokio::fs::remove_file(command.file).await;
    }
}

impl Default for ToPcmStream {
    fn default() -> Self {
        Self::new()
    }
}

fn execute_data(file_name: &str, last_sample_data: usize) -> FFmpegExecutionResult {
    let mut result = Vec::new();

    let ictx = ffmpeg_next::format::input(file_name);

    if ictx.is_err() {
        return FFmpegExecutionResult {
            data: result,
            last_sample_no: last_sample_data,
            codec: AudioCodec::None,
            sample_rate: 0,
        };
    }

    let mut ictx = ictx.unwrap();

    let audio_stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Audio)
        .ok_or_else(|| ffmpeg_next::Error::StreamNotFound)
        .unwrap();
    let stream_index = audio_stream.index();

    let context =
        ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters()).unwrap();
    let mut decoder = context.decoder().audio().unwrap();

    let codec = match decoder.codec() {
        Some(codec) => codec.id().into(),
        None => AudioCodec::None,
    };

    let sample_rate = decoder.rate();

    //  println!("FFMPEG: Bitrate: {}", decoder.bit_rate());
    //  println!("FFMPEG: SampleRate: {}", decoder.rate());
    //   println!("Codec: {:?}", codec);

    let mut sample_no = 0;

    for (_, packet) in ictx.packets() {
        if packet.stream() == stream_index {
            decoder.send_packet(&packet).unwrap();
            let mut decoded = ffmpeg_next::frame::Audio::empty();

            loop {
                let receive = decoder.receive_frame(&mut decoded);

                if receive.is_err() {
                    break;
                }

                let data: &[f32] = decoded.plane(0);

                for v in data {
                    if sample_no >= last_sample_data {
                        result.push(PcmSample::new(*v));
                    }

                    sample_no += 1;
                }
            }
        }
    }

    FFmpegExecutionResult {
        data: result,
        last_sample_no: sample_no,
        codec,
        sample_rate,
    }
}

pub struct FFmpegExecutionResult {
    pub data: Vec<PcmSample>,
    pub last_sample_no: usize,
    pub codec: crate::AudioCodec,
    pub sample_rate: u32,
}
