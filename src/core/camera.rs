use realsense_rust as rs;

use crate::core::Mode;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    context::Context,
    kind::Rs2StreamKind,
    pipeline::{ActivePipeline, InactivePipeline},
};
use std::ffi::{CStr, CString};

pub struct Camera {
    // context: Context,
    // pipe: ActivePipeline,
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

        let mut pipeline = pipeline.start(Some(config))?;

        Ok(Self {})
    }
}
