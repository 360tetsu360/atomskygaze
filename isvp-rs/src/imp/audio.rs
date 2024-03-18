use isvp_sys::*;

#[derive(Debug, Clone, Copy)]
pub enum AudioSampleRate {
    AUDIO_SAMPLE_RATE_8000,
    AUDIO_SAMPLE_RATE_16000,
    AUDIO_SAMPLE_RATE_24000,
    AUDIO_SAMPLE_RATE_32000,
    AUDIO_SAMPLE_RATE_44100,
    AUDIO_SAMPLE_RATE_48000,
    AUDIO_SAMPLE_RATE_96000,
}

impl AudioSampleRate {
    pub fn to_raw(&self) -> IMPAudioSampleRate {
        match self {
            Self::AUDIO_SAMPLE_RATE_8000 => IMPAudioSampleRate_AUDIO_SAMPLE_RATE_8000,
            Self::AUDIO_SAMPLE_RATE_16000 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_16000,
            Self::AUDIO_SAMPLE_RATE_24000 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_24000,
            Self::AUDIO_SAMPLE_RATE_32000 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_32000,
            Self::AUDIO_SAMPLE_RATE_44100 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_44100,
            Self::AUDIO_SAMPLE_RATE_48000 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_48000,
            Self::AUDIO_SAMPLE_RATE_96000 =>IMPAudioSampleRate_AUDIO_SAMPLE_RATE_96000,
        }
    }

    pub fn from_raw(raw : IMPAudioSampleRate) -> Self {
        match raw {
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_8000 => Self::AUDIO_SAMPLE_RATE_8000,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_16000 => Self::AUDIO_SAMPLE_RATE_16000,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_24000 => Self::AUDIO_SAMPLE_RATE_24000,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_32000 => Self::AUDIO_SAMPLE_RATE_32000,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_44100 => Self::AUDIO_SAMPLE_RATE_44100,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_48000 => Self::AUDIO_SAMPLE_RATE_48000,
            IMPAudioSampleRate_AUDIO_SAMPLE_RATE_96000 => Self::AUDIO_SAMPLE_RATE_96000,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AudioBitWidth {
    AUDIO_BIT_WIDTH_16,
}

impl AudioBitWidth {
    pub fn to_raw(&self) -> IMPAudioBitWidth {
        match self {
            Self::AUDIO_BIT_WIDTH_16 => IMPAudioBitWidth_AUDIO_BIT_WIDTH_16
        }
    }

    pub fn from_raw(raw: IMPAudioBitWidth) -> Self {
        match raw {
            IMPAudioBitWidth_AUDIO_BIT_WIDTH_16 => Self::AUDIO_BIT_WIDTH_16,
            _ => unreachable!()
        }
    }
}

pub enum AudioSoundMode {
    AUDIO_SOUND_MODE_MONO,
    AUDIO_SOUND_MODE_STEREO,
}

impl AudioSoundMode {
    pub fn to_raw(&self) -> IMPAudioSoundMode {
        match self {
            Self::AUDIO_SOUND_MODE_MONO => IMPAudioSoundMode_AUDIO_SOUND_MODE_MONO,
            Self::AUDIO_SOUND_MODE_STEREO => IMPAudioSoundMode_AUDIO_SOUND_MODE_STEREO
        }
    }

    pub fn from_raw(raw : IMPAudioSoundMode) -> Self {
        match raw {
            IMPAudioSoundMode_AUDIO_SOUND_MODE_MONO => Self::AUDIO_SOUND_MODE_MONO,
            IMPAudioSoundMode_AUDIO_SOUND_MODE_STEREO => Self::AUDIO_SOUND_MODE_STEREO,
            _ => unreachable!()
        }
    }
}

pub enum AudioPayloadType {
    PT_PCM,
    PT_G711A,
    PT_G711U,
    PT_G726,
    PT_AEC,
    PT_ADPCM,
    PT_MAX,
}

impl AudioPayloadType {
    pub fn to_raw(&self) -> IMPAudioPalyloadType {
        match self {
            Self::PT_PCM => IMPAudioPalyloadType_PT_PCM,
            Self::PT_G711A => IMPAudioPalyloadType_PT_G711A,
            Self::PT_G711U => IMPAudioPalyloadType_PT_G711U,
            Self::PT_G726 => IMPAudioPalyloadType_PT_G726,
            Self::PT_AEC => IMPAudioPalyloadType_PT_AEC,
            Self::PT_ADPCM => IMPAudioPalyloadType_PT_ADPCM,
            Self::PT_MAX => IMPAudioPalyloadType_PT_MAX,
        }
    }

    pub fn from_raw(raw: IMPAudioPalyloadType) -> Self {
        match raw {
            IMPAudioPalyloadType_PT_PCM => Self::PT_PCM,
            IMPAudioPalyloadType_PT_G711A => Self::PT_G711A,
            IMPAudioPalyloadType_PT_G711U => Self::PT_G711U,
            IMPAudioPalyloadType_PT_G726 => Self::PT_G726 ,
            IMPAudioPalyloadType_PT_AEC => Self::PT_AEC,
            IMPAudioPalyloadType_PT_ADPCM => Self::PT_ADPCM,
            IMPAudioPalyloadType_PT_MAX => Self::PT_MAX,
            _ => unreachable!()
        }
    }
}

pub enum AudioDecMode {
    ADEC_MODE_PACK,
    ADEC_MODE_STREAM,
}

impl AudioDecMode {
    pub fn to_raw(&self) -> IMPAudioDecMode {
        match self {
            Self::ADEC_MODE_PACK => IMPAudioDecMode_ADEC_MODE_PACK,
            Self::ADEC_MODE_STREAM => IMPAudioDecMode_ADEC_MODE_STREAM,
        }
    }

    pub fn from_raw(raw: IMPAudioDecMode) -> Self {
        match raw {
            IMPAudioDecMode_ADEC_MODE_PACK => Self::ADEC_MODE_PACK,
            IMPAudioDecMode_ADEC_MODE_STREAM => Self::ADEC_MODE_STREAM,
            _ => unreachable!()
        }
    }
}