//! Locate native dependencies for applications and exported shared libraries.
use std::path::{Path, PathBuf};

#[cfg(windows)]
fn module_directory() -> Option<PathBuf> {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleExW(flags: u32, address: *const u16, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, filename: *mut u16, length: u32) -> u32;
    }
    let mut module = std::ptr::null_mut();
    let mut filename = vec![0u16; 32768];
    unsafe {
        if GetModuleHandleExW(6, module_directory as *const () as *const u16, &mut module) == 0 {
            return None;
        }
        let length = GetModuleFileNameW(module, filename.as_mut_ptr(), filename.len() as u32);
        if length == 0 || length as usize == filename.len() {
            return None;
        }
        PathBuf::from(String::from_utf16_lossy(&filename[..length as usize]))
            .parent()
            .map(Path::to_path_buf)
    }
}

#[cfg(unix)]
fn module_directory() -> Option<PathBuf> {
    unsafe {
        let mut info: libc::Dl_info = std::mem::zeroed();
        if libc::dladdr(module_directory as *const () as *const _, &mut info) == 0
            || info.dli_fname.is_null()
        {
            return None;
        }
        let path = std::ffi::CStr::from_ptr(info.dli_fname).to_string_lossy();
        Path::new(path.as_ref()).parent().map(Path::to_path_buf)
    }
}

#[cfg(not(any(windows, unix)))]
fn module_directory() -> Option<PathBuf> {
    None
}

pub(crate) fn native_path(variable: &str, file: &str, rid: &str) -> anyhow::Result<PathBuf> {
    if let Some(path) = std::env::var_os(variable) {
        return Ok(path.into());
    }
    let directories = [
        module_directory(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf)),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
    ];
    for directory in directories.into_iter().flatten() {
        for relative in [
            PathBuf::from(file),
            PathBuf::from(format!("runtimes/{rid}/native/{file}")),
        ] {
            let candidate = directory.join(relative);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    // Source-checkout convenience; packaged applications resolve above this point.
    anyhow::ensure!(
        std::env::var_os("QUESTPDF_DISABLE_DEVELOPMENT_LOOKUP").is_none(),
        "Native {file} was not found beside the Rust library or executable; configure {variable}"
    );
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../dotnet/library/QuestPDF/Runtimes/{rid}/native/{file}"
    )))
}

/// Directory containing the Rust engine, including when loaded through its C ABI.
pub fn library_directory() -> Option<PathBuf> {
    module_directory()
}
pub fn application_files_directory() -> PathBuf {
    library_directory()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(Path::to_path_buf))
        })
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
/// Resolve resource inputs against the working directory, then the application.
pub fn resolve_resource_file_path(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let path = path.as_ref();
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    if !path.is_absolute() {
        let candidate = application_files_directory().join(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    anyhow::bail!(
        "Resource file not found: {} (working directory and application directory checked)",
        path.display()
    )
}
