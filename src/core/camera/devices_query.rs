use anyhow::{Context, Result};

use realsense_rust as rs;

pub struct Devices {
    devices: std::collections::HashMap<String, rs::device::Device>,
}

impl Devices {
    pub fn query(context: &rs::context::Context) -> Self {
        let mut query = std::collections::HashSet::new();
        query.insert(rs::kind::Rs2ProductLine::D400);

        let devices = context.query_devices(query);

        Self {
            devices: devices
                .into_iter()
                .map_while(|d| {
                    let sn = d.info(rs::kind::Rs2CameraInfo::SerialNumber)?;
                    let sn = sn.to_str().ok()?;
                    Some((sn.to_owned(), d))
                })
                .collect(),
        }
    }

    pub fn get_names(&self) -> Vec<(String, String)> {
        self.devices
            .iter()
            .map_while(|(sn, d)| {
                let name = d.info(rs::kind::Rs2CameraInfo::Name)?;
                let name = name.to_str().ok()?;
                Some((sn.clone(), name.to_owned()))
            })
            .collect()
    }
}
