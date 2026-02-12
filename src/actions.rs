use realsense_rust::kind::Rs2StreamKind;

pub enum Action {
    None,
    ChangeCamera { serial: String },
    StartCamera,
    StopCamera,
    RefreshDevices,
    SelectSensor { sensor: String },
    SelectStream { stream: Rs2StreamKind },
    SelectMode { mode: crate::core::Mode },
}
