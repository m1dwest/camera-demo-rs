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

    pub fn update(&mut self, devices: &[Device]) {
        let current_serial = self.current_item().and_then(|item| item.serial.as_deref());

        let items: Vec<_> = devices.iter().map(DevicesModelItem::from_device).collect();
        let selected_index = items
            .iter()
            .position(|item| item.serial.as_deref() == current_serial && current_serial.is_some())
            .unwrap_or(0);

        self.items = items;
        self.selected_index = selected_index;
    }

    pub fn current_item(&self) -> Option<&DevicesModelItem> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.selected_index])
        }
    }

    pub fn current_index(&self) -> usize {
        self.selected_index
    }

    pub fn set_selection(&mut self, serial: &str) -> bool {
        let index = self
            .items
            .iter()
            .position(|item| item.serial.as_deref() == Some(serial));

        if let Some(i) = index {
            self.selected_index = i;
            true
        } else {
            false
        }
    }
}
