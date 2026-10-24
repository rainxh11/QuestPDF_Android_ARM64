//! Native ABI mirrors of src/dotnet/library/QuestPDF/Skia. No CLR or managed calls.
#![allow(non_snake_case, dead_code)]
use anyhow::{Context, Result, bail};
use libloading::Library;
use std::{
    ffi::{CString, c_char, c_void},
    path::Path,
    ptr,
    sync::{Arc, OnceLock},
};
pub type Ptr = *mut c_void;
pub fn trace(message: &str) {
    if std::env::var_os("QUESTPDF_TRACE_NATIVE").is_some() {
        eprintln!("QuestPDF native: {message}")
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Matrix {
    pub scale_x: f32,
    pub skew_x: f32,
    pub translate_x: f32,
    pub skew_y: f32,
    pub scale_y: f32,
    pub translate_y: f32,
    pub perspective1: f32,
    pub perspective2: f32,
    pub perspective3: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LineExtent {
    pub top: f32,
    pub bottom: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DateTime {
    pub zone: i16,
    pub year: u16,
    pub month: u8,
    pub weekday: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PdfMetadata {
    pub title: Ptr,
    pub author: Ptr,
    pub subject: Ptr,
    pub keywords: Ptr,
    pub creator: Ptr,
    pub producer: Ptr,
    pub language: Ptr,
    pub created: DateTime,
    pub modified: DateTime,
    pub pdfa: i32,
    pub pdfua: i32,
    pub compress: bool,
    pub dpi: f32,
    pub semantic_root: Ptr,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FontFeature {
    pub name: *const c_char,
    pub value: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextStyleConfig {
    pub size: f32,
    pub weight: i32,
    pub flags: u32,
    pub families: [Ptr; 16],
    pub features: [FontFeature; 16],
    pub foreground: u32,
    pub background: u32,
    pub decoration_color: u32,
    pub decoration_thickness: f32,
    pub decoration: i32,
    pub decoration_mode: i32,
    pub decoration_style: i32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub baseline_offset: f32,
}
#[repr(C)]
#[derive(Default)]
pub struct ParagraphConfig {
    pub alignment: i32,
    pub direction: i32,
    pub max_lines: i32,
    pub ellipsis: Ptr,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Placeholder {
    pub width: f32,
    pub height: f32,
    pub alignment: i32,
    pub baseline: i32,
    pub offset: f32,
}
#[repr(C)]
#[derive(Default)]
pub struct ImageDetails {
    pub width: i32,
    pub height: i32,
    pub encoded_length: i32,
}
#[repr(C)]
#[derive(Default)]
pub struct DataBytes {
    pub length: i32,
    pub bytes: *const u8,
}
#[repr(C)]
#[derive(Default)]
pub struct SvgSize {
    pub width: f32,
    pub height: f32,
    pub width_unit: i32,
    pub height_unit: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RoundedRect {
    pub rect: Rect,
    pub tl: Point,
    pub tr: Point,
    pub br: Point,
    pub bl: Point,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub color: u32,
}

macro_rules! api {
 ($($name:ident($($arg:ident:$ty:ty),*) $(-> $ret:ty)?;)+) => {
  pub struct Api { _library:Library, $(pub $name:unsafe extern "C" fn($($ty),*) $(-> $ret)?,)+ }
  impl Api {
   unsafe fn load(path:&Path)->Result<Self> {
    let library=unsafe {Library::new(path)}.with_context(||format!("load native Skia {}",path.display()))?;
    let version=unsafe{library.get::<unsafe extern "C" fn()->i32>(b"questpdf_skia_get_compatibility_version\0")?()};
    if version!=26{bail!("Skia ABI {version}, expected 26 from the ported 2026.9.1 source")}
    $(let $name=unsafe{*library.get::<unsafe extern "C" fn($($ty),*) $(-> $ret)?>(concat!(stringify!($name),"\0").as_bytes())?};)+
    Ok(Self{_library:library,$($name,)+})
   }
  }
 }
}
api! {
 questpdf_skia_get_compatibility_version()->i32;
 questpdf_skia_unicode_create()->Ptr;
 questpdf_skia_data_create_from_binary(data:*const u8,length:i32)->Ptr;
 questpdf_skia_data_unref(data:Ptr);
 questpdf_skia_data_get_bytes(data:Ptr,result:*mut DataBytes);
 questpdf_skia_bitmap_create(width:i32,height:i32)->Ptr;
 questpdf_skia_bitmap_delete(bitmap:Ptr);
 questpdf_skia_bitmap_encode_as_png(bitmap:Ptr)->Ptr;
 questpdf_skia_bitmap_encode_as_jpg(bitmap:Ptr,quality:i32)->Ptr;
 questpdf_skia_bitmap_encode_as_webp(bitmap:Ptr,quality:i32)->Ptr;
 questpdf_skia_canvas_create_from_bitmap(bitmap:Ptr)->Ptr;
 questpdf_skia_canvas_delete(canvas:Ptr);
 questpdf_skia_picture_recorder_create()->Ptr;
 questpdf_skia_picture_recorder_begin_recording(recorder:Ptr,width:f32,height:f32)->Ptr;
 questpdf_skia_picture_recorder_end_recording(recorder:Ptr)->Ptr;
 questpdf_skia_picture_recorder_delete(recorder:Ptr);
 questpdf_skia_picture_unref(picture:Ptr);
 questpdf_skia_canvas_draw_picture(canvas:Ptr,picture:Ptr);
 questpdf_skia_typeface_font_provider_create()->Ptr;
 questpdf_skia_typeface_font_provider_add_typefaces_from_data(provider:Ptr,data:Ptr,alias:*const c_char)->i32;
 questpdf_skia_typeface_font_provider_as_font_manager(provider:Ptr)->Ptr;
 questpdf_skia_font_manager_get_typefaces(manager:Ptr,array:*mut *mut NativeFontInfo,length:*mut i32);
 questpdf_skia_font_manager_create_system()->Ptr;
 questpdf_skia_font_manager_delete_typefaces(array:*mut NativeFontInfo);
 questpdf_skia_typeface_font_provider_unref(provider:Ptr);
 questpdf_skia_font_collection_create(provider:Ptr,manager:Ptr)->Ptr;
 questpdf_skia_font_collection_unref(collection:Ptr);
 questpdf_skia_text_style_create(config:*const TextStyleConfig)->Ptr;
 questpdf_skia_text_style_delete(style:Ptr);
 questpdf_skia_paragraph_builder_create(config:*const ParagraphConfig,unicode:Ptr,fonts:Ptr)->Ptr;
 questpdf_skia_paragraph_builder_add_text(builder:Ptr,text:*const c_char,style:Ptr);
 questpdf_skia_paragraph_builder_add_placeholder(builder:Ptr,placeholder:*const Placeholder);
 questpdf_skia_paragraph_builder_create_paragraph(builder:Ptr)->Ptr;
 questpdf_skia_paragraph_builder_delete(builder:Ptr);
 questpdf_skia_paragraph_plan_layout(paragraph:Ptr,width:f32);
 questpdf_skia_paragraph_get_size(paragraph:Ptr,width:*mut f32,height:*mut f32);
 questpdf_skia_paragraph_get_line_extents(paragraph:Ptr,array:*mut *mut LineExtent,length:*mut i32);
 questpdf_skia_paragraph_delete_line_extents(array:*mut LineExtent);
 questpdf_skia_paragraph_get_placeholder_positions(paragraph:Ptr,array:*mut *mut Rect,length:*mut i32);
 questpdf_skia_paragraph_get_text_range_positions(paragraph:Ptr,start:i32,end:i32,array:*mut *mut Rect,length:*mut i32);
 questpdf_skia_paragraph_delete_positions(array:*mut Rect);
 questpdf_skia_paragraph_get_unresolved_codepoints(paragraph:Ptr,array:*mut *mut i32,length:*mut i32);
 questpdf_skia_paragraph_delete_unresolved_codepoints(array:*mut i32);
 questpdf_skia_paragraph_delete(paragraph:Ptr);
 questpdf_skia_write_stream_create(callback:unsafe extern "C" fn(Ptr,i32))->Ptr;
 questpdf_skia_write_stream_flush(stream:Ptr);
 questpdf_skia_write_stream_delete(stream:Ptr);
 questpdf_skia_pdf_document_create(stream:Ptr,metadata:*const PdfMetadata)->Ptr;
 questpdf_skia_document_begin_page(document:Ptr,width:f32,height:f32)->Ptr;
 questpdf_skia_document_end_page(document:Ptr);
 questpdf_skia_document_close(document:Ptr);
 questpdf_skia_document_unref(document:Ptr);
 questpdf_skia_canvas_save(canvas:Ptr);
 questpdf_skia_canvas_restore(canvas:Ptr);
 questpdf_skia_canvas_translate(canvas:Ptr,x:f32,y:f32);
 questpdf_skia_canvas_scale(canvas:Ptr,x:f32,y:f32);
 questpdf_skia_canvas_rotate(canvas:Ptr,degrees:f32);
 questpdf_skia_canvas_get_matrix9(canvas:Ptr,matrix:*mut Matrix);
 questpdf_skia_canvas_set_matrix9(canvas:Ptr,matrix:*const Matrix);
 questpdf_skia_canvas_set_semantic_node_id(canvas:Ptr,id:i32);
 questpdf_skia_pdf_structure_element_create(id:i32,kind:*const c_char,alt:*const c_char,language:*const c_char)->Ptr;
 questpdf_skia_pdf_structure_element_set_children(element:Ptr,children:*const Ptr,count:i32);
 questpdf_skia_pdf_structure_element_add_attribute_text(element:Ptr,owner:*const c_char,name:*const c_char,value:*const c_char);
 questpdf_skia_pdf_structure_element_add_attribute_integer(element:Ptr,owner:*const c_char,name:*const c_char,value:i32);
 questpdf_skia_pdf_structure_element_add_attribute_float(element:Ptr,owner:*const c_char,name:*const c_char,value:f32);
 questpdf_skia_pdf_structure_element_add_attribute_float_array(element:Ptr,owner:*const c_char,name:*const c_char,values:*const f32,length:i32);
 questpdf_skia_pdf_structure_element_add_attribute_node_ids(element:Ptr,owner:*const c_char,name:*const c_char,values:*const i32,length:i32);
 questpdf_skia_pdf_structure_element_delete(element:Ptr);
 questpdf_skia_canvas_draw_paragraph(canvas:Ptr,paragraph:Ptr,from:i32,to:i32);
 questpdf_skia_canvas_annotate_url(canvas:Ptr,width:f32,height:f32,url:*const c_char,description:*const c_char);
 questpdf_skia_canvas_annotate_destination(canvas:Ptr,name:*const c_char);
 questpdf_skia_canvas_annotate_destination_link(canvas:Ptr,width:f32,height:f32,name:*const c_char,description:*const c_char);
 questpdf_skia_canvas_draw_rectangle(canvas:Ptr,rect:*const Rect,paint:Ptr);
 questpdf_skia_canvas_draw_line(canvas:Ptr,start:*const Point,end:*const Point,paint:Ptr);
 questpdf_skia_canvas_clip_rectangle(canvas:Ptr,rect:*const Rect);
 questpdf_skia_canvas_clip_rounded_rectangle(canvas:Ptr,rect:*const RoundedRect);
 questpdf_skia_canvas_draw_complex_border(canvas:Ptr,inner:*const RoundedRect,outer:*const RoundedRect,paint:Ptr);
 questpdf_skia_canvas_draw_shadow(canvas:Ptr,rect:*const RoundedRect,shadow:*const BoxShadow);
 questpdf_skia_canvas_draw_image(canvas:Ptr,image:Ptr,width:f32,height:f32);
 questpdf_skia_canvas_draw_svg(canvas:Ptr,svg:Ptr,width:f32,height:f32);
 questpdf_skia_canvas_draw_svg_path(canvas:Ptr,path:*const c_char,color:u32);
 questpdf_skia_paint_create()->Ptr;
 questpdf_skia_paint_delete(paint:Ptr);
 questpdf_skia_paint_set_solid_color(paint:Ptr,color:u32);
 questpdf_skia_paint_set_stroke(paint:Ptr,thickness:f32);
 questpdf_skia_paint_set_linear_gradient(paint:Ptr,start:*const Point,end:*const Point,length:i32,colors:*const u32);
 questpdf_skia_paint_set_dashed_path_effect(paint:Ptr,length:i32,values:*const f32);
 questpdf_skia_image_create_from_data(data:Ptr)->Ptr;
 questpdf_skia_image_unref(image:Ptr);
 questpdf_skia_image_get_details(image:Ptr,details:*mut ImageDetails);
 questpdf_skia_image_resize_and_compress(image:Ptr,width:i32,height:i32,quality:i32,downsample:bool)->Ptr;
 questpdf_skia_svg_create(data:Ptr,resources:Ptr,provider:Ptr,font_manager:Ptr)->Ptr;
 questpdf_skia_svg_create_canvas(bounds:*const Rect,stream:Ptr)->Ptr;
 questpdf_skia_resource_provider_create(path:*const c_char)->Ptr;
 questpdf_skia_svg_unref(svg:Ptr);
 questpdf_skia_svg_get_size(svg:Ptr,size:*mut SvgSize,viewbox:*mut Rect);
}
// Parenthesized field calls access the ABI directly; method calls dispatch through
// the current native picture layer. This keeps native handles separate from routing.
macro_rules! canvas_methods {
    ($($name:ident($($arg:ident:$ty:ty),*);)+) => {impl Api {$(
        pub unsafe fn $name(&self,canvas:Ptr,$($arg:$ty),*) {unsafe{(self.$name)(crate::canvas::route(canvas),$($arg),*)}}
    )+}}
}
canvas_methods! {
 questpdf_skia_canvas_get_matrix9(matrix:*mut Matrix);
 questpdf_skia_canvas_draw_paragraph(paragraph:Ptr,from:i32,to:i32);
 questpdf_skia_canvas_annotate_url(width:f32,height:f32,url:*const c_char,description:*const c_char);
 questpdf_skia_canvas_annotate_destination(name:*const c_char);
 questpdf_skia_canvas_annotate_destination_link(width:f32,height:f32,name:*const c_char,description:*const c_char);
 questpdf_skia_canvas_draw_rectangle(rect:*const Rect,paint:Ptr); questpdf_skia_canvas_draw_line(start:*const Point,end:*const Point,paint:Ptr);
 questpdf_skia_canvas_clip_rectangle(rect:*const Rect); questpdf_skia_canvas_clip_rounded_rectangle(rect:*const RoundedRect);
 questpdf_skia_canvas_draw_complex_border(inner:*const RoundedRect,outer:*const RoundedRect,paint:Ptr);
 questpdf_skia_canvas_draw_shadow(rect:*const RoundedRect,shadow:*const BoxShadow);
 questpdf_skia_canvas_draw_image(image:Ptr,width:f32,height:f32); questpdf_skia_canvas_draw_svg(svg:Ptr,width:f32,height:f32);
 questpdf_skia_canvas_draw_svg_path(path:*const c_char,color:u32);
 questpdf_skia_canvas_draw_picture(picture:Ptr);
}
impl Api {
    pub unsafe fn questpdf_skia_canvas_save(&self, canvas: Ptr) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Save) {
            unsafe {
                (self.questpdf_skia_canvas_save)(crate::canvas::route(canvas));
            }
        }
    }
    pub unsafe fn questpdf_skia_canvas_restore(&self, canvas: Ptr) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Restore) {
            unsafe {
                (self.questpdf_skia_canvas_restore)(crate::canvas::route(canvas));
            }
        }
    }
    pub unsafe fn questpdf_skia_canvas_translate(&self, canvas: Ptr, x: f32, y: f32) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Translate(x, y)) {
            unsafe {
                (self.questpdf_skia_canvas_translate)(crate::canvas::route(canvas), x, y);
            }
        }
    }
    pub unsafe fn questpdf_skia_canvas_scale(&self, canvas: Ptr, x: f32, y: f32) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Scale(x, y)) {
            unsafe {
                (self.questpdf_skia_canvas_scale)(crate::canvas::route(canvas), x, y);
            }
        }
    }
    pub unsafe fn questpdf_skia_canvas_rotate(&self, canvas: Ptr, degrees: f32) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Rotate(degrees)) {
            unsafe {
                (self.questpdf_skia_canvas_rotate)(crate::canvas::route(canvas), degrees);
            }
        }
    }
    pub unsafe fn questpdf_skia_canvas_set_matrix9(&self, canvas: Ptr, matrix: *const Matrix) {
        if !crate::canvas::transform(canvas, crate::canvas::Transform::Set(unsafe { *matrix })) {
            unsafe {
                (self.questpdf_skia_canvas_set_matrix9)(crate::canvas::route(canvas), matrix);
            }
        }
    }
}
pub fn api() -> Result<Arc<Api>> {
    static API: OnceLock<Arc<Api>> = OnceLock::new();
    if let Some(a) = API.get() {
        return Ok(a.clone());
    }
    let file = if cfg!(target_os = "windows") {
        "questpdf_skia.dll"
    } else if cfg!(target_os = "macos") {
        "libquestpdf_skia.dylib"
    } else {
        "libquestpdf_skia.so"
    };
    let rid = if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "android") {
        "android"
    } else {
        "linux"
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
    let path = crate::runtime::native_path("QUESTPDF_SKIA_PATH", file, &format!("{rid}-{arch}"))?;
    let a = Arc::new(unsafe { Api::load(&path) }?);
    let version = unsafe { (a.questpdf_skia_get_compatibility_version)() };
    if version != 26 {
        bail!("Skia ABI {version}, expected 26 from the ported 2026.9.1 source")
    }
    let _ = API.set(a.clone());
    Ok(API.get().unwrap_or(&a).clone())
}
pub fn cstr(s: &str) -> CString {
    CString::new(s.replace('\0', "�")).expect("NUL replaced")
}
pub struct Handle {
    pub ptr: Ptr,
    delete: unsafe extern "C" fn(Ptr),
    _api: Arc<Api>,
}
impl Handle {
    pub unsafe fn new(ptr: Ptr, delete: unsafe extern "C" fn(Ptr), a: &Arc<Api>) -> Result<Self> {
        if ptr.is_null() {
            bail!("QuestPDF cannot instantiate native object")
        }
        Ok(Self {
            ptr,
            delete,
            _api: a.clone(),
        })
    }
    pub fn into_raw(mut self) -> Ptr {
        let ptr = self.ptr;
        self.ptr = ptr::null_mut();
        ptr
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { (self.delete)(self.ptr) }
        }
    }
}
pub fn data(a: &Arc<Api>, bytes: &[u8]) -> Result<Handle> {
    let len = i32::try_from(bytes.len()).context("native data exceeds i32")?;
    unsafe {
        Handle::new(
            (a.questpdf_skia_data_create_from_binary)(bytes.as_ptr(), len),
            a.questpdf_skia_data_unref,
            a,
        )
    }
}
#[repr(C)]
pub struct NativeFontInfo {
    pub family: [u8; 256],
    pub postscript: [u8; 256],
    pub weight: i32,
    pub italic: bool,
    pub variable: bool,
}
pub struct Fonts {
    pub api: Arc<Api>,
    pub collection: Handle,
    pub provider: Handle,
    pub unicode: Ptr,
    pub registered: std::collections::HashSet<String>,
    pub revision: u64,
    pub system: Ptr,
}
impl Fonts {
    pub fn new() -> Result<Self> {
        let a = api()?;
        unsafe {
            let provider = Handle::new(
                (a.questpdf_skia_typeface_font_provider_create)(),
                a.questpdf_skia_typeface_font_provider_unref,
                &a,
            )?;
            let collection = Handle::new(
                (a.questpdf_skia_font_collection_create)(provider.ptr, ptr::null_mut()),
                a.questpdf_skia_font_collection_unref,
                &a,
            )?;
            // The .NET engine also retains a process-global Unicode singleton.
            static UNICODE: OnceLock<usize> = OnceLock::new();
            let unicode =
                *UNICODE.get_or_init(|| (a.questpdf_skia_unicode_create)() as usize) as Ptr;
            let mut fonts = Self {
                api: a,
                collection,
                provider,
                unicode,
                registered: Default::default(),
                revision: next_font_revision(),
                system: ptr::null_mut(),
            };
            for font in DEFAULT_FONTS {
                fonts.register(font)?;
            }
            Ok(fonts)
        }
    }
    pub fn register(&mut self, bytes: &[u8]) -> Result<()> {
        let d = data(&self.api, bytes)?;
        let a = &self.api;
        if unsafe {
            (a.questpdf_skia_typeface_font_provider_add_typefaces_from_data)(
                self.provider.ptr,
                d.ptr,
                ptr::null(),
            )
        } == 0
        {
            bail!("invalid font data")
        }
        let mut array = ptr::null_mut();
        let mut length = 0;
        unsafe {
            let manager =
                (a.questpdf_skia_typeface_font_provider_as_font_manager)(self.provider.ptr);
            (a.questpdf_skia_font_manager_get_typefaces)(manager, &mut array, &mut length);
            if length > 0 {
                for font in std::slice::from_raw_parts(array, length as usize) {
                    let end = font.family.iter().position(|b| *b == 0).unwrap_or(256);
                    self.registered
                        .insert(String::from_utf8_lossy(&font.family[..end]).to_lowercase());
                }
            }
            (a.questpdf_skia_font_manager_delete_typefaces)(array);
        }
        self.revision = next_font_revision();
        Ok(())
    }
    pub fn registered_fonts(&self) -> Vec<crate::FontInfo> {
        unsafe {
            self.enumerate((self
                .api
                .questpdf_skia_typeface_font_provider_as_font_manager)(
                self.provider.ptr,
            ))
        }
    }
    pub fn system_fonts(&self) -> Vec<crate::FontInfo> {
        self.enumerate(self.system_manager())
    }
    pub fn system_manager(&self) -> Ptr {
        static MANAGER: OnceLock<usize> = OnceLock::new();
        *MANAGER.get_or_init(|| unsafe {
            (self.api.questpdf_skia_font_manager_create_system)() as usize
        }) as Ptr
    }
    fn enumerate(&self, manager: Ptr) -> Vec<crate::FontInfo> {
        if manager.is_null() {
            return vec![];
        }
        let mut array = ptr::null_mut();
        let mut length = 0;
        unsafe {
            (self.api.questpdf_skia_font_manager_get_typefaces)(manager, &mut array, &mut length);
            let decode = |bytes: &[u8]| {
                String::from_utf8_lossy(
                    &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())],
                )
                .into_owned()
            };
            let result = if length > 0 {
                std::slice::from_raw_parts(array, length as usize)
                    .iter()
                    .map(|f| crate::FontInfo {
                        family_name: decode(&f.family),
                        postscript_name: decode(&f.postscript),
                        weight: f.weight,
                        is_italic: f.italic,
                        is_variable: f.variable,
                    })
                    .collect()
            } else {
                vec![]
            };
            (self.api.questpdf_skia_font_manager_delete_typefaces)(array);
            result
        }
    }
    pub fn use_system_fonts(&mut self, enabled: bool) -> Result<()> {
        self.system = if enabled {
            self.system_manager()
        } else {
            ptr::null_mut()
        };
        self.collection = unsafe {
            Handle::new(
                (self.api.questpdf_skia_font_collection_create)(self.provider.ptr, self.system),
                self.api.questpdf_skia_font_collection_unref,
                &self.api,
            )?
        };
        self.registered = self
            .registered_fonts()
            .into_iter()
            .map(|f| f.family_name.to_lowercase())
            .collect();
        if enabled {
            self.registered.extend(
                self.system_fonts()
                    .into_iter()
                    .map(|f| f.family_name.to_lowercase()),
            );
        }
        self.revision = next_font_revision();
        Ok(())
    }
}
fn next_font_revision() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
const DEFAULT_FONTS: &[&[u8]] = &[
    include_bytes!("../resources/LatoFont/Lato-Black.ttf"),
    include_bytes!("../resources/LatoFont/Lato-BlackItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Bold.ttf"),
    include_bytes!("../resources/LatoFont/Lato-BoldItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-ExtraBold.ttf"),
    include_bytes!("../resources/LatoFont/Lato-ExtraBoldItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-ExtraLight.ttf"),
    include_bytes!("../resources/LatoFont/Lato-ExtraLightItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Italic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Light.ttf"),
    include_bytes!("../resources/LatoFont/Lato-LightItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Medium.ttf"),
    include_bytes!("../resources/LatoFont/Lato-MediumItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Regular.ttf"),
    include_bytes!("../resources/LatoFont/Lato-SemiBold.ttf"),
    include_bytes!("../resources/LatoFont/Lato-SemiBoldItalic.ttf"),
    include_bytes!("../resources/LatoFont/Lato-Thin.ttf"),
    include_bytes!("../resources/LatoFont/Lato-ThinItalic.ttf"),
];
