use crate::core::{Device, Mode, Sensor, Stream};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use realsense_rust::kind::{Rs2Format, Rs2StreamKind};

#[derive(Default)]
pub struct DevicesModel {
    pub devices: Vec<Device>,

    sel_device: Option<String>,
    sel_device_name: Option<String>,

    sensors: Vec<String>,
    sel_sensor: Option<String>,

    streams: Vec<Rs2StreamKind>,
    sel_stream: Option<Rs2StreamKind>,

    modes: Vec<Mode>,
    sel_mode: Option<Mode>,

    formats: Vec<PixelFormat>,
    sel_format: Option<PixelFormat>,
}

#[repr(i32)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb8 = Rs2Format::Rgb8 as i32,
}

pub struct UnsupportedPixelFormat;

impl TryFrom<Rs2Format> for PixelFormat {
    type Error = UnsupportedPixelFormat;

    fn try_from(format: Rs2Format) -> Result<Self, Self::Error> {
        match format {
            Rs2Format::Rgb8 => Ok(PixelFormat::Rgb8),
            _ => Err(UnsupportedPixelFormat),
        }
    }
}

impl From<PixelFormat> for Rs2Format {
    fn from(val: PixelFormat) -> Self {
        match val {
            PixelFormat::Rgb8 => Rs2Format::Rgb8,
        }
    }
}

impl TryFrom<i32> for PixelFormat {
    type Error = UnsupportedPixelFormat;

    fn try_from(format: i32) -> Result<Self, Self::Error> {
        use num_traits::FromPrimitive;

        match Rs2Format::from_i32(format) {
            Some(f) => PixelFormat::try_from(f),
            None => Err(UnsupportedPixelFormat),
        }
    }
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let string = match self {
            PixelFormat::Rgb8 => "RGB8",
        };
        write!(f, "{}", string)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub sel_device: Option<String>,
    pub sel_sensor: Option<String>,
    pub sel_stream: Option<i32>,
    pub sel_mode: Option<Mode>,
    pub sel_format: Option<i32>,
}

impl DevicesModel {
    pub fn from_devices(devices: Vec<Device>, current: Option<Self>) -> Self {
        if let Some(current) = current
            && let Some(curr_serial) = &current.sel_device
        {
            let serial_found = devices
                .iter()
                .any(|d| d.serial.as_ref() == Some(curr_serial));
            if serial_found {
                return Self { devices, ..current };
            }
        }

        let selected_device = devices.iter().find(|d| d.serial.is_some());
        let sel_device = selected_device.and_then(|d| d.serial.clone());
        let sel_device_name = selected_device.map(|d| d.name.clone());
        let sensors = selected_device
            .map(|d| {
                let s: Vec<_> = d.sensors.iter().map(|s| s.name.clone()).collect();
                s
            })
            .unwrap_or_default();

        Self {
            devices,
            sensors,
            sel_device,
            sel_device_name,
            ..Self::default()
        }
    }

    pub fn select_device(&mut self, serial: String) -> anyhow::Result<()> {
        let is_same_device = self.sel_device.as_deref() == Some(serial.as_str());
        if is_same_device {
            return Ok(());
        }

        let device = self
            .devices
            .iter()
            .find(|d| d.serial.as_deref() == Some(serial.as_str()));

        let Some(device) = device else {
            anyhow::bail!("Logic error. No device with serial {} was found", serial);
        };

        self.sel_device = Some(serial.to_owned());
        self.sel_device_name = Some(device.name.clone());
        self.sensors = device.sensors.iter().map(|s| s.name.clone()).collect();
        self.sel_sensor = None;
        self.streams.clear();
        self.sel_stream = None;
        self.modes.clear();
        self.sel_mode = None;
        self.formats.clear();
        self.sel_format = None;
        Ok(())
    }

    pub fn selected_device_serial(&self) -> Option<&str> {
        self.sel_device.as_deref()
    }

    pub fn selected_device_name(&self) -> Option<&str> {
        self.sel_device_name.as_deref()
    }

    pub fn select_sensor(&mut self, sensor: String) -> anyhow::Result<()> {
        let is_same_sensor = self.sel_sensor.as_deref() == Some(sensor.as_str());
        if is_same_sensor {
            return Ok(());
        }

        let selected_device = self.devices.iter().find(|d| d.serial == self.sel_device);
        let Some(selected_device) = selected_device else {
            anyhow::bail!("Logic error. Unable to select sensor without active device");
        };

        let streams = selected_device.sensors.iter().find_map(|s| {
            if s.name == sensor {
                let streams: Vec<_> = s.streams.iter().map(|c| c.kind).collect();
                Some(streams)
            } else {
                None
            }
        });

        let Some(streams) = streams else {
            anyhow::bail!("Logic error. No sensor with name {} available", sensor);
        };

        self.sel_sensor = Some(sensor.to_owned());
        self.streams = streams;
        self.sel_stream = None;
        self.modes.clear();
        self.sel_mode = None;
        self.formats.clear();
        self.sel_format = None;
        Ok(())
    }

    pub fn selected_sensor(&self) -> Option<&str> {
        self.sel_sensor.as_deref()
    }

    pub fn sensors(&self) -> &Vec<String> {
        &self.sensors
    }

