//! C ABI parity with RIS ReportNativeExports. Buffers belong to this library.
use std::{
    ffi::{CStr, c_char, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};
const INVALID_INPUT: i32 = 1;
const RENDER_FAILURE: i32 = 2;
unsafe fn allocate(bytes: &[u8]) -> anyhow::Result<*mut c_void> {
    let p = unsafe { libc::malloc(bytes.len().max(1)) };
    if p.is_null() {
        anyhow::bail!("Cannot allocate output buffer")
    };
    if !bytes.is_empty() {
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), p.cast(), bytes.len()) }
    }
    Ok(p)
}
unsafe fn fail(err: *mut *mut c_void, code: i32, error: &str) -> i32 {
    if !err.is_null() {
        let mut message = error.as_bytes().to_vec();
        message.push(0);
        unsafe { *err = allocate(&message).unwrap_or(ptr::null_mut()) }
    }
    code
}
unsafe fn input(
    kind: *const c_char,
    json: *const u8,
    len: i32,
) -> anyhow::Result<(String, serde_json::Value)> {
    if kind.is_null() || json.is_null() || len <= 0 {
        anyhow::bail!("report type and JSON payload are required.")
    }
    let name = unsafe { CStr::from_ptr(kind) }.to_str()?.to_string();
    if ![
        "debt",
        "debt-out",
        "income",
        "substitute",
        "receipt",
        "debt-receipt",
        "large-receipt",
    ]
    .contains(&name.as_str())
    {
        anyhow::bail!("Unsupported report type '{name}'.")
    }
    let value = serde_json::from_slice(unsafe { std::slice::from_raw_parts(json, len as usize) })?;
    Ok((name, value))
}
/// # Safety
/// All non-null pointers must reference valid storage for their stated lengths.
/// The caller must release returned buffers with doc_reports_free_buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn doc_reports_generate_pdf(
    kind: *const c_char,
    json: *const u8,
    len: i32,
    data: *mut *mut c_void,
    data_len: *mut i32,
    error: *mut *mut c_void,
) -> i32 {
    if !data.is_null() {
        unsafe { *data = ptr::null_mut() }
    }
    if !data_len.is_null() {
        unsafe { *data_len = 0 }
    }
    if !error.is_null() {
        unsafe { *error = ptr::null_mut() }
    }
    if data.is_null() || data_len.is_null() || error.is_null() {
        return unsafe { fail(error, INVALID_INPUT, "Output pointers are required.") };
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let (kind, value) =
            unsafe { input(kind, json, len) }.map_err(|e| (INVALID_INPUT, e.to_string()))?;
        let document = crate::compose(&kind, value).map_err(|e| (INVALID_INPUT, e.to_string()))?;
        let pdf = crate::documents::render_document(&document)
            .map_err(|e| (RENDER_FAILURE, format!("{e:#}")))?;
        let length = i32::try_from(pdf.len())
            .map_err(|_| (RENDER_FAILURE, "PDF exceeds C ABI i32 length".into()))?;
        let buffer = unsafe { allocate(&pdf) }.map_err(|e| (RENDER_FAILURE, e.to_string()))?;
        unsafe {
            *data = buffer;
            *data_len = length;
        }
        Ok::<_, (i32, String)>(())
    })) {
        Ok(Ok(())) => 0,
        Ok(Err((code, message))) => unsafe { fail(error, code, &message) },
        Err(_) => unsafe { fail(error, RENDER_FAILURE, "Rust document rendering panicked.") },
    }
}
/// # Safety
/// ptr must be null or an outstanding buffer allocated by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn doc_reports_free_buffer(buffer: *mut c_void) {
    if !buffer.is_null() {
        unsafe { libc::free(buffer) }
    }
}
/// # Safety
/// Input pointers must be valid NUL-terminated strings or a JSON byte slice.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn doc_reports_print_receipt(
    kind: *const c_char,
    json: *const u8,
    len: i32,
    host: *const c_char,
    port: i32,
    error: *mut *mut c_void,
) -> i32 {
    if !error.is_null() {
        unsafe { *error = ptr::null_mut() }
    }
    if error.is_null() {
        return INVALID_INPUT;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        if host.is_null() || !(1..=65535).contains(&port) {
            return Err((
                INVALID_INPUT,
                "Printer host and a port between 1 and 65535 are required.".into(),
            ));
        }
        let (name, mut payload) =
            unsafe { input(kind, json, len) }.map_err(|e| (INVALID_INPUT, e.to_string()))?;
        if !["receipt", "debt-receipt", "large-receipt"].contains(&name.as_str()) {
            return Err((INVALID_INPUT, format!("Unsupported receipt type '{name}'.")));
        }
        let host = unsafe { CStr::from_ptr(host) }
            .to_str()
            .map_err(|e| (INVALID_INPUT, e.to_string()))?;
        if host.trim().is_empty() {
            return Err((INVALID_INPUT, "Printer host is required.".into()));
        }
        payload = crate::util::normalize_keys(payload);
        if name == "debt-receipt" {
            payload["is_debt_receipt"] = true.into()
        }
        let doc = crate::compose(&name, payload).map_err(|e| (INVALID_INPUT, e.to_string()))?;
        crate::printer::print_receipt(&doc, host.trim(), port as u16)
            .map_err(|e| (RENDER_FAILURE, e.to_string()))?;
        Ok(())
    })) {
        Ok(Ok(())) => 0,
        Ok(Err((code, message))) => unsafe { fail(error, code, &message) },
        Err(_) => unsafe { fail(error, RENDER_FAILURE, "Rust receipt printing panicked.") },
    }
}
