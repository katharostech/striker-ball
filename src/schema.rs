use super::*;

pub fn register_schemas() {
    Data::register_schema();
    LocalizationAsset::register_schema();
    FluentBundleAsset::register_schema();
    FluentResourceAsset::register_schema();
    SfxVolumeSetting::register_schema();
    MusicVolumeSetting::register_schema();
}
