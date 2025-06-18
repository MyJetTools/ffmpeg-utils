mod audio_codec;
pub use audio_codec::*;

mod pcm_sample;
pub use pcm_sample::*;

pub mod to_pcm_stream;
pub mod voice_detector;

pub fn init_ffmpeg() {
    if let Err(err) = ffmpeg_next::init() {
        println!("Can not init ffmpeg: {:?}", err);
        panic!("Can not init ffmpeg: {:?}", err)
    }
}
