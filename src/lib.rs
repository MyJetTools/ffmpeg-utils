pub mod to_pcm_stream;

pub fn init_ffmpeg() {
    if let Err(err) = ffmpeg_next::init() {
        println!("Can not init ffmpeg: {:?}", err);
        panic!("Can not init ffmpeg: {:?}", err)
    }
}
