use realsense_rust as rs;

use anyhow::{Context as _, Result};
use rs::kind::{Rs2CameraInfo, Rs2Format, Rs2ProductLine, Rs2StreamKind};
use std::ffi::CStr;

trait InfoProvider {
    fn into_cstr(&self, key: Rs2CameraInfo) -> Option<&CStr>;
}

impl InfoProvider for rs::device::Device {
    fn into_cstr(&self, key: Rs2CameraInfo) -> Option<&CStr> {
        self.info(key)
    }
}

impl InfoProvider for rs::sensor::Sensor {
    fn into_cstr(&self, key: Rs2CameraInfo) -> Option<&CStr> {
        self.info(key)
    }
}

fn parse_info<T, P>(provider: &P, key: Rs2CameraInfo, info_str: &str) -> Option<T>
where
    P: InfoProvider,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let cstr = provider.into_cstr(key).or_else(|| {
        log::error!("Device does not support {}", info_str);
        None
    })?;

    let s = cstr
        .to_str()
        .inspect_err(|e| log::error!("Device {} is not a valid UTF-8: {}", info_str, e))
        .ok()?;

    s.parse::<T>()
        .inspect_err(|e| log::error!("Failed to parse {} as {}: {}", s, info_str, e))
        .ok()
}

pub struct Mode {
    kind: Rs2StreamKind,
    format: Rs2Format,
    framerate: i32,
    resolution: Option<(usize, usize)>,
}

impl Mode {
    fn from_profile(profile: &rs::stream_profile::StreamProfile) -> Self {
        Self {
            kind: profile.kind(),
            format: profile.format(),
            framerate: profile.framerate(),
            resolution: {
                match profile.intrinsics() {
                    Ok(i) => Some((i.width(), i.height())),
                    Err(_) => None,
                }
            },
        }
    }
}

pub struct Device {
    pub name: Option<String>,
    pub serial: Option<String>,

    pub usb_type: Option<f32>,
    pub sensor_modes: Vec<(Option<String>, Vec<Mode>)>,
}

pub struct RealSenseBackend {
    ctx: rs::context::Context,
}

impl RealSenseBackend {
    pub fn new() -> Result<Self> {
        let ctx = rs::context::Context::new().context("Failed to create RealSense context")?;
        Ok(Self { ctx })
    }

    pub fn context(&self) -> &rs::context::Context {
        &self.ctx
    }

    pub fn devices(&self) -> Vec<Device> {
        let mut query = std::collections::HashSet::new();
        query.insert(Rs2ProductLine::D400);

        let devices = self.ctx.query_devices(query);

        devices
            .into_iter()
            .map(|device| {
                let name =
                    parse_info::<String, _>(&device, Rs2CameraInfo::Name, "Rs2CameraInfo::Name");

                let serial = parse_info::<String, _>(
                    &device,
                    Rs2CameraInfo::SerialNumber,
                    "Rs2CameraInfo::SerialNumber",
                );

                let usb_type = parse_info::<f32, _>(
                    &device,
                    Rs2CameraInfo::UsbTypeDescriptor,
                    "Rs2CameraInfo::UsbTypeDescriptor",
                );

                let sensor_modes = device
                    .sensors()
                    .iter()
                    .map(|sensor| {
                        let name = parse_info::<String, _>(
                            sensor,
                            Rs2CameraInfo::Name,
                            "Rs2CameraInfo::Name",
                        );

                        let modes = sensor
                            .stream_profiles()
                            .iter()
                            .map(|profile| Mode::from_profile(profile))
                            .collect();

                        (name, modes)
                    })
                    .collect();

                Device {
                    name,
                    serial,
                    usb_type,
                    sensor_modes,
                }
            })
            .collect()
    }
}
