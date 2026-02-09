use crate::core::Device;
use crate::core::Mode;

use realsense_rust::kind::Rs2StreamKind;

#[derive(Default)]
pub struct DevicesModel {
    pub devices: Vec<Device>,
    selected_serial: Option<String>,

    sensors: Vec<String>,
    selected_sensor: Option<String>,

    kinds: Vec<Rs2StreamKind>,
    selected_kind: Option<Rs2StreamKind>,

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

        let selected_serial = devices.iter().find_map(|d| d.serial.clone());

        Self {
            devices,
            selected_serial,
            ..Self::default()
        }
    }

    pub fn select_device(&mut self, serial: &str) -> anyhow::Result<()> {
        let is_same_device = self.selected_serial.as_deref() == Some(serial);
        if is_same_device {
            return Ok(());
        }

        let device = self
            .devices
            .iter()
            .find(|d| d.serial.as_deref() == Some(serial));

        if let Some(device) = device {
            self.selected_serial = Some(serial.to_owned());
            self.sensors = device.capabilities.iter().map(|c| c.0.clone()).collect();
            self.selected_sensor = None;
            self.kinds.clear();
            self.selected_kind = None;
            self.modes.clear();
            self.selected_mode = None;
            Ok(())
        } else {
            anyhow::bail!("No device with serial {} was found", serial);
        }
    }

    pub fn selected_device(&self) -> Option<&Device> {
        self.devices
            .iter()
            .find(|d| d.serial == self.selected_serial)
    }

    pub fn select_sensor(&mut self, sensor: &str) -> anyhow::Result<()> {
        let is_same_sensor = self.selected_sensor.as_deref() == Some(sensor);
        if is_same_sensor {
            return Ok(());
        }

        let Some(selected_device) = self.selected_device() else {
            anyhow::bail!("Logic error. Unable to select sensor without active device");
        };

        let kinds = selected_device
            .capabilities
            .iter()
            .find_map(|(sensor, cap)| {
                if sensor.as_str() == sensor {
                    Some(cap.get_kinds())
                } else {
                    None
                }
            });

        if let Some(kinds) = kinds {
            self.selected_sensor = Some(sensor.to_owned());
            self.kinds = kinds;
            self.selected_kind = None;
            self.modes.clear();
            self.selected_mode = None;
            Ok(())
        } else {
            anyhow::bail!("No sensor with name {} available", sensor);
        }
    }

    pub fn selected_sensor(&self) -> &Option<String> {
        &self.selected_sensor
    }

    pub fn select_kind(&mut self, kind: Rs2StreamKind) -> anyhow::Result<()> {
        let is_same_kind = self.selected_kind == Some(kind);
        if is_same_kind {
            return Ok(());
        }

        let Some(selected_device) = self.selected_device() else {
            anyhow::bail!("Logic error. Unable to select stream kind without active device");
        };

        if self.sensors.is_empty() {
            anyhow::bail!("Logic error. Unable to select stream kind without active sensor");
        }

        let cap = selected_device
            .capabilities
            .iter()
            .find_map(|(sensor, cap)| {
                if Some(sensor) == self.selected_sensor.as_ref() {
                    Some(cap)
                } else {
                    None
                }
            });

        let modes = cap.iter().find_map(|cap| cap.get_modes_for(kind)).cloned();

        if let Some(modes) = modes {
            self.selected_kind = Some(kind);
            self.modes = modes;
            self.selected_mode = None;
            Ok(())
        } else {
            anyhow::bail!("No kind {} available", kind);
        }
    }

    pub fn select_mode(&mut self, mode: Mode) -> anyhow::Result<()> {
        if self.selected_device().is_none() {
            anyhow::bail!("Logic error. Unable to select stream kind without active device");
        };

        if self.sensors.is_empty() {
            anyhow::bail!("Logic error. Unable to select stream kind without active sensor");
        }

        if self.kinds.is_empty() {
            anyhow::bail!("Logic error. Unable to select stream kind without active kind");
        }

        if self.modes.iter().any(|m| m == &mode) {
            self.selected_mode = Some(mode);
            Ok(())
        } else {
            anyhow::bail!("No mode {:#?} available", mode);
        }
    }
}
