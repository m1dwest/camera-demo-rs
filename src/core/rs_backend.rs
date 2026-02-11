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

pub struct Capabilities(Vec<(Rs2StreamKind, Vec<Mode>)>);

impl Capabilities {
    fn from_sensor(sensor: &rs::sensor::Sensor) -> Capabilities {
        use std::collections::HashMap;

        let mut groups: HashMap<Rs2StreamKind, Vec<Mode>> = HashMap::new();

        for p in &sensor.stream_profiles() {
            let stream = p.kind();
            let cap = Mode::from_profile(p);
            groups.entry(stream).or_default().push(cap);
        }

        Capabilities(groups.into_iter().collect())
    }

    pub fn get_streams(&self) -> Vec<Rs2StreamKind> {
        self.0.iter().map(|(k, _)| *k).collect()
    }

    pub fn get_modes_for(&self, stream: Rs2StreamKind) -> Option<&Vec<Mode>> {
        self.0
            .iter()
            .find_map(|(k, m)| if *k == stream { Some(m) } else { None })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mode {
    pub format: Rs2Format,
    pub framerate: i32,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

impl Mode {
    fn from_profile(profile: &rs::stream_profile::StreamProfile) -> Mode {
        let (width, height) = match profile.intrinsics() {
            Ok(i) => (Some(i.width()), Some(i.height())),
            Err(_) => (None, None),
        };

        return Self {
            format: profile.format(),
            framerate: profile.framerate(),
            width,
            height,
        };
    }
}

pub struct Device {
    pub name: String,
    pub serial: Option<String>,

    pub usb_type: Option<f32>,
    pub capabilities: Vec<(String, Capabilities)>,
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

                let capabilities = device
                    .sensors()
                    .iter()
                    .map(|sensor| {
                        let name = parse_info::<String, _>(
                            sensor,
                            Rs2CameraInfo::Name,
                            "Rs2CameraInfo::Name",
                        )
                        .unwrap_or(Self::UNKNOWN_NAME.to_owned());
                        let cap = Capabilities::from_sensor(sensor);

                        (name, cap)
                    })
                    .collect();

                Device {
                    name,
                    serial,
                    usb_type,
                    capabilities,
                }
            })
            .collect();

        result.push(Device {
            name: "Some device name".to_owned(),
            serial: None,
            usb_type: None,
            capabilities: Vec::new(),
        });

        result.push(Device {
            name: "Some valid dev".to_owned(),
            serial: Some("ssserial".to_owned()),
            usb_type: None,
            capabilities: Vec::new(),
        });

        result
    }
}
