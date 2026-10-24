//! Process-wide application font registrations, with optional system-font fallback.
use crate::{FontInfo, native::Fonts};
use anyhow::{Result, bail};
use std::{
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};
#[derive(Default)]
struct Registry {
    fonts: Vec<Arc<[u8]>>,
}
static REGISTRY: Mutex<Registry> = Mutex::new(Registry { fonts: vec![] });
pub struct FontManager;
impl FontManager {
    pub fn register_font_from_bytes(bytes: &[u8]) -> Result<()> {
        let mut fonts = Fonts::new()?;
        fonts.register(bytes)?;
        let mut registry = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        if !registry.fonts.iter().any(|f| &**f == bytes) {
            registry.fonts.push(Arc::from(bytes));
        }
        Ok(())
    }
    pub fn register_font_from_file(path: impl AsRef<Path>) -> Result<()> {
        Self::register_font_from_bytes(&std::fs::read(crate::runtime::resolve_resource_file_path(
            path,
        )?)?)
    }
    pub fn register_font_from_stream(mut reader: impl Read) -> Result<()> {
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;
        Self::register_font_from_bytes(&bytes)
    }
    pub fn register_fonts_from_directory(path: impl AsRef<Path>) -> Result<()> {
        fn visit(path: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
            for item in std::fs::read_dir(path)? {
                let item = item?;
                let path = item.path();
                if item.file_type()?.is_dir() {
                    visit(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                    ["ttf", "otf", "ttc", "pfb"].contains(&s.to_ascii_lowercase().as_str())
                }) {
                    files.push(path);
                }
            }
            Ok(())
        }
        if !path.as_ref().is_dir() {
            bail!(
                "Font discovery directory does not exist: {}",
                path.as_ref().display()
            );
        }
        let mut files = vec![];
        visit(path.as_ref(), &mut files)?;
        files.sort();
        for file in files {
            let _ = Self::register_font_from_file(file);
        }
        Ok(())
    }
    pub fn set_use_system_fonts(enabled: bool) {
        crate::Settings::set_use_system_fonts(enabled);
    }
    pub fn get_registered_fonts() -> Result<Vec<FontInfo>> {
        let mut fonts = Fonts::new()?;
        apply(&mut fonts)?;
        Ok(fonts.registered_fonts())
    }
    pub fn get_system_fonts() -> Result<Vec<FontInfo>> {
        Ok(Fonts::new()?.system_fonts())
    }
    pub fn is_font_family_available(name: &str) -> Result<bool> {
        let mut fonts = Fonts::new()?;
        apply(&mut fonts)?;
        Ok(fonts.registered.contains(&name.to_lowercase()))
    }
}
pub(crate) fn apply(fonts: &mut Fonts) -> Result<()> {
    static DISCOVERY: std::sync::Once = std::sync::Once::new();
    DISCOVERY.call_once(|| {
        if let Some(path) = crate::Settings::get().font_discovery_path {
            let _ = FontManager::register_fonts_from_directory(path);
        }
    });
    let registry = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    let data = registry.fonts.clone();
    let system = crate::Settings::get().use_system_fonts;
    drop(registry);
    for bytes in data {
        fonts.register(&bytes)?;
    }
    if system {
        fonts.use_system_fonts(true)?;
    }
    Ok(())
}
