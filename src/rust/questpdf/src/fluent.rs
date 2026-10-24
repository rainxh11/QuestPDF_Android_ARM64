//! Fluent descriptors mirror QuestPDF.Fluent with idiomatic snake_case names.
use crate::{Renderer, SemanticInfo, elements::*, infrastructure::*};
use anyhow::Result;
use std::path::Path;
#[derive(Clone, Default)]
pub struct DocumentContainer {
    pub pages: Vec<Page>,
}
impl DocumentContainer {
    pub fn page(&mut self, compose: impl FnOnce(&mut Page)) {
        let mut p = Page::default();
        compose(&mut p);
        self.pages.push(p)
    }
}
pub struct Document {
    pub container: DocumentContainer,
    pub metadata: DocumentMetadata,
    pub settings: DocumentSettings,
    pub(crate) page_number_groups: Option<Vec<usize>>,
    pub(crate) continuous_page_numbers: bool,
}
impl Document {
    pub fn create(compose: impl FnOnce(&mut DocumentContainer)) -> Self {
        let mut c = DocumentContainer::default();
        compose(&mut c);
        Self {
            container: c,
            metadata: DocumentMetadata::default(),
            settings: DocumentSettings::default(),
            page_number_groups: None,
            continuous_page_numbers: true,
        }
    }
    pub fn from_document(doc: &impl IDocument) -> Self {
        let mut d = Self::create(|c| doc.compose(c));
        d.metadata = doc.metadata();
        d.settings = doc.settings();
        d.page_number_groups = doc.page_number_groups();
        d.continuous_page_numbers = doc.continuous_page_numbers();
        d
    }
    pub fn with_metadata(mut self, metadata: DocumentMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    pub fn merge(documents: impl IntoIterator<Item = Document>) -> MergedDocument {
        let mut document = Self::create(|_| {});
        let mut groups = vec![];
        for part in documents {
            groups.push(part.container.pages.len());
            document.container.pages.extend(part.container.pages);
        }
        document.page_number_groups = Some(groups);
        document.continuous_page_numbers = false;
        MergedDocument(document)
    }
    pub fn with_settings(mut self, settings: DocumentSettings) -> Self {
        self.settings = settings;
        self
    }
    pub fn generate_pdf(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.generate_pdf_bytes()?)?;
        Ok(())
    }
    pub fn generate_pdf_bytes(&self) -> Result<Vec<u8>> {
        Renderer::new()?.generate(self)
    }
    pub fn generate_pdf_to(&self, mut stream: impl std::io::Write) -> Result<()> {
        stream.write_all(&self.generate_pdf_bytes()?)?;
        Ok(())
    }
    pub fn generate_images_to(
        &self,
        mut path: impl FnMut(usize) -> std::path::PathBuf,
        settings: &ImageGenerationSettings,
    ) -> Result<()> {
        for (index, image) in self
            .generate_images_with_settings(settings)?
            .into_iter()
            .enumerate()
        {
            std::fs::write(path(index), image)?;
        }
        Ok(())
    }
    pub fn generate_pdf_and_show(&self) -> Result<std::path::PathBuf> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let number = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = crate::Settings::get()
            .temporary_storage_path
            .unwrap_or_else(std::env::temp_dir)
            .join(format!(
                "QuestPDF Preview {} {number}.pdf",
                std::process::id()
            ));
        self.generate_pdf(&path)?;
        #[cfg(windows)]
        std::process::Command::new("explorer.exe")
            .arg(&path)
            .spawn()?;
        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(&path).spawn()?;
        #[cfg(all(unix, not(target_os = "macos")))]
        std::process::Command::new("xdg-open").arg(&path).spawn()?;
        Ok(path)
    }
    pub fn generate_images(&self, dpi: f32) -> Result<Vec<Vec<u8>>> {
        Renderer::new()?.generate_images(self, dpi)
    }
    pub fn generate_images_with_settings(
        &self,
        settings: &ImageGenerationSettings,
    ) -> Result<Vec<Vec<u8>>> {
        Renderer::new()?.generate_images_with_settings(self, settings)
    }
    pub fn generate_svg(&self) -> Result<Vec<String>> {
        Renderer::new()?.generate_svg(self)
    }
}
impl IDocument for Document {
    fn compose(&self, container: &mut DocumentContainer) {
        container.pages.extend(self.container.pages.iter().cloned());
    }
    fn metadata(&self) -> DocumentMetadata {
        self.metadata.clone()
    }
    fn settings(&self) -> DocumentSettings {
        self.settings.clone()
    }
    fn page_number_groups(&self) -> Option<Vec<usize>> {
        self.page_number_groups.clone()
    }
    fn continuous_page_numbers(&self) -> bool {
        self.continuous_page_numbers
    }
}
pub struct MergedDocument(Document);
impl MergedDocument {
    pub fn use_original_page_numbers(mut self) -> Self {
        self.0.continuous_page_numbers = false;
        self
    }
    pub fn use_continuous_page_numbers(mut self) -> Self {
        self.0.continuous_page_numbers = true;
        self
    }
    pub fn with_metadata(mut self, metadata: DocumentMetadata) -> Self {
        self.0.metadata = metadata;
        self
    }
    pub fn with_settings(mut self, settings: DocumentSettings) -> Self {
        self.0.settings = settings;
        self
    }
}
impl std::ops::Deref for MergedDocument {
    type Target = Document;
    fn deref(&self) -> &Document {
        &self.0
    }
}
impl IDocument for MergedDocument {
    fn compose(&self, container: &mut DocumentContainer) {
        self.0.compose(container);
    }
    fn metadata(&self) -> DocumentMetadata {
        self.0.metadata.clone()
    }
    fn settings(&self) -> DocumentSettings {
        self.0.settings.clone()
    }
    fn page_number_groups(&self) -> Option<Vec<usize>> {
        self.0.page_number_groups.clone()
    }
    fn continuous_page_numbers(&self) -> bool {
        self.0.continuous_page_numbers
    }
}
impl Page {
    pub fn size(&mut self, size: Size) {
        self.size = size;
        self.min_size = Some(size);
        self.max_size = Some(size);
        self.continuous = false
    }
    pub fn min_size(&mut self, size: Size) {
        self.min_size = Some(size);
    }
    pub fn max_size(&mut self, size: Size) {
        self.max_size = Some(size);
    }
    pub fn continuous_size(&mut self, width: f32, unit: Unit) {
        self.size = Size::new(unit.points(width), INFINITY);
        self.min_size = Some(Size::new(unit.points(width), 0.));
        self.max_size = Some(self.size);
        self.continuous = true
    }
    pub fn landscape(&mut self) {
        self.size = Size::new(self.size.height, self.size.width);
        self.min_size = self.min_size.map(|s| Size::new(s.height, s.width));
        self.max_size = self.max_size.map(|s| Size::new(s.height, s.width));
    }
    pub fn margin(&mut self, value: f32) {
        self.margins = [value; 4]
    }
    pub fn margin_unit(&mut self, value: f32, unit: Unit) {
        self.margin(unit.points(value))
    }
    pub fn margin_horizontal(&mut self, value: f32) {
        self.margins[0] = value;
        self.margins[2] = value
    }
    pub fn margin_vertical(&mut self, value: f32) {
        self.margins[1] = value;
        self.margins[3] = value
    }
    pub fn margin_bottom(&mut self, value: f32) {
        self.margins[3] = value
    }
    pub fn margin_left(&mut self, value: f32) {
        self.margins[0] = value;
    }
    pub fn margin_right(&mut self, value: f32) {
        self.margins[2] = value;
    }
    pub fn margin_top(&mut self, value: f32) {
        self.margins[1] = value
    }
    pub fn default_text_style(&mut self, f: impl FnOnce(TextStyle) -> TextStyle) {
        self.style = f(TextStyle::default())
    }
    pub fn content_from_right_to_left(&mut self) {
        self.direction = ContentDirection::RightToLeft;
        self.direction_override = Some(self.direction);
    }
    pub fn content_from_left_to_right(&mut self) {
        self.direction = ContentDirection::LeftToRight;
        self.direction_override = Some(self.direction);
    }
    pub fn page_color(&mut self, v: Color) {
        self.color = v
    }
    pub fn header(&mut self) -> Container {
        Container(self.header.clone())
    }
    pub fn content(&mut self) -> Container {
        Container(self.content.clone())
    }
    pub fn footer(&mut self) -> Container {
        Container(self.footer.clone())
    }
    pub fn background(&mut self) -> Container {
        Container(self.background.clone())
    }
    pub fn foreground(&mut self) -> Container {
        Container(self.foreground.clone())
    }
}
#[derive(Clone)]
pub struct Container(pub Element);
impl Container {
    pub fn semantic_header1(&self) -> Self {
        self.semantic_heading(1)
    }
    pub fn semantic_header2(&self) -> Self {
        self.semantic_heading(2)
    }
    pub fn semantic_header3(&self) -> Self {
        self.semantic_heading(3)
    }
    pub fn semantic_header4(&self) -> Self {
        self.semantic_heading(4)
    }
    pub fn semantic_header5(&self) -> Self {
        self.semantic_heading(5)
    }
    pub fn semantic_header6(&self) -> Self {
        self.semantic_heading(6)
    }
    pub fn external_link(&self, url: impl Into<String>) -> Self {
        self.hyperlink(url)
    }
    pub fn internal_link(&self, name: impl Into<String>) -> Self {
        self.section_link(name)
    }
    pub fn location(&self, name: impl Into<String>) -> Self {
        self.section(name)
    }
    pub fn minimal_box(&self) -> Self {
        self.shrink()
    }
    pub fn stack(&self, compose: impl FnOnce(&mut ColumnDescriptor)) {
        self.column(compose);
    }
    pub fn new() -> Self {
        Self(Element::empty())
    }
    #[track_caller]
    pub fn element(&self, compose: impl FnOnce(Container)) {
        let location = std::panic::Location::caller();
        compose(self.debug_pointer(format!("{}:{}", location.file(), location.line())))
    }
    pub fn container(&self) -> Self {
        self.wrap(Kind::Proxy)
    }
    pub fn debug_pointer(&self, label: impl Into<String>) -> Self {
        let child = self.container();
        self.0.set_debug_label(label.into());
        child
    }
    pub fn debug_area(&self, text: impl Into<String>, color: Option<Color>) -> Self {
        let text = text.into();
        let color = color.unwrap_or(crate::colors::red::MEDIUM);
        let child = Self::new();
        self.border(1.).border_color(color).layers(|layers| {
            layers.primary_layer().0.set(Kind::Proxy(child.0.clone()));
            layers.layer().background(color.with_alpha(64));
            if !text.trim().is_empty() {
                layers
                    .layer()
                    .align_center()
                    .shrink()
                    .background(Color::WHITE)
                    .padding(2.)
                    .text(text)
                    .font_color(color)
                    .font_size(8.);
            }
        });
        child
    }
    pub fn show_if_predicate(&self, predicate: impl Fn(ShowIfContext) -> bool + 'static) -> Self {
        self.wrap(|c| Kind::ShowIfPredicate(std::rc::Rc::new(predicate), c))
    }
    pub fn component(&self, c: &impl IComponent) {
        c.compose(
            self.container()
                .debug_pointer(std::any::type_name_of_val(c)),
        );
    }
    pub fn dynamic(&self, component: impl crate::IDynamicComponent) {
        self.0.set(Kind::Dynamic(crate::dynamic::host(component)));
    }
    pub fn dynamic_element(&self, element: &crate::DynamicElement) {
        self.0.set(Kind::Proxy(element.element.clone()));
    }
    pub fn section(&self, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty());
        self.wrap(|c| Kind::Section(name, c))
    }
    pub fn z_index(&self, index: i32) -> Self {
        self.wrap(|c| Kind::ZIndex(index, c))
    }
    pub fn capture_content_position(&self, id: impl Into<String>) -> Self {
        self.wrap(|c| Kind::CapturePosition(id.into(), c))
    }
    pub fn semantic_tag(&self, info: SemanticInfo) -> Self {
        self.wrap(|c| Kind::Semantic(info, c))
    }
    pub fn semantic_ignore(&self) -> Self {
        self.wrap(|c| Kind::Artifact(-1, c))
    }
    pub fn semantic_article(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Art"))
    }
    pub fn semantic_section(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Sect"))
    }
    pub fn semantic_division(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Div"))
    }
    pub fn semantic_block_quotation(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("BlockQuote"))
    }
    pub fn semantic_caption(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Caption"))
    }
    pub fn semantic_index(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Index"))
    }
    pub fn semantic_language(&self, language: impl Into<String>) -> Self {
        let mut info = SemanticInfo::new("NonStruct");
        info.language = Some(language.into());
        self.semantic_tag(info)
    }
    pub fn semantic_table_of_contents(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("TOC"))
    }
    pub fn semantic_table_of_contents_item(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("TOCI"))
    }
    pub fn semantic_heading(&self, level: usize) -> Self {
        assert!((1..=6).contains(&level));
        self.semantic_tag(SemanticInfo::new(format!("H{level}")))
    }
    pub fn semantic_heading1(&self) -> Self {
        self.semantic_heading(1)
    }
    pub fn semantic_heading2(&self) -> Self {
        self.semantic_heading(2)
    }
    pub fn semantic_heading3(&self) -> Self {
        self.semantic_heading(3)
    }
    pub fn semantic_heading4(&self) -> Self {
        self.semantic_heading(4)
    }
    pub fn semantic_heading5(&self) -> Self {
        self.semantic_heading(5)
    }
    pub fn semantic_heading6(&self) -> Self {
        self.semantic_heading(6)
    }
    pub fn semantic_paragraph(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("P"))
    }
    pub fn semantic_list(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("L"))
    }
    pub fn semantic_list_item(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("LI"))
    }
    pub fn semantic_list_label(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Lbl"))
    }
    pub fn semantic_list_item_body(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("LBody"))
    }
    pub fn semantic_table(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Table"))
    }
    pub fn semantic_span(&self, alternative: Option<String>) -> Self {
        let mut info = SemanticInfo::new("Span");
        info.alternative_text = alternative;
        self.semantic_tag(info)
    }
    pub fn semantic_quote(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Quote"))
    }
    pub fn semantic_code(&self) -> Self {
        self.semantic_tag(SemanticInfo::new("Code"))
    }
    pub fn semantic_link(&self, alternative: impl Into<String>) -> Self {
        let mut info = SemanticInfo::new("Link");
        info.alternative_text = Some(alternative.into());
        self.semantic_tag(info)
    }
    pub fn semantic_figure(&self, alternative: impl Into<String>) -> Self {
        let mut info = SemanticInfo::new("Figure");
        info.alternative_text = Some(alternative.into());
        self.semantic_tag(info)
    }
    pub fn semantic_image(&self, alternative: impl Into<String>) -> Self {
        self.semantic_figure(alternative)
    }
    pub fn semantic_formula(&self, alternative: impl Into<String>) -> Self {
        let mut info = SemanticInfo::new("Formula");
        info.alternative_text = Some(alternative.into());
        self.semantic_tag(info)
    }
    pub fn hyperlink(&self, url: impl Into<String>) -> Self {
        self.hyperlink_with_description(url, None)
    }
    pub fn hyperlink_with_description(
        &self,
        url: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        let url = url.into();
        assert!(!url.trim().is_empty());
        self.wrap(|c| Kind::Hyperlink(url, description, c))
    }
    pub fn section_link(&self, name: impl Into<String>) -> Self {
        self.section_link_with_description(name, None)
    }
    pub fn section_link_with_description(
        &self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty());
        self.wrap(|c| Kind::SectionLink(name, description, c))
    }
    fn wrap(&self, make: impl FnOnce(Element) -> Kind) -> Self {
        let child = Element::empty();
        self.0.set(make(child.clone()));
        Self(child)
    }
    fn pad(&self, v: [f32; 4]) -> Self {
        if let Some(c) = self.0.modify_padding(v) {
            Self(c)
        } else {
            self.wrap(|c| Kind::Padding(v, c))
        }
    }
    pub fn padding(&self, v: f32) -> Self {
        self.pad([v; 4])
    }
    pub fn padding_unit(&self, v: f32, u: Unit) -> Self {
        self.padding(u.points(v))
    }
    pub fn padding_horizontal(&self, v: f32) -> Self {
        self.pad([v, 0., v, 0.])
    }
    pub fn padding_vertical(&self, v: f32) -> Self {
        self.pad([0., v, 0., v])
    }
    pub fn padding_left(&self, v: f32) -> Self {
        self.pad([v, 0., 0., 0.])
    }
    pub fn padding_top(&self, v: f32) -> Self {
        self.pad([0., v, 0., 0.])
    }
    pub fn padding_right(&self, v: f32) -> Self {
        self.pad([0., 0., v, 0.])
    }
    pub fn padding_bottom(&self, v: f32) -> Self {
        self.pad([0., 0., 0., v])
    }
    fn constrained(&self, values: [Option<f32>; 4]) -> Self {
        assert!(
            values.iter().flatten().all(|v| *v >= 0.),
            "Constraints cannot be negative"
        );
        if let Some(c) = self.0.modify_constraints(values) {
            return Self(c);
        }
        self.wrap(|c| Kind::Constrained {
            min: Size::new(values[0].unwrap_or(0.), values[2].unwrap_or(0.)),
            max: Size::new(values[1].unwrap_or(INFINITY), values[3].unwrap_or(INFINITY)),
            child: c,
        })
    }
    pub fn width(&self, v: f32) -> Self {
        self.constrained([Some(v), Some(v), None, None])
    }
    pub fn height(&self, v: f32) -> Self {
        self.constrained([None, None, Some(v), Some(v)])
    }
    pub fn width_unit(&self, v: f32, u: Unit) -> Self {
        self.width(u.points(v))
    }
    pub fn height_unit(&self, v: f32, u: Unit) -> Self {
        self.height(u.points(v))
    }
    pub fn min_width(&self, v: f32) -> Self {
        self.constrained([Some(v), None, None, None])
    }
    pub fn min_height(&self, v: f32) -> Self {
        self.constrained([None, None, Some(v), None])
    }
    pub fn max_width(&self, v: f32) -> Self {
        self.constrained([None, Some(v), None, None])
    }
    pub fn max_height(&self, v: f32) -> Self {
        self.constrained([None, None, None, Some(v)])
    }
    fn align(&self, h: Option<HorizontalAlignment>, v: Option<VerticalAlignment>) -> Self {
        if let Some(c) = self.0.modify_alignment(h, v) {
            Self(c)
        } else {
            self.wrap(|c| Kind::Alignment(h, v, c))
        }
    }
    pub fn align_left(&self) -> Self {
        self.align(Some(HorizontalAlignment::Left), None)
    }
    pub fn align_center(&self) -> Self {
        self.align(Some(HorizontalAlignment::Center), None)
    }
    pub fn align_right(&self) -> Self {
        self.align(Some(HorizontalAlignment::Right), None)
    }
    pub fn align_top(&self) -> Self {
        self.align(None, Some(VerticalAlignment::Top))
    }
    pub fn align_middle(&self) -> Self {
        self.align(None, Some(VerticalAlignment::Middle))
    }
    pub fn align_bottom(&self) -> Self {
        self.align(None, Some(VerticalAlignment::Bottom))
    }
    pub fn default_text_style(&self, style: TextStyle) -> Self {
        self.wrap(|c| Kind::DefaultTextStyle(style, c))
    }
    pub fn content_from_right_to_left(&self) -> Self {
        self.wrap(|c| Kind::Direction(ContentDirection::RightToLeft, c))
    }
    pub fn content_from_left_to_right(&self) -> Self {
        self.wrap(|c| Kind::Direction(ContentDirection::LeftToRight, c))
    }
    fn styled_box(&self, change: impl Fn(&mut BoxStyle)) -> Self {
        // As in StyledBoxExtensions, consecutive styling calls share one box.
        if let Some(child) = self.0.modify_box(&change) {
            return Self(child);
        }
        self.wrap(|c| {
            let mut s = BoxStyle::default();
            change(&mut s);
            Kind::StyledBox(s, c)
        })
    }
    pub fn background(&self, v: Color) -> Self {
        self.styled_box(|s| s.background = v)
    }
    fn set_border(&self, values: [Option<f32>; 4]) -> Self {
        assert!(values.iter().flatten().all(|v| *v >= 0.));
        self.styled_box(|s| {
            if s.border_color == Color::TRANSPARENT {
                s.border_color = Color::BLACK;
            }
            for (edge, value) in s.border.iter_mut().zip(values) {
                if let Some(value) = value {
                    *edge = value;
                }
            }
        })
    }
    pub fn border(&self, v: f32) -> Self {
        self.set_border([Some(v); 4])
    }
    pub fn border_color(&self, v: Color) -> Self {
        self.styled_box(|s| s.border_color = v)
    }
    pub fn corner_radius(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.styled_box(|s| {
            s.radius = [v; 4];
            s.extended = true;
        })
    }
    pub fn corner_radius_top_left(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.styled_box(|s| {
            s.radius[0] = v;
            s.extended = true;
        })
    }
    pub fn corner_radius_top_right(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.styled_box(|s| {
            s.radius[1] = v;
            s.extended = true;
        })
    }
    pub fn corner_radius_bottom_right(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.styled_box(|s| {
            s.radius[2] = v;
            s.extended = true;
        })
    }
    pub fn corner_radius_bottom_left(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.styled_box(|s| {
            s.radius[3] = v;
            s.extended = true;
        })
    }
    pub fn border_left(&self, v: f32) -> Self {
        self.set_border([Some(v), None, None, None])
    }
    pub fn border_right(&self, v: f32) -> Self {
        self.set_border([None, None, Some(v), None])
    }
    pub fn border_horizontal(&self, v: f32) -> Self {
        self.set_border([None, Some(v), None, Some(v)])
    }
    pub fn border_vertical(&self, v: f32) -> Self {
        self.set_border([Some(v), None, Some(v), None])
    }
    pub fn border_alignment_inside(&self) -> Self {
        self.styled_box(|s| {
            s.border_alignment = Some(0.);
            s.extended = true;
        })
    }
    pub fn border_alignment_middle(&self) -> Self {
        self.styled_box(|s| {
            s.border_alignment = Some(0.5);
            s.extended = true;
        })
    }
    pub fn border_alignment_outside(&self) -> Self {
        self.styled_box(|s| {
            s.border_alignment = Some(1.);
            s.extended = true;
        })
    }
    pub fn background_linear_gradient(&self, angle: f32, colors: Vec<Color>) -> Self {
        assert!(!colors.is_empty());
        self.styled_box(|s| {
            s.background_angle = angle;
            s.background_gradient = colors.clone();
            s.extended = true;
        })
    }
    pub fn border_linear_gradient(&self, angle: f32, colors: Vec<Color>) -> Self {
        assert!(!colors.is_empty());
        self.styled_box(|s| {
            s.border_angle = angle;
            s.border_gradient = colors.clone();
            s.extended = true;
        })
    }
    pub fn shadow(&self, shadow: BoxShadowStyle) -> Self {
        assert!(shadow.blur >= 0.);
        self.styled_box(|s| {
            s.shadow = Some(shadow);
            s.extended = true;
        })
    }
    pub fn border_bottom(&self, v: f32) -> Self {
        self.set_border([None, None, None, Some(v)])
    }
    pub fn border_top(&self, v: f32) -> Self {
        self.set_border([None, Some(v), None, None])
    }
    pub fn aspect_ratio(&self, ratio: f32, option: AspectRatioOption) -> Self {
        self.wrap(|c| Kind::AspectRatio(ratio, option, c))
    }
    pub fn scale_to_fit(&self) -> Self {
        self.wrap(Kind::ScaleToFit)
    }
    pub fn scale(&self, v: f32) -> Self {
        self.scale_value(v, v)
    }
    fn scale_value(&self, x: f32, y: f32) -> Self {
        assert!(x != 0. && y != 0.);
        self.0
            .modify_transform(x, y, true)
            .map(Self)
            .unwrap_or_else(|| self.wrap(|c| Kind::Scale(x, y, c)))
    }
    pub fn scale_horizontal(&self, v: f32) -> Self {
        self.scale_value(v, 1.)
    }
    pub fn scale_vertical(&self, v: f32) -> Self {
        self.scale_value(1., v)
    }
    pub fn flip_horizontal(&self) -> Self {
        self.scale_value(-1., 1.)
    }
    pub fn flip_vertical(&self) -> Self {
        self.scale_value(1., -1.)
    }
    pub fn flip_over(&self) -> Self {
        self.scale_value(-1., -1.)
    }
    pub fn rotate(&self, degrees: f32) -> Self {
        self.0
            .modify_rotation(degrees, 0, false)
            .map(Self)
            .unwrap_or_else(|| self.wrap(|c| Kind::Rotate(degrees, c)))
    }
    pub fn rotate_right(&self) -> Self {
        self.0
            .modify_rotation(0., 1, true)
            .map(Self)
            .unwrap_or_else(|| self.wrap(|c| Kind::RotateLayout(1, c)))
    }
    pub fn rotate_left(&self) -> Self {
        self.0
            .modify_rotation(0., -1, true)
            .map(Self)
            .unwrap_or_else(|| self.wrap(|c| Kind::RotateLayout(-1, c)))
    }
    pub fn translate(&self, x: f32, y: f32) -> Self {
        self.0
            .modify_transform(x, y, false)
            .map(Self)
            .unwrap_or_else(|| self.wrap(|c| Kind::Offset(x, y, c)))
    }
    pub fn offset_x(&self, value: f32, unit: Unit) -> Self {
        self.translate(unit.points(value), 0.)
    }
    pub fn offset_y(&self, value: f32, unit: Unit) -> Self {
        self.translate(0., unit.points(value))
    }
    pub fn translate_x(&self, value: f32, unit: Unit) -> Self {
        self.offset_x(value, unit)
    }
    pub fn translate_y(&self, value: f32, unit: Unit) -> Self {
        self.offset_y(value, unit)
    }
    pub fn rotate_layout_clockwise(&self) -> Self {
        self.rotate_right()
    }
    pub fn rotate_layout_counterclockwise(&self) -> Self {
        self.rotate_left()
    }
    pub fn extend(&self) -> Self {
        self.wrap(|c| Kind::Extend(true, true, c))
    }
    pub fn extend_horizontal(&self) -> Self {
        self.wrap(|c| Kind::Extend(true, false, c))
    }
    pub fn extend_vertical(&self) -> Self {
        self.wrap(|c| Kind::Extend(false, true, c))
    }
    pub fn shrink(&self) -> Self {
        self.wrap(|c| Kind::Shrink(true, true, c))
    }
    pub fn shrink_horizontal(&self) -> Self {
        self.wrap(|c| Kind::Shrink(true, false, c))
    }
    pub fn shrink_vertical(&self) -> Self {
        self.wrap(|c| Kind::Shrink(false, true, c))
    }
    pub fn show_entire(&self) -> Self {
        self.wrap(Kind::ShowEntire)
    }
    pub fn ensure_space(&self, v: f32) -> Self {
        self.wrap(|c| Kind::EnsureSpace(v, c))
    }
    pub fn prevent_page_break(&self) -> Self {
        self.wrap(Kind::PreventPageBreak)
    }
    pub fn show_once(&self) -> Self {
        self.wrap(Kind::ShowOnce)
    }
    pub fn skip_once(&self) -> Self {
        self.wrap(Kind::SkipOnce)
    }
    pub fn repeat(&self) -> Self {
        self.wrap(|c| Kind::Repeat(-2, c))
    }
    pub fn show_if(&self, v: bool) -> Self {
        if v { self.clone() } else { Self::new() }
    }
    pub fn unconstrained(&self) -> Self {
        self.wrap(Kind::Unconstrained)
    }
    pub fn stop_paging(&self) -> Self {
        self.wrap(Kind::StopPaging)
    }
    pub fn page_break(&self) {
        self.0.set(Kind::PageBreak)
    }
    pub fn text(&self, text: impl Into<String>) -> TextDescriptor {
        self.text_spans(|t| {
            t.span(text);
        })
    }
    pub fn text_spans(&self, compose: impl FnOnce(&mut TextDescriptor)) -> TextDescriptor {
        let alignment = self.0.infer_text_alignment();
        self.0.set(Kind::Text(TextBlock {
            alignment,
            ..Default::default()
        }));
        let mut t = TextDescriptor {
            element: self.0.clone(),
            span: None,
        };
        compose(&mut t);
        t
    }
    pub fn column(&self, compose: impl FnOnce(&mut ColumnDescriptor)) {
        let mut d = ColumnDescriptor::default();
        compose(&mut d);
        self.0.set(Kind::Column(d.items, d.spacing))
    }
    pub fn row(&self, compose: impl FnOnce(&mut RowDescriptor)) {
        let mut d = RowDescriptor::default();
        compose(&mut d);
        self.0.set(Kind::Row(d.items, d.spacing))
    }
    pub fn inlined(&self, compose: impl FnOnce(&mut InlinedDescriptor)) {
        let mut d = InlinedDescriptor(Inlined::default());
        compose(&mut d);
        self.0.set(Kind::Inlined(d.0))
    }
    pub fn table(&self, compose: impl FnOnce(&mut TableDescriptor)) {
        let mut d = TableDescriptor::default();
        compose(&mut d);
        self.0.set(Kind::Table(d.table))
    }
    pub fn lazy(&self, source: impl Fn(Container) + 'static) {
        self.0.set(Kind::Lazy {
            source: std::rc::Rc::new(source),
            child: None,
            retain_content: false,
        });
    }
    pub fn lazy_with_cache(&self, source: impl Fn(Container) + 'static) {
        self.lazy(source);
        self.0.set_lazy_cacheable();
    }
    pub fn grid(&self, compose: impl FnOnce(&mut GridDescriptor)) {
        let alignment = match self.0.infer_text_alignment() {
            TextAlignment::Center => HorizontalAlignment::Center,
            TextAlignment::Right => HorizontalAlignment::Right,
            _ => HorizontalAlignment::Left,
        };
        let mut d = GridDescriptor {
            items: vec![],
            columns: 12,
            horizontal: 0.,
            vertical: 0.,
            alignment,
        };
        compose(&mut d);
        let mut rows = vec![];
        let mut items = d.items.into_iter().peekable();
        while items.peek().is_some() {
            let mut used = 0;
            let mut row = vec![];
            while let Some((span, _)) = items.peek() {
                assert!(
                    *span <= d.columns,
                    "Grid item span exceeds its column count"
                );
                if used + span > d.columns {
                    break;
                }
                let (span, item) = items.next().unwrap();
                used += span;
                row.push(RowItem {
                    element: item,
                    kind: RowItemType::Relative,
                    value: span as f32,
                    rendered: false,
                });
            }
            let missing = d.columns - used;
            let weight = if d.alignment == HorizontalAlignment::Center {
                missing / 2
            } else {
                missing
            };
            if missing > 0 && d.alignment != HorizontalAlignment::Left {
                row.insert(
                    0,
                    RowItem {
                        element: Element::empty(),
                        kind: RowItemType::Relative,
                        value: weight as f32,
                        rendered: false,
                    },
                );
            }
            if missing > 0 && d.alignment != HorizontalAlignment::Right {
                row.push(RowItem {
                    element: Element::empty(),
                    kind: RowItemType::Relative,
                    value: weight as f32,
                    rendered: false,
                });
            }
            let el = Element::empty();
            el.set(Kind::Row(row, d.horizontal));
            rows.push(el);
        }
        self.0.set(Kind::Column(rows, d.vertical));
    }
    pub fn layers(&self, compose: impl FnOnce(&mut LayersDescriptor)) {
        let mut d = LayersDescriptor::default();
        compose(&mut d);
        self.0.set(Kind::Layers(d.items))
    }
    pub fn decoration(&self, compose: impl FnOnce(&mut DecorationDescriptor)) {
        let mut d = DecorationDescriptor::default();
        compose(&mut d);
        self.0.set(Kind::Decoration {
            before: d.before,
            content: d.content,
            after: d.after,
        })
    }
    pub fn multi_column(&self, compose: impl FnOnce(&mut MultiColumnDescriptor)) {
        let mut d = MultiColumnDescriptor::default();
        compose(&mut d);
        let target = if d.balance {
            self.shrink_vertical()
        } else {
            self.clone()
        };
        target.0.set(Kind::MultiColumn {
            content: d.content,
            spacer: d.spacer,
            count: d.count,
            spacing: d.spacing,
            balance: d.balance,
        })
    }
    pub fn svg(&self, svg: impl Into<String>) -> ImageDescriptor {
        let fit = self.0.infer_image_fit();
        self.0.set(Kind::Svg(svg.into()));
        self.0.set_image_fit(fit);
        ImageDescriptor {
            element: self.0.clone(),
            fit,
        }
    }
    pub fn svg_file(&self, path: impl AsRef<Path>) -> Result<ImageDescriptor> {
        Ok(self.svg(std::fs::read_to_string(
            crate::runtime::resolve_resource_file_path(path)?,
        )?))
    }
    pub fn svg_dynamic(&self, source: impl Fn(Size) -> String + 'static) {
        self.0.set(Kind::DynamicSvg(std::rc::Rc::new(source)))
    }
    pub fn svg_path(&self, path: impl Into<String>, color: Color) {
        self.0.set(Kind::SvgPath(path.into(), color));
    }
    pub fn placeholder(&self, text: impl AsRef<str>) {
        let c = self
            .background(Color(0xffe0e0e0))
            .padding(5.)
            .align_middle()
            .align_center();
        if text.as_ref().trim().is_empty() {
            c.height(24.).width(24.).svg_path("M19,19H5V5H19M19,3H5A2,2 0 0,0 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V5A2,2 0 0,0 19,3M13.96,12.29L11.21,15.83L9.25,13.47L6.5,17H17.5L13.96,12.29Z",Color::BLACK);
        } else {
            c.text(text.as_ref()).font_size(14.);
        }
    }
    pub fn image_dynamic(
        &self,
        source: impl Fn(GenerateDynamicImagePayload) -> Option<Vec<u8>> + 'static,
    ) -> DynamicImageDescriptor {
        self.0.set(Kind::DynamicImage(std::rc::Rc::new(source)));
        DynamicImageDescriptor(self.0.clone())
    }
    pub fn image(&self, bytes: Vec<u8>) -> ImageDescriptor {
        self.0.set(Kind::Image(bytes));
        ImageDescriptor {
            element: self.0.clone(),
            fit: AspectRatioOption::FitWidth,
        }
    }
    pub fn image_file(&self, path: impl AsRef<Path>) -> Result<ImageDescriptor> {
        Ok(
            self.image(std::fs::read(crate::runtime::resolve_resource_file_path(
                path,
            )?)?),
        )
    }
    pub fn line_horizontal(&self, width: f32) -> LineDescriptor {
        assert!(width >= 0.);
        self.0.set(Kind::Line {
            width,
            color: Color::BLACK,
            vertical: false,
            dash: vec![],
            gradient: vec![],
        });
        LineDescriptor(self.0.clone())
    }
    pub fn line_vertical(&self, width: f32) -> LineDescriptor {
        assert!(width >= 0.);
        self.0.set(Kind::Line {
            width,
            color: Color::BLACK,
            vertical: true,
            dash: vec![],
            gradient: vec![],
        });
        LineDescriptor(self.0.clone())
    }
}
impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Default)]
pub struct ColumnDescriptor {
    items: Vec<Element>,
    spacing: f32,
}
impl ColumnDescriptor {
    pub fn spacing(&mut self, v: f32) {
        self.spacing = v
    }
    pub fn item(&mut self) -> Container {
        let c = Container::new();
        self.items.push(c.0.clone());
        c
    }
}
#[derive(Default)]
pub struct InlinedDescriptor(Inlined);
impl InlinedDescriptor {
    pub fn item(&mut self) -> Container {
        let c = Container::new();
        self.0.items.push(c.0.clone());
        c
    }
    pub fn spacing(&mut self, v: f32) {
        self.horizontal_spacing(v);
        self.vertical_spacing(v);
    }
    pub fn horizontal_spacing(&mut self, v: f32) {
        assert!(v >= 0.);
        self.0.horizontal_spacing = v;
    }
    pub fn vertical_spacing(&mut self, v: f32) {
        assert!(v >= 0.);
        self.0.vertical_spacing = v;
    }
    pub fn baseline_top(&mut self) {
        self.0.baseline = VerticalAlignment::Top;
    }
    pub fn baseline_middle(&mut self) {
        self.0.baseline = VerticalAlignment::Middle;
    }
    pub fn baseline_bottom(&mut self) {
        self.0.baseline = VerticalAlignment::Bottom;
    }
    pub fn align_left(&mut self) {
        self.0.alignment = Some(InlinedAlignment::Left);
    }
    pub fn align_center(&mut self) {
        self.0.alignment = Some(InlinedAlignment::Center);
    }
    pub fn align_right(&mut self) {
        self.0.alignment = Some(InlinedAlignment::Right);
    }
    pub fn align_justify(&mut self) {
        self.0.alignment = Some(InlinedAlignment::Justify);
    }
    pub fn align_space_around(&mut self) {
        self.0.alignment = Some(InlinedAlignment::SpaceAround);
    }
}
pub struct GridDescriptor {
    items: Vec<(usize, Element)>,
    columns: usize,
    horizontal: f32,
    vertical: f32,
    alignment: HorizontalAlignment,
}
impl GridDescriptor {
    pub fn alignment(&mut self, alignment: HorizontalAlignment) {
        self.alignment = alignment;
    }
    pub fn columns(&mut self, count: usize) {
        assert!(count > 0);
        self.columns = count;
    }
    pub fn spacing(&mut self, value: f32) {
        self.horizontal_spacing(value);
        self.vertical_spacing(value);
    }
    pub fn horizontal_spacing(&mut self, value: f32) {
        assert!(value >= 0.);
        self.horizontal = value;
    }
    pub fn vertical_spacing(&mut self, value: f32) {
        assert!(value >= 0.);
        self.vertical = value;
    }
    pub fn align_left(&mut self) {
        self.alignment = HorizontalAlignment::Left;
    }
    pub fn align_center(&mut self) {
        self.alignment = HorizontalAlignment::Center;
    }
    pub fn align_right(&mut self) {
        self.alignment = HorizontalAlignment::Right;
    }
    pub fn item(&mut self, columns: usize) -> Container {
        assert!(columns > 0);
        let c = Container::new();
        self.items.push((columns, c.0.clone()));
        c
    }
}
#[derive(Default)]
pub struct RowDescriptor {
    items: Vec<RowItem>,
    spacing: f32,
}
impl RowDescriptor {
    pub fn spacing(&mut self, v: f32) {
        self.spacing = v
    }
    fn item(&mut self, kind: RowItemType, value: f32) -> Container {
        let c = Container::new();
        self.items.push(RowItem {
            element: c.0.clone(),
            kind,
            value,
            rendered: false,
        });
        c
    }
    pub fn relative_item(&mut self) -> Container {
        self.relative_item_weight(1.)
    }
    pub fn relative_item_weight(&mut self, v: f32) -> Container {
        self.item(RowItemType::Relative, v)
    }
    pub fn constant_item(&mut self, v: f32) -> Container {
        self.item(RowItemType::Constant, v)
    }
    pub fn constant_item_unit(&mut self, v: f32, u: Unit) -> Container {
        self.constant_item(u.points(v))
    }
    pub fn auto_item(&mut self) -> Container {
        self.item(RowItemType::Auto, 0.)
    }
}
#[derive(Default)]
pub struct TableDescriptor {
    pub(crate) table: Table,
}
impl TableDescriptor {
    pub fn extend_last_cells_to_table_bottom(&mut self) {
        self.table.extend_last_cells = true;
    }
    pub fn columns_definition(&mut self, compose: impl FnOnce(&mut ColumnsDescriptor)) {
        let mut d = ColumnsDescriptor::default();
        compose(&mut d);
        self.table.columns = d.0
    }
    pub fn cell(&mut self) -> CellDescriptor {
        let c = Container::new();
        let cell = TableCell::new(c.0.clone());
        let position = cell.position.clone();
        self.table.cells.push(cell);
        CellDescriptor {
            container: c,
            position,
        }
    }
    pub fn header(&mut self, compose: impl FnOnce(&mut TableDescriptor)) {
        let mut d = Self::default();
        d.table.columns = self.table.columns.clone();
        compose(&mut d);
        self.table.header = d.table.cells
    }
    pub fn footer(&mut self, compose: impl FnOnce(&mut TableDescriptor)) {
        let mut d = Self::default();
        d.table.columns = self.table.columns.clone();
        compose(&mut d);
        self.table.footer = d.table.cells
    }
}
#[derive(Default)]
pub struct ColumnsDescriptor(Vec<ColumnDefinition>);
#[derive(Default)]
pub struct LayersDescriptor {
    items: Vec<(bool, Element)>,
}
impl LayersDescriptor {
    pub fn primary_layer(&mut self) -> Container {
        let c = Container::new();
        self.items.push((true, c.0.clone()));
        c
    }
    pub fn layer(&mut self) -> Container {
        let c = Container::new();
        self.items.push((false, c.0.clone()));
        c
    }
}
pub struct DecorationDescriptor {
    before: Element,
    content: Element,
    after: Element,
}
pub struct MultiColumnDescriptor {
    content: Element,
    spacer: Element,
    count: usize,
    spacing: f32,
    balance: bool,
}
impl Default for MultiColumnDescriptor {
    fn default() -> Self {
        Self {
            content: Element::empty(),
            spacer: Element::empty(),
            count: 2,
            spacing: 0.,
            balance: false,
        }
    }
}
impl MultiColumnDescriptor {
    pub fn content(&mut self) -> Container {
        Container(self.content.clone())
    }
    pub fn spacer(&mut self) -> Container {
        Container(self.spacer.clone()).repeat()
    }
    pub fn columns(&mut self, count: usize) {
        self.count = count
    }
    pub fn spacing(&mut self, value: f32) {
        self.spacing = value
    }
    pub fn balance_height(&mut self, enable: bool) {
        self.balance = enable
    }
}
impl Default for DecorationDescriptor {
    fn default() -> Self {
        Self {
            before: Element::empty(),
            content: Element::empty(),
            after: Element::empty(),
        }
    }
}
impl DecorationDescriptor {
    pub fn before(&mut self) -> Container {
        Container(self.before.clone()).wrap(|c| Kind::Repeat(-3, c))
    }
    pub fn content(&mut self) -> Container {
        Container(self.content.clone())
    }
    pub fn after(&mut self) -> Container {
        Container(self.after.clone()).wrap(|c| Kind::Repeat(-4, c))
    }
}
impl ColumnsDescriptor {
    pub fn relative_column(&mut self) {
        self.relative_column_weight(1.)
    }
    pub fn relative_column_weight(&mut self, v: f32) {
        self.0.push(ColumnDefinition::Relative(v))
    }
    pub fn constant_column(&mut self, v: f32) {
        self.0.push(ColumnDefinition::Constant(v))
    }
}
pub struct CellDescriptor {
    pub container: Container,
    position: std::rc::Rc<std::cell::RefCell<CellPosition>>,
}
impl CellDescriptor {
    pub fn as_semantic_horizontal_header(self) -> Self {
        self.semantic_horizontal_header()
    }
    pub fn semantic_horizontal_header(self) -> Self {
        self.position.borrow_mut().semantic_horizontal_header = true;
        self
    }
    pub fn column_span(self, value: usize) -> Self {
        self.position.borrow_mut().column_span = value;
        self
    }
    pub fn row_span(self, value: usize) -> Self {
        self.position.borrow_mut().row_span = value;
        self
    }
    pub fn column(self, value: usize) -> Self {
        self.position.borrow_mut().column = value;
        self
    }
    pub fn row(self, value: usize) -> Self {
        self.position.borrow_mut().row = value;
        self
    }
}
impl std::ops::Deref for CellDescriptor {
    type Target = Container;
    fn deref(&self) -> &Container {
        &self.container
    }
}
#[derive(Clone)]
pub struct TextDescriptor {
    element: Element,
    span: Option<usize>,
}
impl TextDescriptor {
    fn mutate_style(&self, change: impl FnOnce(&mut TextStyle)) -> Self {
        self.element.change_text_style(self.span, change);
        self.clone()
    }
    pub fn span(&mut self, text: impl Into<String>) -> Self {
        self.add(SpanContent::Text(text.into()))
    }
    pub fn current_page_number(&mut self) -> Self {
        self.add(SpanContent::CurrentPageNumber)
    }
    pub fn total_pages(&mut self) -> Self {
        self.add(SpanContent::TotalPages)
    }
    pub fn begin_page_number_of_section(&mut self, name: impl Into<String>) -> Self {
        self.add(SpanContent::SectionPageNumber(
            name.into(),
            SectionPageNumber::Begin,
        ))
    }
    pub fn end_page_number_of_section(&mut self, name: impl Into<String>) -> Self {
        self.add(SpanContent::SectionPageNumber(
            name.into(),
            SectionPageNumber::End,
        ))
    }
    pub fn page_number_within_section(&mut self, name: impl Into<String>) -> Self {
        self.add(SpanContent::SectionPageNumber(
            name.into(),
            SectionPageNumber::Current,
        ))
    }
    pub fn total_pages_within_section(&mut self, name: impl Into<String>) -> Self {
        self.add(SpanContent::SectionPageNumber(
            name.into(),
            SectionPageNumber::Total,
        ))
    }
    pub fn format(&self, formatter: impl Fn(Option<i32>) -> String + 'static) -> Self {
        self.element.format_page_number(
            self.span.expect("Format applies to a page-number span"),
            PageNumberFormatter(std::rc::Rc::new(formatter)),
        );
        self.clone()
    }
    pub fn empty_line(&mut self) -> Self {
        self.span("\n")
    }
    pub fn line(&mut self, value: impl Into<String>) -> Self {
        let span = self.span(value);
        self.span("\n");
        span
    }
    pub fn hyperlink(&mut self, text: impl Into<String>, url: impl Into<String>) -> Self {
        let url = url.into();
        assert!(!url.trim().is_empty());
        self.add(SpanContent::Hyperlink(text.into(), url))
    }
    pub fn section_link(&mut self, text: impl Into<String>, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.trim().is_empty());
        self.add(SpanContent::SectionLink(text.into(), name))
    }
    pub fn element(&mut self, alignment: TextInjectedElementAlignment) -> Container {
        let c = Container::new();
        self.add(SpanContent::Element(c.0.clone(), alignment));
        c.align_bottom().shrink()
    }
    pub fn clamp_lines(&self, count: usize, ellipsis: impl Into<String>) -> Self {
        assert!(count > 0);
        self.element
            .configure_text(|t| t.clamp = Some((count, ellipsis.into())));
        self.clone()
    }
    pub fn paragraph_spacing(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.element.configure_text(|t| t.paragraph_spacing = v);
        self.clone()
    }
    pub fn paragraph_first_line_indentation(&self, v: f32) -> Self {
        assert!(v >= 0.);
        self.element.configure_text(|t| t.indentation = v);
        self.clone()
    }
    fn add(&mut self, content: SpanContent) -> Self {
        let i = self.element.add_span(TextSpan {
            content,
            style: TextStyle::default(),
            formatter: None,
        });
        Self {
            element: self.element.clone(),
            span: Some(i),
        }
    }
    pub fn font_size(&self, v: f32) -> Self {
        self.mutate_style(|s| s.size = Some(v))
    }
    pub fn style(&self, value: TextStyle) -> Self {
        self.style_with(|s| s.merge(&value))
    }
    pub fn decoration_solid(&self) -> Self {
        self.style_with(TextStyle::decoration_solid)
    }
    pub fn decoration_double(&self) -> Self {
        self.style_with(TextStyle::decoration_double)
    }
    pub fn decoration_dotted(&self) -> Self {
        self.style_with(TextStyle::decoration_dotted)
    }
    pub fn decoration_dashed(&self) -> Self {
        self.style_with(TextStyle::decoration_dashed)
    }
    pub fn decoration_wavy(&self) -> Self {
        self.style_with(TextStyle::decoration_wavy)
    }
    pub fn direction_auto(&self) -> Self {
        self.style_with(TextStyle::direction_auto)
    }
    pub fn direction_from_left_to_right(&self) -> Self {
        self.style_with(TextStyle::direction_from_left_to_right)
    }
    pub fn direction_from_right_to_left(&self) -> Self {
        self.style_with(TextStyle::direction_from_right_to_left)
    }
    pub fn wrap_anywhere(&self, value: bool) -> Self {
        self.break_anywhere(value)
    }
    pub fn fallback(&self, value: &TextStyle) -> Self {
        self.style_with(|s| s.fallback(value))
    }
    pub fn style_with(&self, change: impl FnOnce(TextStyle) -> TextStyle) -> Self {
        self.mutate_style(|s| *s = change(s.clone()))
    }
    pub fn subscript(&self) -> Self {
        self.style_with(TextStyle::subscript)
    }
    pub fn superscript(&self) -> Self {
        self.style_with(TextStyle::superscript)
    }
    pub fn normal_position(&self) -> Self {
        self.style_with(TextStyle::normal_position)
    }
    pub fn overline(&self) -> Self {
        self.style_with(TextStyle::overline)
    }
    pub fn break_anywhere(&self, v: bool) -> Self {
        self.style_with(|s| s.break_anywhere(v))
    }
    pub fn font_families(&self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.style_with(|s| s.font_families(values))
    }
    pub fn enable_font_feature(&self, name: impl Into<String>) -> Self {
        self.style_with(|s| s.font_feature(name, true))
    }
    pub fn disable_font_feature(&self, name: impl Into<String>) -> Self {
        self.style_with(|s| s.font_feature(name, false))
    }
    pub fn decoration_color(&self, v: Color) -> Self {
        self.style_with(|s| s.decoration_color(v))
    }
    pub fn decoration_thickness(&self, v: f32) -> Self {
        self.style_with(|s| s.decoration_thickness(v))
    }
    pub fn decoration_style(&self, v: TextDecorationStyle) -> Self {
        self.style_with(|s| s.decoration_style(v))
    }
    pub fn font_family(&self, v: impl Into<String>) -> Self {
        let v = v.into();
        self.mutate_style(|s| s.font_families = Some(vec![v.clone()]))
    }
    pub fn font_color(&self, v: Color) -> Self {
        self.mutate_style(|s| s.color = Some(v))
    }
    pub fn background_color(&self, v: Color) -> Self {
        self.mutate_style(|s| s.background = Some(v))
    }
    pub fn weight(&self, v: i32) -> Self {
        self.mutate_style(|s| s.weight = Some(v))
    }
    pub fn bold(&self) -> Self {
        self.weight(700)
    }
    pub fn semi_bold(&self) -> Self {
        self.weight(600)
    }
    pub fn extra_black(&self) -> Self {
        self.weight(1000)
    }
    pub fn normal_weight(&self) -> Self {
        self.weight(400)
    }
    pub fn italic(&self) -> Self {
        self.mutate_style(|s| s.italic = Some(true))
    }
    pub fn line_height(&self, v: f32) -> Self {
        self.mutate_style(|s| s.line_height = Some(v))
    }
    pub fn letter_spacing(&self, v: f32) -> Self {
        self.mutate_style(|s| s.letter_spacing = Some(v))
    }
    pub fn word_spacing(&self, v: f32) -> Self {
        self.mutate_style(|s| s.word_spacing = Some(v))
    }
    pub fn underline(&self) -> Self {
        self.style_with(TextStyle::underline)
    }
    pub fn strikethrough(&self) -> Self {
        self.style_with(TextStyle::strikethrough)
    }
    fn align(&self, v: TextAlignment) -> Self {
        self.element.align_text(v);
        self.clone()
    }
    pub fn align_center(&self) -> Self {
        self.align(TextAlignment::Center)
    }
    pub fn align_left(&self) -> Self {
        self.align(TextAlignment::Left)
    }
    pub fn align_right(&self) -> Self {
        self.align(TextAlignment::Right)
    }
    pub fn align_start(&self) -> Self {
        self.align(TextAlignment::Start)
    }
    pub fn align_end(&self) -> Self {
        self.align(TextAlignment::End)
    }
    pub fn justify(&self) -> Self {
        self.align(TextAlignment::Justify)
    }
}
pub struct ImageDescriptor {
    element: Element,
    fit: AspectRatioOption,
}
pub struct DynamicImageDescriptor(Element);
impl DynamicImageDescriptor {
    pub fn use_original_image(self, value: bool) -> Self {
        self.0.configure_image(Some(value), None, None, None);
        self
    }
    pub fn with_raster_dpi(self, dpi: f32) -> Self {
        assert!(dpi > 0. && dpi.is_finite());
        self.0.configure_image(None, Some(dpi), None, None);
        self
    }
    pub fn with_compression_quality(self, quality: ImageCompressionQuality) -> Self {
        self.0.configure_image(None, None, Some(quality), None);
        self
    }
}
impl ImageDescriptor {
    pub fn use_original_image(self, value: bool) -> Self {
        self.element.configure_image(Some(value), None, None, None);
        self
    }
    pub fn with_raster_dpi(self, dpi: f32) -> Self {
        assert!(dpi > 0. && dpi.is_finite());
        self.element.configure_image(None, Some(dpi), None, None);
        self
    }
    pub fn with_compression_quality(self, quality: ImageCompressionQuality) -> Self {
        self.element
            .configure_image(None, None, Some(quality), None);
        self
    }
    pub fn fit_unproportionally(self) -> Self {
        self.element.configure_image(None, None, None, Some(true));
        self
    }
    fn fit(mut self, v: AspectRatioOption) -> Self {
        self.fit = v;
        self.element.set_image_fit(v);
        self
    }
    pub fn fit_width(self) -> Self {
        self.fit(AspectRatioOption::FitWidth)
    }
    pub fn fit_height(self) -> Self {
        self.fit(AspectRatioOption::FitHeight)
    }
    pub fn fit_area(self) -> Self {
        self.fit(AspectRatioOption::FitArea)
    }
}
pub struct LineDescriptor(Element);
impl LineDescriptor {
    pub fn line_dash_pattern(&self, dash: Vec<f32>) -> &Self {
        assert!(!dash.is_empty() && dash.len().is_multiple_of(2));
        self.0.configure_line(Some(dash), None);
        self
    }
    pub fn line_gradient(&self, gradient: Vec<Color>) -> &Self {
        assert!(!gradient.is_empty());
        self.0.configure_line(None, Some(gradient));
        self
    }
    pub fn line_color(&self, color: Color) -> &Self {
        self.0.line_color(color);
        self
    }
}
