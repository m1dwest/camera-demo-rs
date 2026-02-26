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
    SelectFormat { format: crate::core::PixelFormat },
}

pub struct InferenceConfig {
    pub model_path: String,
    pub classes_path: String,
    pub input_size: u32,
    pub prob_threshold: f32,
}

pub enum VisionAction {
    Stop,
    EnableInference { config: InferenceConfig },
    DisableInference,
}
