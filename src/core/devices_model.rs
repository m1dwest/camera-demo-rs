use crate::core::Device;
use crate::core::Mode;
use crate::core::Stream;

use realsense_rust::kind::{Rs2Format, Rs2StreamKind};

#[derive(Default)]
pub struct DevicesModel {
    pub devices: Vec<Device>,
    selected_serial: Option<String>,

    sensors: Vec<String>,
    selected_sensor: Option<String>,

    streams: Vec<Rs2StreamKind>,
    selected_stream: Option<Rs2StreamKind>,

    modes: Vec<Mode>,
    selected_mode: Option<Mode>,
}

impl DevicesModel {
    pub fn from_devices(devices: Vec<Device>, current: Option<Self>) -> Self {
        if let Some(current) = current
            && let Some(curr_serial) = &current.selected_serial
        {
            let serial_found = devices
                .iter()
                .any(|d| d.serial.as_ref() == Some(curr_serial));
            if serial_found {
                return Self { devices, ..current };
            }
        }

        let selected_device = devices.iter().find(|d| d.serial.is_some());
        let selected_serial = selected_device.and_then(|d| d.serial.clone());
        let sensors = selected_device
            .map(|d| {
                let s: Vec<_> = d.sensors.iter().map(|s| s.name.clone()).collect();
                s
            })
            .unwrap_or_default();

        Self {
            devices,
            sensors,
            selected_serial,
            ..Self::default()
        }
    }

    pub fn select_device(&mut self, serial: String) -> anyhow::Result<()> {
        let is_same_device = self.selected_serial.as_deref() == Some(serial.as_str());
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

        self.selected_serial = Some(serial.to_owned());
        self.sensors = device.sensors.iter().map(|s| s.name.clone()).collect();
        self.selected_sensor = None;
        self.streams.clear();
        self.selected_stream = None;
        self.modes.clear();
        self.selected_mode = None;
        Ok(())
    }

    pub fn selected_device(&self) -> Option<&Device> {
        self.devices
            .iter()
            .find(|d| d.serial == self.selected_serial)
    }

    pub fn select_sensor(&mut self, sensor: String) -> anyhow::Result<()> {
        let is_same_sensor = self.selected_sensor.as_deref() == Some(sensor.as_str());
        if is_same_sensor {
            return Ok(());
        }

        let Some(selected_device) = self.selected_device() else {
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

        self.selected_sensor = Some(sensor.to_owned());
        self.streams = streams;
        self.selected_stream = None;
        self.modes.clear();
        self.selected_mode = None;
        Ok(())
    }

    pub fn selected_sensor(&self) -> Option<&str> {
        self.selected_sensor.as_deref()
    }

    pub fn sensors(&self) -> &Vec<String> {
        &self.sensors
    }

    pub fn select_stream(&mut self, stream: Rs2StreamKind) -> anyhow::Result<()> {
        let is_same_stream = self.selected_stream == Some(stream);
        if is_same_stream {
            return Ok(());
        }

        let Some(selected_device) = self.selected_device() else {
            anyhow::bail!("Logic error. Unable to select stream without active device");
        };

        let Some(selected_sensor) = self.selected_sensor() else {
            anyhow::bail!("Logic error. Unable to select stream without active sensor");
        };

        let streams = selected_device.sensors.iter().find_map(|sensor| {
            if sensor.name == selected_sensor {
                Some(&sensor.streams)
            } else {
                None
            }
        });

        let Some(streams) = streams else {
            anyhow::bail!(
                "Logic error. No sensor with name {} available",
                selected_sensor
            );
        };

        let modes = streams.iter().find_map(|s| {
            if s.kind == stream {
                let modes: Vec<_> = s.profiles.iter().map(|p| p.mode.clone()).collect();
                Some(modes)
            } else {
                None
            }
        });

        let Some(mut modes) = modes else {
            anyhow::bail!("Logic error. No stream {} available", stream);
        };

        modes.sort_by_key(|m| std::cmp::Reverse(m.width));

        self.selected_stream = Some(stream);
        self.modes = modes;
        self.selected_mode = None;
        Ok(())
    }

    pub fn selected_stream(&self) -> Option<Rs2StreamKind> {
        self.selected_stream
    }

    pub fn streams(&self) -> &Vec<Rs2StreamKind> {
        &self.streams
    }

    pub fn select_mode(&mut self, mode: Mode) -> anyhow::Result<()> {
        if self.selected_device().is_none() {
            anyhow::bail!("Logic error. Unable to select mode without active device");
        };

        if self.sensors.is_empty() {
            anyhow::bail!("Logic error. Unable to select mode without active sensor");
        }

        if self.streams.is_empty() {
            anyhow::bail!("Logic error. Unable to select mode without active stream");
        }

        if self.modes.iter().any(|m| m == &mode) {
            self.selected_mode = Some(mode);
            Ok(())
        } else {
            anyhow::bail!("No mode {:#?} available", mode);
        }
    }

    pub fn selected_mode(&self) -> Option<&Mode> {
        self.selected_mode.as_ref()
    }

    pub fn modes(&self) -> &Vec<Mode> {
        &self.modes
    }
}
