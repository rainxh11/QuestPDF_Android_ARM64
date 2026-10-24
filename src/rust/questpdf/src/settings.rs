//! Process-wide settings corresponding to QuestPDF.Settings.
use crate::LicenseType;
use anyhow::{Result, ensure};
use std::{
    path::PathBuf,
    sync::{
        LazyLock, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Debug)]
pub struct GlobalSettings {
    pub license: Option<LicenseType>,
    pub enable_caching: bool,
    pub enable_detailed_layout_errors: bool,
    pub throw_on_missing_text_glyphs: bool,
    pub throw_on_missing_font_families: bool,
    pub use_system_fonts: bool,
    pub font_discovery_path: Option<PathBuf>,
    pub temporary_storage_path: Option<PathBuf>,
}
impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            license: None,
            enable_caching: true,
            enable_detailed_layout_errors: false,
            throw_on_missing_text_glyphs: true,
            throw_on_missing_font_families: true,
            use_system_fonts: false,
            font_discovery_path: std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(std::path::Path::to_path_buf)),
            temporary_storage_path: None,
        }
    }
}
static SETTINGS: LazyLock<RwLock<GlobalSettings>> =
    LazyLock::new(|| RwLock::new(GlobalSettings::default()));
static LICENSE_VALIDATED: AtomicBool = AtomicBool::new(false);
pub struct Settings;
impl Settings {
    /// Configure once at application startup, before font discovery or generation.
    pub fn configure(change: impl FnOnce(&mut GlobalSettings)) {
        change(&mut SETTINGS.write().unwrap_or_else(|e| e.into_inner()));
    }
    pub fn get() -> GlobalSettings {
        SETTINGS.read().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn set_license(license: LicenseType) {
        Self::configure(|s| s.license = Some(license));
    }
    pub fn set_enable_caching(enabled: bool) {
        Self::configure(|s| s.enable_caching = enabled);
    }
    pub fn set_use_system_fonts(enabled: bool) {
        Self::configure(|s| s.use_system_fonts = enabled);
    }
    pub fn set_font_discovery_path(path: Option<PathBuf>) {
        Self::configure(|s| s.font_discovery_path = path);
    }
}
pub(crate) fn validate_license() -> Result<()> {
    if LICENSE_VALIDATED.load(Ordering::Acquire) {
        return Ok(());
    }
    let license = Settings::get().license;
    ensure!(
        license.is_some(),
        "Configure the QuestPDF license at application startup using Settings::set_license(LicenseType::Evaluation), or the applicable Community, Professional or Enterprise tier. See the accompanying LICENSE.md."
    );
    if license == Some(LicenseType::Evaluation) {
        eprintln!(
            "[QuestPDF] Evaluation mode is fully functional and intended for evaluation before selecting a license. See the accompanying LICENSE.md."
        );
    }
    LICENSE_VALIDATED.store(true, Ordering::Release);
    Ok(())
}
