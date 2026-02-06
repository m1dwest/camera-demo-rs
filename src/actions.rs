pub enum Action {
    None,
    ChangeCamera { serial: String },
    DisableCamera,
    RefreshDeviceList,
}
