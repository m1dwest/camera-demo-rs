use realsense_rust::kind::Rs2StreamKind;

pub enum Action {
    None,
    ChangeCamera { serial: String },
    // TODO: remove?
    StartCamera,
    StopCamera,
    RefreshDevices,
    SelectSensor { sensor: String },
    SelectStream { stream: Rs2StreamKind },
    SelectMode { mode: crate::core::Mode },
}

pub enum CameraAction {
    Stop,
}
