use realsense_rust::{self as rs, frame::FrameEx, kind::Rs2Format};

use crate::core::Mode;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    frame::{ColorFrame, ImageFrame},
    kind::Rs2StreamKind,
    pipeline::ActivePipeline,
};
use std::ffi::{CString, c_void};

pub struct Frame {
    pub timestamp: f64,
    pub data: Vec<u8>,
}

impl Frame {
    pub fn from_rs_frame<T>(frame: &ImageFrame<T>) -> Self {
        Self {
            timestamp: frame.timestamp(),
            data: Frame::frame_as_bytes(frame).to_vec(),
        }
    }

    fn frame_as_bytes<T>(frame: &ImageFrame<T>) -> &[u8] {
        let len = frame.get_data_size();

        unsafe {
            let ptr = frame.get_data() as *const c_void as *const u8;
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

pub struct Camera {
    pipeline: ActivePipeline,
    mode: Mode,
}

impl Camera {
    const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);

    pub fn new(
        serial: &str,
        stream: Rs2StreamKind,
        format: Rs2Format,
        mode: Mode,
        context: &rs::context::Context,
    ) -> Result<Self> {
        let pipeline = rs::pipeline::InactivePipeline::try_from(context)
            .context("Unable to create RealSense pipeline")?;
        let mut config = Config::new();

        let serial = CString::new(serial.to_owned())?;
        config
            .enable_device_from_serial(serial.as_c_str())?
            .disable_all_streams()?
            .enable_stream(
                stream,
                None,
                mode.width,
                mode.height,
                format,
                mode.framerate as usize,
            )?;

        let pipeline = pipeline.start(Some(config))?;

        Ok(Self { pipeline, mode })
    }

    pub fn wait_for_frames(&mut self) -> anyhow::Result<Frame> {
        let frames = self.pipeline.wait(Some(Camera::TIMEOUT))?;

        let color_frames = frames.frames_of_type::<ColorFrame>();
        if color_frames.is_empty() {
            anyhow::bail!("Empty frames received");
        }

        let color_frame = color_frames.first().unwrap();
        Ok(Frame::from_rs_frame(color_frame))
    }

    pub fn width(&self) -> usize {
        self.mode.width
    }

    pub fn height(&self) -> usize {
        self.mode.height
    }

    pub fn framerate(&self) -> i32 {
        self.mode.framerate
    }
}
