use realsense_rust as rs;

use crate::core::Mode;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    context::Context,
    frame::{ColorFrame, CompositeFrame},
    kind::Rs2StreamKind,
    pipeline::{ActivePipeline, FrameWaitError},
};
use std::ffi::{CStr, CString};

pub struct Camera {
    // context: Context,
    pipeline: ActivePipeline,
}

impl Camera {
    pub fn new(
        serial: &str,
        stream: Rs2StreamKind,
        mode: Mode,
        context: &rs::context::Context,
    ) -> Result<Self> {
        // TODO:
        log::info!("Camera created: {}", serial);
        let pipeline = rs::pipeline::InactivePipeline::try_from(context)
            .context("Unable to create RealSense pipeline")?;
        let mut config = Config::new();

        // TODO: Rs2Format
        let serial = CString::new(serial.to_owned())?;
        config
            .enable_device_from_serial(serial.as_c_str())?
            .disable_all_streams()?
            .enable_stream(
                stream,
                None,
                mode.width,
                mode.height,
                realsense_rust::kind::Rs2Format::Rgb8,
                mode.framerate as usize,
            )?;

        let pipeline = pipeline.start(Some(config))?;

        Ok(Self { pipeline })
    }

    pub fn wait_for_wrames(&mut self) {
        let timeout = std::time::Duration::from_millis(1000);
        let result = self.pipeline.wait(Some(timeout));

        // TODO: return result
        let Ok(frames) = result else {
            return;
        };

        let color = frames.frames_of_type::<ColorFrame>();

        if color.is_empty() {
            // no frames
        }

        log::info!("frames count: {}", color.len());
    }
}
