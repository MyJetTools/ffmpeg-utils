use rust_extensions::{StrOrString, TaskCompletion};
use tokio::sync::Mutex;

use super::{ToPcmCommand, pcm_sample::PcmSample};

lazy_static::lazy_static! {
     static ref TO_PCM_STREAM_TASKS: Mutex<Option<tokio::sync::mpsc::Sender<ToPcmCommand>>> = {
        Default::default()
    };
}

pub struct ToPcmStream {
    last_sample_number: usize,
    temp_folder: StrOrString<'static>,
}

impl ToPcmStream {
    pub fn new() -> Self {
        Self {
            last_sample_number: 0,
            temp_folder: "/dev/shm".into(),
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

fn execute_data(file_name: &str, last_sample_data: usize) -> FFmpegExecutionResult {
    let mut result = Vec::new();

    let ictx = ffmpeg_next::format::input(file_name);

    if ictx.is_err() {
        return FFmpegExecutionResult {
            data: result,
            last_sample_no: last_sample_data,
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
    println!("Bitrate: {}", decoder.bit_rate());
    println!("SampleRate: {}", decoder.rate());

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
    }
}

pub struct FFmpegExecutionResult {
    pub data: Vec<PcmSample>,
    pub last_sample_no: usize,
}
