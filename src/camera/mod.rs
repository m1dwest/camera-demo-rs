// pub mod camera;
pub mod devices_query;

// pub use camera::Camera;
pub use devices_query::Devices;

use realsense_rust as rs;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    context::Context,
    pipeline::{ActivePipeline, InactivePipeline},
};

pub struct Camera {
    pub devices: Devices,

    context: Context,
    // pipe: ActivePipeline,
}

impl Camera {
    pub fn new() -> Result<Self> {
        let context =
            rs::context::Context::new().context("RealSense context initialization failed")?;
        let devices = Devices::query(&context);

        let pipeline = rs::pipeline::InactivePipeline::try_from(&context)
            .context("Unable to create RealSense pipeline")?;
        let mut config = Config::new();

        Ok(Self { devices, context })
    }
}
