use realsense_rust as rs;

use anyhow::{Context as _, Result};
use realsense_rust::{
    config::Config,
    context::Context,
    pipeline::{ActivePipeline, InactivePipeline},
};

pub struct Camera {
    // context: Context,
    // pipe: ActivePipeline,
}

impl Camera {
    pub fn new(
        sn: &str,
        // mode: &crate::core::devices::Mode,
        context: &rs::context::Context,
    ) -> Result<Self> {
        // TODO:
        log::info!("camera created {}", sn);
        let pipeline = rs::pipeline::InactivePipeline::try_from(context)
            .context("Unable to create RealSense pipeline")?;
        let mut config = Config::new();

        Ok(Self {})
    }
}
