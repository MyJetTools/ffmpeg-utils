#[derive(Debug, Clone, Copy)]
pub enum AudioCodec {
    None,
    Aac,
    Mp3,
    Opus,
}

impl AudioCodec {
    pub fn to_u8(&self) -> u8 {
        match self {
            AudioCodec::None => 0,
            AudioCodec::Aac => 1,
            AudioCodec::Mp3 => 2,
            AudioCodec::Opus => 3,
        }
    }
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Aac,
            2 => Self::Mp3,
            3 => Self::Opus,
            _ => Self::Opus,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }
}

impl Into<AudioCodec> for ffmpeg_next::codec::Id {
    fn into(self) -> AudioCodec {
        match self {
            ffmpeg_next::codec::Id::AAC => AudioCodec::Aac,
            ffmpeg_next::codec::Id::MP3 => AudioCodec::Mp3,
            ffmpeg_next::codec::Id::OPUS => AudioCodec::Opus,
            _ => AudioCodec::None,
        }
    }
}