    pub fn select_stream(&mut self, stream: Rs2StreamKind) -> anyhow::Result<()> {
        let is_same_stream = self.sel_stream == Some(stream);
        if is_same_stream {
            return Ok(());
        }

        let selected_device = self.devices.iter().find(|d| d.serial == self.sel_device);
        let Some(selected_device) = selected_device else {
            anyhow::bail!("Logic error. Unable to select stream without active device");
        };

        let selected_sensor = selected_device
            .sensors
            .iter()
            .find(|s| Some(s.name.as_str()) == self.sel_sensor.as_deref());
        let Some(selected_sensor) = selected_sensor else {
            anyhow::bail!("Logic error. Unable to select stream without active sensor");
        };

        let modes = selected_sensor.streams.iter().find_map(|s| {
            if s.kind == stream {
                let modes: Vec<_> = s.profiles.iter().map(|p| p.mode).collect();
                Some(modes)
            } else {
                None
            }
        });
        let Some(mut modes) = modes else {
            anyhow::bail!("Logic error. No stream {} available", stream);
        };

        modes.sort_by_key(|m| std::cmp::Reverse(m.width));

        self.sel_stream = Some(stream);
        self.modes = modes;
        self.sel_mode = None;
        self.formats.clear();
        self.sel_format = None;
        Ok(())
    }

    pub fn selected_stream(&self) -> Option<Rs2StreamKind> {
        self.sel_stream
    }

    pub fn streams(&self) -> &Vec<Rs2StreamKind> {
        &self.streams
    }

    pub fn select_mode(&mut self, mode: Mode) -> anyhow::Result<()> {
        let selected_device = self.devices.iter().find(|d| d.serial == self.sel_device);
        let Some(selected_device) = selected_device else {
            anyhow::bail!("Logic error. Unable to select mode without active device");
        };

        let selected_sensor = selected_device
            .sensors
            .iter()
            .find(|s| Some(s.name.as_str()) == self.sel_sensor.as_deref());
        let Some(selected_sensor) = selected_sensor else {
            anyhow::bail!("Logic error. Unable to select mode without active sensor");
        };

        let selected_stream = selected_sensor
            .streams
            .iter()
            .find(|s| Some(s.kind) == self.sel_stream);
        let Some(selected_stream) = selected_stream else {
            anyhow::bail!("Logic error. Unable to select mode without active stream");
        };

        let formats = selected_stream.profiles.iter().find_map(|p| {
            if p.mode == mode {
                Some(&p.formats)
            } else {
                None
            }
        });
        let Some(formats) = formats else {
            anyhow::bail!("Logic error. No mode {:#?} available", mode);
        };

        self.sel_mode = Some(mode);
        self.formats = formats
            .iter()
            .filter_map(|f| PixelFormat::try_from(*f).ok())
            .collect();
        self.sel_format = None;

        Ok(())
    }

    pub fn selected_mode(&self) -> Option<Mode> {
        self.sel_mode
    }

    pub fn modes(&self) -> &Vec<Mode> {
        &self.modes
    }

    pub fn select_format(&mut self, format: PixelFormat) -> anyhow::Result<()> {
        let selected_device = self.devices.iter().find(|d| d.serial == self.sel_device);
        let Some(selected_device) = selected_device else {
            anyhow::bail!("Logic error. Unable to select format without active device");
        };

        let selected_sensor = selected_device
            .sensors
            .iter()
            .find(|s| Some(s.name.as_str()) == self.sel_sensor.as_deref());
        let Some(selected_sensor) = selected_sensor else {
            anyhow::bail!("Logic error. Unable to select format without active sensor");
        };

        let selected_stream = selected_sensor
            .streams
            .iter()
            .find(|s| Some(s.kind) == self.sel_stream);
        let Some(_) = selected_stream else {
            anyhow::bail!("Logic error. Unable to select stream without active stream");
        };

        if self.sel_mode.is_none() {
            anyhow::bail!("Logic error. Unable to select stream without active mode");
        };

        if !self.formats.contains(&format) {
            anyhow::bail!("Logic error. No format {} available", format);
        }

        self.sel_format = Some(format);

        Ok(())
    }

    pub fn formats(&self) -> &Vec<PixelFormat> {
        &self.formats
    }

    pub fn selected_format(&self) -> Option<PixelFormat> {
        self.sel_format
    }

    pub fn export_config(&self) -> Config {
        Config {
            sel_device: self.sel_device.clone(),
            sel_sensor: self.sel_sensor.clone(),
            sel_stream: self.sel_stream.map(|s| s as i32),
            sel_mode: self.sel_mode,
            sel_format: self.sel_format.map(|f| f as i32),
        }
    }

    pub fn apply_config(&mut self, config: Config) -> anyhow::Result<()> {
        if let Some(s) = config.sel_device {
            self.select_device(s)?;
        } else {
            return Ok(());
        }

        if let Some(s) = config.sel_sensor {
            self.select_sensor(s)?;
        } else {
            return Ok(());
        }

        if let Some(s) = config.sel_stream {
            use num_traits::FromPrimitive;
            let kind =
                Rs2StreamKind::from_i32(s).context("Unable to parse Rs2StreamKind from {s}")?;
            self.select_stream(kind)?;
        } else {
            return Ok(());
        }

        if let Some(m) = config.sel_mode {
            self.select_mode(m)?;
        } else {
            return Ok(());
        }

        if let Some(f) = config.sel_format {
            let format = PixelFormat::try_from(f)
                .ok()
                .context(format!("Unable to parse PixelFormat from {f}"))?;
            self.select_format(format)?;
        } else {
            return Ok(());
        }

        Ok(())
    }
}
