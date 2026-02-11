use realsense_rust::kind::Rs2StreamKind;

pub enum Action {
    None,
    ChangeCamera { serial: String },
    DisableCamera,
    RefreshDeviceList,
    SelectSensor { sensor: String },
    SelectStream { stream: Rs2StreamKind },
}
