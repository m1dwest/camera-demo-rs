use realsense_rust::{
    self as rs,
    frame::{DepthFrame, FrameEx},
    processing_blocks::colorizer::Colorizer,
};

use crate::core::{Mode, PixelFormat};

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    frame::{ColorFrame, ImageFrame},
    kind::Rs2StreamKind,
    pipeline::ActivePipeline,
};
use std::ffi::{CString, c_void};

#[derive(Clone)]
pub struct Frame(Vec<u8>);

pub struct Intrinsics {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub timestamp: f64,
}

impl Frame {
    pub fn from_rs_frame<T>(frame: &ImageFrame<T>) -> Self {
        Self(Frame::frame_as_bytes(frame).to_vec())
    }

    fn frame_as_bytes<T>(frame: &ImageFrame<T>) -> &[u8] {
        let len = frame.get_data_size();

        unsafe {
            let ptr = frame.get_data() as *const c_void as *const u8;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self(vec)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

pub struct Camera {
    pipeline: ActivePipeline,
    mode: Mode,
    format: PixelFormat,
    colorizer: Colorizer,
}

impl Camera {
    const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);

    pub fn new(
        serial: &str,
        stream: Rs2StreamKind,
        format: PixelFormat,
        mode: Mode,
    ) -> Result<Self> {
        let ctx = rs::context::Context::new().context("Failed to create RealSense context")?;
        let pipeline = rs::pipeline::InactivePipeline::try_from(&ctx)
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
                format.into(),
                mode.framerate as usize,
            )?;

        let pipeline = pipeline.start(Some(config))?;

        Ok(Self {
            pipeline,
            mode,
            format,
            colorizer: Colorizer::new(1)?,
        })
    }

    pub fn wait_for_frames(&mut self) -> anyhow::Result<(Frame, Intrinsics)> {
        let frames = self.pipeline.wait(Some(Camera::TIMEOUT))?;

        let color_frames = frames.frames_of_type::<ColorFrame>();
        let depth_frames = frames.frames_of_type::<DepthFrame>();
        let color_frame = if !depth_frames.is_empty() {
            let Some(depth_frame) = depth_frames.into_iter().next() else {
                anyhow::bail!("Unable to receive depth frame");
            };

            self.colorizer.queue(depth_frame);

            match self.colorizer.wait(std::time::Duration::from_millis(100)) {
                Ok(colorized_frame) => colorized_frame,
                Err(e) => {
                    anyhow::bail!("Error processing colorized frame: {}", e);
                }
            }
        } else {
            color_frames.into_iter().next().unwrap()
        };

        let frame = Frame::from_rs_frame(&color_frame);
        let intrinsics = Intrinsics {
            width: self.mode.width,
            height: self.mode.height,
            timestamp: color_frame.timestamp(),
            format: self.format,
        };

        // TODO: send intrinsics on streaming start
        Ok((frame, intrinsics))
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
