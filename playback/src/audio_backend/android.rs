use super::{Open, Sink, SinkAsBytes, SinkError, SinkResult};
use crate::config::AudioFormat;
use crate::convert::Converter;
use crate::decoder::AudioPacket;
use crate::{NUM_CHANNELS, SAMPLE_RATE};
use std::sync::OnceLock;
use zerocopy::IntoBytes;

pub type PcmCallback =
    extern "C" fn(data: *const u8, len: usize, sample_rate: u32, channels: u8, format: AudioFormat);
// Global PCM callback
static PCM_CALLBACK: OnceLock<PcmCallback> = OnceLock::new();

pub struct AndroidSink {
    format: AudioFormat,
}

impl AndroidSink {
    pub const NAME: &'static str = "android";

    pub fn set_callback(cb: PcmCallback) {
        if PCM_CALLBACK.set(cb).is_err() {
            log::warn!("PCM Callback is already set!");
        }
    }
}

impl Open for AndroidSink {
    fn open(_: Option<String>, format: AudioFormat) -> Self {
        info!("Using AndroidSink with format: {format:?}");
        Self { format }
    }
}

impl Sink for AndroidSink {
    fn start(&mut self) -> SinkResult<()> {
        Ok(())
    }

    fn stop(&mut self) -> SinkResult<()> {
        Ok(())
    }

    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        let bytes: Vec<u8> = match packet {
            AudioPacket::Samples(samples) => match self.format {
                AudioFormat::S16 => converter.f64_to_s16(&samples).as_bytes().to_vec(),
                AudioFormat::F32 => converter.f64_to_f32(&samples).as_bytes().to_vec(),
                AudioFormat::F64 => samples.as_bytes().to_vec(),
                _ => unimplemented!("Other formats not yet implemented"),
            },
            AudioPacket::Raw(data) => data,
        };

        if let Some(cb) = PCM_CALLBACK.get() {
            cb(
                bytes.as_ptr(),
                bytes.len(),
                SAMPLE_RATE,
                NUM_CHANNELS,
                self.format,
            );
            Ok(())
        } else {
            Err(SinkError::NotConnected(
                "PCM callback not registered".into(),
            ))
        }
    }
}

impl SinkAsBytes for AndroidSink {
    fn write_bytes(&mut self, data: &[u8]) -> SinkResult<()> {
        if let Some(cb) = PCM_CALLBACK.get() {
            cb(
                data.as_ptr(),
                data.len(),
                SAMPLE_RATE,
                NUM_CHANNELS,
                self.format,
            );
            Ok(())
        } else {
            Err(SinkError::NotConnected(
                "PCM callback not registered".into(),
            ))
        }
    }
}
