//! Native qpdf job execution, mirroring QuestPDF/Qpdf/QpdfAPI.cs.
use crate::native::{Ptr, cstr};
use anyhow::{Context, Result, bail};
use libloading::Library;
use std::{
    ffi::c_char,
    path::{Path, PathBuf},
    ptr,
};
type LogCallback = unsafe extern "C" fn(Ptr, i32, Ptr) -> i32;
macro_rules! bindings{($($name:ident($($ty:ty),*)$(->$ret:ty)?;)+)=>{
 struct Api{_library:Library,$($name:unsafe extern "C" fn($($ty),*)$(->$ret)?,)+}
 impl Api{unsafe fn load(path:&Path)->Result<Self>{let library=unsafe{Library::new(path)}.context("load QuestPDF native qpdf")?;$(let $name=unsafe{*library.get::<unsafe extern "C" fn($($ty),*)$(->$ret)?>(concat!(stringify!($name),"\0").as_bytes())?};)+Ok(Self{_library:library,$($name,)+})}}
}}
bindings! {
 questpdf_get_compatibility_version()->i32;
 qpdfjob_init()->Ptr;
 qpdfjob_cleanup(*mut Ptr);
 qpdfjob_initialize_from_json(Ptr,*const c_char)->i32;
 qpdfjob_create_qpdf(Ptr)->Ptr;
 qpdfjob_write_qpdf(Ptr,Ptr)->i32;
 qpdf_cleanup(*mut Ptr);
 qpdflogger_create()->Ptr;
 qpdflogger_cleanup(*mut Ptr);
 qpdflogger_set_error(Ptr,i32,LogCallback,Ptr);
 qpdfjob_set_logger(Ptr,Ptr);
 questpdf_job_get_xmp_metadata(Ptr,Ptr,*mut Ptr,*mut usize)->i32;
 questpdf_job_set_xmp_metadata(Ptr,Ptr,*const u8,usize)->i32;
 qpdf_oh_free_buffer(*mut Ptr);
}
struct Job<'a> {
    api: &'a Api,
    logger: Ptr,
    job: Ptr,
    document: Ptr,
}
impl Drop for Job<'_> {
    fn drop(&mut self) {
        unsafe {
            if !self.document.is_null() {
                (self.api.qpdf_cleanup)(&mut self.document)
            }
            if !self.job.is_null() {
                (self.api.qpdfjob_cleanup)(&mut self.job)
            }
            if !self.logger.is_null() {
                (self.api.qpdflogger_cleanup)(&mut self.logger)
            }
        }
    }
}
unsafe extern "C" fn log(data: Ptr, length: i32, user: Ptr) -> i32 {
    if length > 0 && !data.is_null() && !user.is_null() {
        let out = unsafe { &mut *(user as *mut Vec<u8>) };
        out.extend_from_slice(unsafe {
            std::slice::from_raw_parts(data as *const u8, length as usize)
        })
    }
    0
}
fn path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("QUESTPDF_QPDF_PATH") {
        return Ok(path.into());
    }
    let (os, file) = if cfg!(target_os = "windows") {
        ("win", "questpdf_qpdf.dll")
    } else if cfg!(target_os = "macos") {
        ("osx", "libquestpdf_qpdf.dylib")
    } else if cfg!(target_os = "android") {
        ("android", "libquestpdf_qpdf.so")
    } else {
        ("linux", "libquestpdf_qpdf.so")
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "x64"
    };
    crate::runtime::native_path("QUESTPDF_QPDF_PATH", file, &format!("{os}-{arch}"))
}
pub type MetadataTransform<'a> = dyn Fn(&[u8]) -> Result<Vec<u8>> + 'a;
pub fn execute_job(
    configuration: &serde_json::Value,
    transform: Option<&MetadataTransform<'_>>,
) -> Result<()> {
    let a = unsafe { Api::load(&path()?) }?;
    if unsafe { (a.questpdf_get_compatibility_version)() } != 4 {
        bail!("qpdf ABI mismatch: this port requires ABI 4")
    }
    let mut errors = Vec::<u8>::new();
    let json = cstr(&serde_json::to_string(configuration)?);
    let mut job = Job {
        api: &a,
        logger: ptr::null_mut(),
        job: ptr::null_mut(),
        document: ptr::null_mut(),
    };
    let check = |status: i32, errors: &[u8]| -> Result<()> {
        if status == 2 {
            bail!(
                "QuestPDF could not perform document operation:\n{}",
                String::from_utf8_lossy(errors)
            )
        }
        Ok(())
    };
    unsafe {
        job.logger = (a.qpdflogger_create)();
        if job.logger.is_null() {
            bail!("Cannot create qpdf logger")
        }
        (a.qpdflogger_set_error)(job.logger, 4, log, (&mut errors as *mut Vec<u8>).cast());
        job.job = (a.qpdfjob_init)();
        if job.job.is_null() {
            bail!("Cannot create qpdf job")
        }
        (a.qpdfjob_set_logger)(job.job, job.logger);
        check(
            (a.qpdfjob_initialize_from_json)(job.job, json.as_ptr()),
            &errors,
        )?;
        job.document = (a.qpdfjob_create_qpdf)(job.job);
        if job.document.is_null() {
            check(2, &errors)?
        }
        if let Some(transform) = transform {
            let mut buffer = ptr::null_mut();
            let mut length = 0;
            check(
                (a.questpdf_job_get_xmp_metadata)(job.job, job.document, &mut buffer, &mut length),
                &errors,
            )?;
            if buffer.is_null() {
                bail!(
                    "Cannot extend metadata: input has no XMP packet; generate a PDF/A or PDF/UA document first"
                )
            }
            let original = std::slice::from_raw_parts(buffer as *const u8, length).to_vec();
            (a.qpdf_oh_free_buffer)(&mut buffer);
            let bytes = transform(&original)?;
            check(
                (a.questpdf_job_set_xmp_metadata)(
                    job.job,
                    job.document,
                    bytes.as_ptr(),
                    bytes.len(),
                ),
                &errors,
            )?;
        }
        check((a.qpdfjob_write_qpdf)(job.job, job.document), &errors)?;
    }
    Ok(())
}
