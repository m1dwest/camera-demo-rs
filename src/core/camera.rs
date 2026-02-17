use realsense_rust::{self as rs, frame::FrameEx};

use crate::core::Mode;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    context::Context,
    frame::{ColorFrame, ImageFrame},
    kind::Rs2StreamKind,
    pipeline::{ActivePipeline, FrameWaitError},
};
use std::ffi::{CStr, CString, c_void};

unsafe fn frame_as_bytes<T>(frame: &ImageFrame<T>) -> &[u8] {
    let len = frame.get_data_size();
    let ptr = frame.get_data() as *const c_void as *const u8;
    std::slice::from_raw_parts(ptr, len)
}

pub struct Camera {
    // context: Context,
    pipeline: ActivePipeline,
    mode: Mode,
    stream: Rs2StreamKind,
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

        Ok(Self {
            pipeline,
            mode,
            stream,
        })
    }

    // TODO: mut?
    pub fn wait_for_frames(&mut self) -> anyhow::Result<Vec<u8>> {
        let timeout = std::time::Duration::from_millis(1000);
        let frames = self.pipeline.wait(Some(timeout))?;

        let color_frames = frames.frames_of_type::<ColorFrame>();

        if color_frames.is_empty() {
            // no frames
        }

        let color_frame = color_frames.first().unwrap();
        let data_size = color_frame.get_data_size();
        // TODO: from format
        // if data_size_expected != color_frame.get_data_size {
        //     anyhow::bail!("Unexpected color frame data size");
        // }

        // TODO: optimize allocation
        unsafe { Ok(frame_as_bytes(color_frame).to_vec()) }

        // log::info!("frames count: {}", color_frame.len());
    }

    pub fn width(&self) -> usize {
        self.mode.width
    }

    pub fn height(&self) -> usize {
        self.mode.height
    }
}
