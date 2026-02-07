use crate::core::Device;

pub struct DevicesModelItem {
    pub name: String,
    pub serial: Option<String>,
}

impl DevicesModelItem {
    const UNKNOWN_NAME: &str = "Unknown device";

    fn from_device(device: &Device) -> Self {
        Self {
            name: device.name.clone().unwrap_or(Self::UNKNOWN_NAME.to_owned()),
            serial: device.serial.clone(),
        }
    }
}

pub struct DevicesModel {
    pub items: Vec<DevicesModelItem>,
    selected_index: usize,
}

impl DevicesModel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
        }
    }

    pub fn update(&self, devices: &[Device]) -> Self {
        let items: Vec<_> = devices.iter().map(DevicesModelItem::from_device).collect();

        let selected_serial = if !self.items.is_empty() {
            self.items[self.selected_index].serial.as_ref()
        } else {
            None
        };

        let selected_index = items
            .iter()
            .position(|item| item.serial.as_ref() == selected_serial)
            .unwrap_or(0);

        Self {
            items,
            selected_index,
        }
    }

    pub fn current_item(&self) -> &DevicesModelItem {
        &self.items[self.selected_index]
    }

    pub fn current_index(&self) -> usize {
        self.selected_index
    }

    pub fn set_current_index(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.items.len() {
            anyhow::bail!("Index out of range");
        }
        self.selected_index = index;
        Ok(())
    }
}
