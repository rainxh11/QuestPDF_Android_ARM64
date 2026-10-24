//! Two-pass page generation corresponding to Drawing/DocumentGenerator.cs.
use crate::{
    elements::{Context, Environment, draw_at, paint_rect},
    native::{self, Handle, Ptr},
    *,
};
use anyhow::{Context as _, Result, bail};
use std::{cell::RefCell, path::Path, ptr};
thread_local! {static PDF_BYTES:RefCell<Option<Vec<u8>>>=const{RefCell::new(None)};}
unsafe extern "C" fn write_bytes(data: Ptr, len: i32) {
    if len <= 0 || data.is_null() {
        return;
    }
    PDF_BYTES.with(|b| {
        if let Some(out) = b.borrow_mut().as_mut() {
            out.extend_from_slice(unsafe {
                std::slice::from_raw_parts(data as *const u8, len as usize)
            })
        }
    })
}
struct OutputGuard;
#[derive(Default)]
struct OutputSettings {
    image: ImageGenerationSettings,
    svg: bool,
    image_generation: bool,
    companion: bool,
}
impl Drop for OutputGuard {
    fn drop(&mut self) {
        PDF_BYTES.with(|b| *b.borrow_mut() = None)
    }
}
pub struct Renderer {
    companion_pages: RefCell<Vec<crate::companion::PageSnapshot>>,
    group_page_counts: RefCell<Vec<usize>>,
    fonts: native::Fonts,
    sections: RefCell<std::collections::HashMap<String, (usize, usize)>>,
    captured_positions: RefCell<Vec<PageElementLocation>>,
    semantic_tree: RefCell<Option<crate::semantic::SemanticTree>>,
}
impl Renderer {
    pub fn new() -> Result<Self> {
        let mut fonts = native::Fonts::new()?;
        crate::font_manager::apply(&mut fonts)?;
        Ok(Self {
            companion_pages: RefCell::new(vec![]),
            group_page_counts: RefCell::new(vec![]),
            fonts,
            sections: RefCell::new(std::collections::HashMap::new()),
            captured_positions: RefCell::new(vec![]),
            semantic_tree: RefCell::new(None),
        })
    }
    pub fn register_font(&mut self, bytes: &[u8]) -> Result<()> {
        self.fonts.register(bytes)
    }
    pub fn captured_positions(&self) -> Vec<PageElementLocation> {
        self.captured_positions.borrow().clone()
    }
    pub fn registered_fonts(&self) -> Vec<FontInfo> {
        self.fonts.registered_fonts()
    }
    pub fn use_system_fonts(&mut self, enabled: bool) -> Result<()> {
        self.fonts.use_system_fonts(enabled)
    }
    pub fn register_font_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.register_font(&std::fs::read(crate::runtime::resolve_resource_file_path(
            path,
        )?)?)
    }
    pub fn generate(&mut self, doc: &Document) -> Result<Vec<u8>> {
        crate::settings::validate_license()?;
        let semantic = matches!(
            doc.settings.pdfa_conformance,
            PdfAConformance::PdfA2A | PdfAConformance::PdfA3A
        ) || doc.settings.pdfua_conformance != PdfUaConformance::None;
        self.semantic_tree.replace(if semantic {
            Some(crate::semantic::SemanticTree::new())
        } else {
            None
        });
        for page in &doc.container.pages {
            for element in [
                &page.header,
                &page.content,
                &page.footer,
                &page.background,
                &page.foreground,
            ] {
                element.clear_semantics();
            }
        }
        self.sections.borrow_mut().clear();
        self.captured_positions.borrow_mut().clear();
        prepare_caching(doc);
        if doc.container.pages.is_empty() {
            bail!("Document must contain at least one page")
        }
        reset(doc);
        let count = self.run(doc, 0, ptr::null_mut(), None, &OutputSettings::default())?;
        native::trace("dry pass complete");
        reset(doc);
        let a = &self.fonts.api;
        let semantic_tree = self
            .semantic_tree
            .borrow()
            .as_ref()
            .map(|tree| tree.native(a))
            .transpose()?;
        let strings: [&str; 7] = [
            &doc.metadata.title,
            &doc.metadata.author,
            &doc.metadata.subject,
            &doc.metadata.keywords,
            &doc.metadata.creator,
            if Settings::get().license == Some(LicenseType::Evaluation) {
                "QuestPDF (Evaluation Mode)"
            } else if doc.metadata.producer.is_empty() {
                "QuestPDF Rust"
            } else {
                &doc.metadata.producer
            },
            &doc.metadata.language,
        ];
        let strings = strings.map(native::cstr);
        let now = chrono::Local::now().fixed_offset();
        let meta = native::PdfMetadata {
            title: strings[0].as_ptr() as Ptr,
            author: strings[1].as_ptr() as Ptr,
            subject: strings[2].as_ptr() as Ptr,
            keywords: strings[3].as_ptr() as Ptr,
            creator: strings[4].as_ptr() as Ptr,
            producer: strings[5].as_ptr() as Ptr,
            language: strings[6].as_ptr() as Ptr,
            created: native_date(doc.metadata.creation_date.unwrap_or(now)),
            modified: native_date(doc.metadata.modified_date.unwrap_or(now)),
            compress: doc.settings.compress,
            dpi: doc.settings.raster_dpi,
            pdfa: doc.settings.pdfa_conformance as i32,
            pdfua: doc.settings.pdfua_conformance as i32,
            semantic_root: semantic_tree
                .as_ref()
                .map(|tree| tree.root.ptr)
                .unwrap_or(ptr::null_mut()),
        };
        if PDF_BYTES.with(|b| b.borrow().is_some()) {
            bail!("Nested PDF generation on the same thread is not supported")
        }
        PDF_BYTES.with(|b| *b.borrow_mut() = Some(Vec::new()));
        let _guard = OutputGuard;
        unsafe {
            let stream = Handle::new(
                (a.questpdf_skia_write_stream_create)(write_bytes),
                a.questpdf_skia_write_stream_delete,
                a,
            )?;
            native::trace("write stream created");
            let pdf = Handle::new(
                (a.questpdf_skia_pdf_document_create)(stream.ptr, &meta),
                a.questpdf_skia_document_unref,
                a,
            )?;
            native::trace("PDF document created");
            self.run(doc, count, pdf.ptr, None, &OutputSettings::default())?;
            (a.questpdf_skia_document_close)(pdf.ptr);
            (a.questpdf_skia_write_stream_flush)(stream.ptr);
        }
        let result = PDF_BYTES.with(|b| b.borrow_mut().take().unwrap_or_default());
        if !result.starts_with(b"%PDF-") {
            bail!("Skia did not produce a PDF")
        };
        Ok(result)
    }
    pub fn generate_images(&mut self, doc: &Document, dpi: f32) -> Result<Vec<Vec<u8>>> {
        self.generate_images_with_settings(
            doc,
            &ImageGenerationSettings {
                raster_dpi: dpi,
                ..Default::default()
            },
        )
    }
    pub fn generate_images_with_settings(
        &mut self,
        doc: &Document,
        settings: &ImageGenerationSettings,
    ) -> Result<Vec<Vec<u8>>> {
        crate::settings::validate_license()?;
        self.sections.borrow_mut().clear();
        self.semantic_tree.replace(None);
        self.captured_positions.borrow_mut().clear();
        if !settings.raster_dpi.is_finite() || settings.raster_dpi <= 0. {
            bail!("Raster DPI must be positive and finite")
        }
        let output = OutputSettings {
            image: settings.clone(),
            svg: false,
            image_generation: true,
            companion: false,
        };
        prepare_caching(doc);
        reset(doc);
        let count = self.run(doc, 0, ptr::null_mut(), None, &output)?;
        reset(doc);
        let mut images = vec![];
        self.run(doc, count, ptr::null_mut(), Some(&mut images), &output)?;
        Ok(images)
    }
    pub fn generate_svg(&mut self, doc: &Document) -> Result<Vec<String>> {
        if PDF_BYTES.with(|b| b.borrow().is_some()) {
            bail!("Nested output generation on the same thread is not supported");
        }
        crate::settings::validate_license()?;
        self.sections.borrow_mut().clear();
        self.semantic_tree.replace(None);
        self.captured_positions.borrow_mut().clear();
        prepare_caching(doc);
        reset(doc);
        let output = OutputSettings {
            svg: true,
            ..Default::default()
        };
        let count = self.run(doc, 0, ptr::null_mut(), None, &output)?;
        reset(doc);
        PDF_BYTES.with(|b| *b.borrow_mut() = Some(vec![]));
        let _guard = OutputGuard;
        let mut pages = vec![];
        self.run(doc, count, ptr::null_mut(), Some(&mut pages), &output)?;
        pages
            .into_iter()
            .map(|p| String::from_utf8(p).map_err(Into::into))
            .collect()
    }
    pub fn generate_companion_snapshot(
        &mut self,
        doc: &Document,
    ) -> Result<crate::companion::DocumentSnapshot> {
        crate::settings::validate_license()?;
        anyhow::ensure!(
            doc.page_number_groups.is_none(),
            "The QuestPDF Companion does not support merged documents"
        );
        Settings::configure(|s| {
            s.enable_caching = false;
            s.enable_detailed_layout_errors = true;
        });
        let mut document = Document::from_document(doc);
        document.settings = DocumentSettings::default();
        self.sections.borrow_mut().clear();
        self.semantic_tree.replace(None);
        self.captured_positions.borrow_mut().clear();
        self.companion_pages.borrow_mut().clear();
        for page in &document.container.pages {
            for element in [
                &page.header,
                &page.content,
                &page.footer,
                &page.background,
                &page.foreground,
            ] {
                element.clear_semantics();
            }
        }
        prepare_caching(&document);
        reset(&document);
        let count = self.run(
            &document,
            0,
            ptr::null_mut(),
            None,
            &OutputSettings::default(),
        )?;
        reset(&document);
        self.run(
            &document,
            count,
            ptr::null_mut(),
            None,
            &OutputSettings {
                companion: true,
                ..Default::default()
            },
        )?;
        let pages = self.companion_pages.take();
        let mut hierarchy_pages = vec![];
        for (index, page) in document.container.pages.iter().enumerate() {
            let children = [
                ("Background", &page.background),
                ("Header", &page.header),
                ("Content", &page.content),
                ("Footer", &page.footer),
                ("Foreground", &page.foreground),
            ]
            .into_iter()
            .map(|(name, element)| {
                crate::companion::HierarchyElement::group(name, vec![element.hierarchy()])
            })
            .collect();
            let mut node = crate::companion::HierarchyElement::group("Page", children);
            node.page_locations = pages
                .iter()
                .enumerate()
                .filter(|(_, p)| p.template_index == index)
                .map(|(index, p)| crate::companion::PageLocation {
                    page_number: index + 1,
                    left: 0.,
                    top: 0.,
                    right: p.size.width,
                    bottom: p.size.height,
                })
                .collect();
            hierarchy_pages.push(node);
        }
        Ok(crate::companion::DocumentSnapshot {
            pages,
            hierarchy: crate::companion::HierarchyElement::group("Document", hierarchy_pages),
        })
    }
    fn run(
        &self,
        doc: &Document,
        total: usize,
        pdf: Ptr,
        mut images: Option<&mut Vec<Vec<u8>>>,
        output: &OutputSettings,
    ) -> Result<usize> {
        let mut page_number = 0;
        let mut logical_page = 0;
        let mut group_index = 0;
        let mut group_start = 0;
        let mut group_counts = vec![0; doc.page_number_groups.as_ref().map_or(1, Vec::len)];
        let dry_pass = total == 0;
        let mut managed_matrix = native::Matrix {
            scale_x: 1.,
            scale_y: 1.,
            perspective3: 1.,
            ..Default::default()
        };
        for (page_index, page) in doc.container.pages.iter().enumerate() {
            if let Some(groups) = &doc.page_number_groups {
                while group_index + 1 < groups.len()
                    && page_index >= group_start + groups[group_index]
                {
                    group_start += groups[group_index];
                    group_index += 1;
                    if !doc.continuous_page_numbers {
                        logical_page = 0;
                    }
                }
            }
            let mut header_drawn = false;
            let mut footer_drawn = false;
            let env = Environment {
                style: TextStyle::library_default().merge(&page.style),
                // Page.Compose supplies a direction setter even for default LTR.
                direction: page.direction,
            };
            loop {
                page_number += 1;
                logical_page += 1;
                group_counts[group_index] += 1;
                if page_number > doc.settings.max_pages {
                    bail!(
                        "Document exceeded {} pages; content cannot make progress",
                        doc.settings.max_pages
                    )
                }
                let ctx = Context {
                    collect_layout: output.companion,
                    document_id: doc.page_number_groups.as_ref().map(|_| group_index),
                    fonts: &self.fonts,
                    page: logical_page,
                    total: if total > 0 && !doc.continuous_page_numbers {
                        self.group_page_counts.borrow()[group_index]
                    } else {
                        total
                    },
                    sections: &self.sections,
                    captured_positions: &self.captured_positions,
                    semantic_tree: &self.semantic_tree,
                    page_size: std::cell::Cell::new(Size::ZERO),
                    in_footer: std::cell::Cell::new(false),
                    image_dpi: if output.image_generation {
                        output.image.raster_dpi
                    } else {
                        doc.settings.raster_dpi
                    },
                    image_quality: doc.settings.image_compression_quality,
                    use_original_images: output.image_generation,
                    throw_on_missing_glyphs: doc.settings.throw_on_missing_glyphs
                        && Settings::get().throw_on_missing_text_glyphs,
                    throw_on_missing_fonts: doc.settings.throw_on_missing_fonts
                        && Settings::get().throw_on_missing_font_families,
                };
                page.header.reset(true);
                page.footer.reset(true);
                page.background.reset(true);
                page.foreground.reset(true);
                let [left, top, right, bottom] = page.margins;
                let flexible = page.min_size.is_some() || page.max_size.is_some();
                let min = if flexible {
                    page.min_size.unwrap_or(Size::ZERO)
                } else {
                    page.size
                };
                let max = if flexible {
                    page.max_size.unwrap_or(Size::MAX)
                } else {
                    page.size
                };
                let min = Size::new(min.width.max(3.), min.height.max(3.));
                let max = Size::new(max.width.min(INFINITY), max.height.min(INFINITY));
                if min.width > max.width || min.height > max.height {
                    bail!("Page minimum size exceeds maximum size");
                }
                let fixed_width = min.width == max.width;
                let fixed_height = min.height == max.height;
                let plan = |space: Size| -> Result<(SpacePlan, SpacePlan, SpacePlan)> {
                    let head = page.header.measure(space, &ctx, &env)?;
                    let foot = page.footer.measure(space, &ctx, &env)?;
                    if !head.done() || !foot.done() {
                        bail!("Page header or footer does not fit on page {page_number}");
                    }
                    let content_space = Size::new(
                        space.width,
                        space.height - head.size.height - foot.size.height,
                    );
                    let content = page
                        .content
                        .measure(content_space, &ctx, &env)
                        .with_context(|| format!("measure page {page_number}"))?;
                    if content.kind == SpacePlanType::Wrap {
                        let detail = if Settings::get().enable_detailed_layout_errors {
                            format!("\n{}", page.content.debug_tree(0))
                        } else {
                            String::new()
                        };
                        bail!(
                            "Document layout cannot fit on page {page_number} (template {page_index}, area {content_space:?}).{detail}"
                        );
                    }
                    Ok((head, content, foot))
                };
                let maximum_space = Size::new(max.width - left - right, max.height - top - bottom);
                let (head, content, foot) = plan(maximum_space)?;
                let width = if fixed_width {
                    maximum_space.width
                } else {
                    head.size.width.max(content.size.width).max(foot.size.width)
                };
                let content_height = if fixed_height {
                    maximum_space.height - head.size.height - foot.size.height
                } else {
                    content.size.height
                };
                let page_size = Size::new(
                    (left + width + right).max(min.width),
                    (top + head.size.height + content_height + foot.size.height + bottom)
                        .max(min.height),
                );
                let available = Size::new(
                    page_size.width - left - right,
                    page_size.height - top - bottom,
                );
                let (head, content, foot) = plan(available)?;
                let content_height = if fixed_height {
                    available.height - head.size.height - foot.size.height
                } else {
                    content.size.height
                };
                let width = if fixed_width {
                    available.width
                } else {
                    head.size.width.max(content.size.width).max(foot.size.width)
                };
                let horizontal_offset = if env.direction == ContentDirection::RightToLeft {
                    available.width - width
                } else {
                    0.
                };
                ctx.page_size.set(page_size);
                let mut bitmap = None;
                let mut bitmap_canvas = None;
                let mut svg_canvas = None;
                let mut svg_stream = None;
                let target_canvas = if images.is_some() && output.svg {
                    let a = &self.fonts.api;
                    PDF_BYTES.with(|b| b.borrow_mut().as_mut().unwrap().clear());
                    unsafe {
                        let stream = Handle::new(
                            (a.questpdf_skia_write_stream_create)(write_bytes),
                            a.questpdf_skia_write_stream_delete,
                            a,
                        )?;
                        let canvas = Handle::new(
                            (a.questpdf_skia_svg_create_canvas)(
                                &native::Rect {
                                    left: 0.,
                                    top: 0.,
                                    right: page_size.width,
                                    bottom: page_size.height,
                                },
                                stream.ptr,
                            ),
                            a.questpdf_skia_canvas_delete,
                            a,
                        )?;
                        let result = canvas.ptr;
                        svg_canvas = Some(canvas);
                        svg_stream = Some(stream);
                        result
                    }
                } else if images.is_some() {
                    let a = &self.fonts.api;
                    let scale = output.image.raster_dpi / 72.;
                    let width = (page_size.width * scale) as i32;
                    let height = (page_size.height * scale) as i32;
                    if width <= 0 || height <= 0 || (width as i64) * (height as i64) > 500_000_000 {
                        bail!("Raster dimensions out of range: {width}x{height}")
                    }
                    unsafe {
                        let b = Handle::new(
                            (a.questpdf_skia_bitmap_create)(width, height),
                            a.questpdf_skia_bitmap_delete,
                            a,
                        )?;
                        let cc = Handle::new(
                            (a.questpdf_skia_canvas_create_from_bitmap)(b.ptr),
                            a.questpdf_skia_canvas_delete,
                            a,
                        )?;
                        let cp = cc.ptr;
                        a.questpdf_skia_canvas_scale(cp, scale, scale);
                        bitmap = Some(b);
                        bitmap_canvas = Some(cc);
                        if !output.image.use_transparent_background
                            || output.image.image_format == ImageFormat::Jpeg
                        {
                            paint_rect(cp, page_size, Color::WHITE, None, &ctx)?;
                        }
                        cp
                    }
                } else if pdf.is_null() {
                    ptr::null_mut()
                } else {
                    unsafe {
                        (self.fonts.api.questpdf_skia_document_begin_page)(
                            pdf,
                            page_size.width,
                            page_size.height,
                        )
                    }
                };
                if !pdf.is_null() && target_canvas.is_null() {
                    bail!("Cannot begin native PDF page")
                }
                let session = crate::canvas::CanvasSession::new(&self.fonts.api, page_size)?;
                if dry_pass {
                    session.enable_managed_matrix(managed_matrix);
                }
                let canvas = session.ptr;
                if !pdf.is_null() {
                    native::trace("PDF page begun")
                }
                crate::canvas::set_z_index(canvas, i32::MIN)?;
                {
                    let _scope = crate::semantic::Scope::artifact(&ctx, canvas, -8);
                    paint_rect(canvas, page_size, page.color, None, &ctx)?;
                }
                crate::canvas::set_z_index(canvas, 0)?;
                {
                    let _scope = crate::semantic::Scope::artifact(&ctx, canvas, -8);
                    page.background.draw(page_size, canvas, &ctx, &env)?;
                }
                if !header_drawn {
                    page.header.retain_lazy_content();
                }
                {
                    let _scope = if header_drawn && self.semantic_tree.borrow().is_some() {
                        Some(crate::semantic::Scope::artifact(&ctx, canvas, -3))
                    } else {
                        None
                    };
                    draw_at(
                        &page.header,
                        Size::new(width, head.size.height),
                        left + horizontal_offset,
                        top,
                        canvas,
                        &ctx,
                        &env,
                    )?;
                }
                if head.kind == SpacePlanType::FullRender {
                    header_drawn = true;
                }
                draw_at(
                    &page.content,
                    Size::new(width, content_height),
                    left + horizontal_offset,
                    top + head.size.height,
                    canvas,
                    &ctx,
                    &env,
                )?;
                ctx.in_footer.set(true);
                if !footer_drawn {
                    page.footer.retain_lazy_content();
                }
                {
                    let _scope = if footer_drawn && self.semantic_tree.borrow().is_some() {
                        Some(crate::semantic::Scope::artifact(&ctx, canvas, -4))
                    } else {
                        None
                    };
                    draw_at(
                        &page.footer,
                        Size::new(width, foot.size.height),
                        left + horizontal_offset,
                        top + head.size.height + content_height,
                        canvas,
                        &ctx,
                        &env,
                    )?;
                }
                if foot.kind == SpacePlanType::FullRender {
                    footer_drawn = true;
                }
                ctx.in_footer.set(false);
                {
                    let _scope = crate::semantic::Scope::artifact(&ctx, canvas, -5);
                    page.foreground.draw(page_size, canvas, &ctx, &env)?;
                }
                if dry_pass {
                    unsafe {
                        self.fonts
                            .api
                            .questpdf_skia_canvas_get_matrix9(canvas, &mut managed_matrix);
                    }
                }
                let layers = session.finish()?;
                if output.companion {
                    self.companion_pages
                        .borrow_mut()
                        .push(crate::companion::PageSnapshot::new(
                            page_size,
                            page_index,
                            layers,
                            &self.fonts.api,
                        )?);
                } else {
                    if !target_canvas.is_null() {
                        unsafe {
                            for layer in layers {
                                self.fonts
                                    .api
                                    .questpdf_skia_canvas_draw_picture(target_canvas, layer.ptr);
                            }
                        }
                    }
                }
                if !pdf.is_null() {
                    unsafe { (self.fonts.api.questpdf_skia_document_end_page)(pdf) }
                }
                if let (Some(canvas), Some(stream), Some(images)) =
                    (svg_canvas, svg_stream, images.as_deref_mut())
                {
                    drop(canvas);
                    unsafe {
                        (self.fonts.api.questpdf_skia_write_stream_flush)(stream.ptr);
                    }
                    let bytes =
                        PDF_BYTES.with(|b| std::mem::take(b.borrow_mut().as_mut().unwrap()));
                    images.push(bytes);
                }
                if let (Some(b), Some(images)) = (bitmap, images.as_deref_mut()) {
                    let a = &self.fonts.api;
                    unsafe {
                        drop(bitmap_canvas);
                        let data = Handle::new(
                            match output.image.image_format {
                                ImageFormat::Png => (a.questpdf_skia_bitmap_encode_as_png)(b.ptr),
                                ImageFormat::Jpeg => (a.questpdf_skia_bitmap_encode_as_jpg)(
                                    b.ptr,
                                    output.image.image_compression_quality.quality(),
                                ),
                                ImageFormat::Webp => (a.questpdf_skia_bitmap_encode_as_webp)(
                                    b.ptr,
                                    output.image.image_compression_quality.quality(),
                                ),
                            },
                            a.questpdf_skia_data_unref,
                            a,
                        )?;
                        let mut bytes = native::DataBytes::default();
                        (a.questpdf_skia_data_get_bytes)(data.ptr, &mut bytes);
                        if bytes.length < 0 || bytes.bytes.is_null() {
                            bail!("Invalid native image output")
                        };
                        images.push(
                            std::slice::from_raw_parts(bytes.bytes, bytes.length as usize).to_vec(),
                        )
                    }
                }
                if content.done() {
                    break;
                }
            }
        }
        if total == 0 {
            self.group_page_counts.replace(group_counts);
        }
        Ok(page_number)
    }
}
fn prepare_caching(doc: &Document) {
    for page in &doc.container.pages {
        for element in [
            &page.header,
            &page.content,
            &page.footer,
            &page.background,
            &page.foreground,
        ] {
            if Settings::get().enable_caching {
                element.prepare_caching();
            } else {
                element.clear_snapshots();
            }
        }
    }
}
fn reset(doc: &Document) {
    for p in &doc.container.pages {
        for e in [
            &p.header,
            &p.content,
            &p.footer,
            &p.background,
            &p.foreground,
        ] {
            e.reset(false)
        }
    }
}
fn native_date(date: chrono::DateTime<chrono::FixedOffset>) -> native::DateTime {
    use chrono::{Datelike, Timelike};
    native::DateTime {
        zone: (date.offset().local_minus_utc() / 60) as i16,
        year: date.year() as u16,
        month: date.month() as u8,
        weekday: date.weekday().num_days_from_sunday() as u8,
        day: date.day() as u8,
        hour: date.hour() as u8,
        minute: date.minute() as u8,
        second: date.second() as u8,
    }
}
