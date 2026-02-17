use realsense_rust as rs;

use anyhow::{Context as _, Result};
use rs::kind::{Rs2CameraInfo, Rs2Format, Rs2ProductLine, Rs2StreamKind};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Hash, Copy, Serialize, Deserialize)]
pub struct Mode {
    pub framerate: i32,
    pub width: usize,
    pub height: usize,
}

pub struct Profile {
    pub mode: Mode,
    pub formats: Vec<Rs2Format>,
}

impl PartialEq for Mode {
    fn eq(&self, other: &Self) -> bool {
        self.framerate == other.framerate
            && self.width == other.width
            && self.height == other.height
    }
}

impl Eq for Mode {}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}, {} Hz", self.width, self.height, self.framerate)
    }
}

pub struct Stream {
    pub kind: Rs2StreamKind,
    pub profiles: Vec<Profile>,
}

pub struct Sensor {
    pub name: String,
    pub streams: Vec<Stream>,
}

impl Stream {
    fn from_sensor(sensor: &rs::sensor::Sensor) -> Vec<Stream> {
        use std::collections::{HashMap, HashSet};

        type ModesMap = HashMap<Mode, HashSet<Rs2Format>>;
        type KindsMap = HashMap<Rs2StreamKind, ModesMap>;

        let mut modes: ModesMap = HashMap::new();
        let mut kinds: KindsMap = HashMap::new();

        for p in &sensor.stream_profiles() {
            let stream = p.kind();

            let framerate = p.framerate();
            let (width, height) = match p.intrinsics() {
                Ok(i) => (i.width(), i.height()),
                Err(_) => {
                    continue;
                }
            };

            let key = Mode {
                framerate,
                width,
                height,
            };

            kinds
                .entry(stream)
                .or_default()
                .entry(key)
                .or_default()
                .insert(p.format());
        }

        kinds
            .into_iter()
            .map(|(kind, modes_map)| {
                let profiles: Vec<_> = modes_map
                    .into_iter()
                    .map(|(mut mode, formats_set)| {
                        let mut formats: Vec<_> = formats_set.into_iter().collect();
                        formats.sort_by_key(|&f| f as i32);

                        Profile { formats, mode }
                    })
                    .collect();

                Stream { kind, profiles }
            })
            .collect()
    }
}

pub struct Device {
    pub name: String,
    pub serial: Option<String>,

    pub usb_type: Option<f32>,
    pub sensors: Vec<Sensor>,
}

pub struct RealSenseBackend {
    ctx: rs::context::Context,
}

impl RealSenseBackend {
    const UNKNOWN_NAME: &str = "Unknown";

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

        let mut result: Vec<_> = devices
            .into_iter()
            .map(|device| {
                let name =
                    parse_info::<String, _>(&device, Rs2CameraInfo::Name, "Rs2CameraInfo::Name")
                        .unwrap_or(Self::UNKNOWN_NAME.to_owned());

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

                let sensors = device
                    .sensors()
                    .iter()
                    .map(|sensor| {
                        let name = parse_info::<String, _>(
                            sensor,
                            Rs2CameraInfo::Name,
                            "Rs2CameraInfo::Name",
                        )
                        .unwrap_or(Self::UNKNOWN_NAME.to_owned());
                        let streams = Stream::from_sensor(sensor);

                        Sensor { name, streams }
                    })
                    .collect();

                Device {
                    name,
                    serial,
                    usb_type,
                    sensors,
                }
            })
            .collect();

        result
    }
}
