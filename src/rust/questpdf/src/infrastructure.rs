//! Direct counterparts of QuestPDF.Infrastructure value types.
use crate::Element;
use anyhow::Result;
pub const EPSILON: f32 = 0.01;
pub const INFINITY: f32 = 14_400.;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    pub fn with_unit(width: f32, height: f32, unit: Unit) -> Self {
        assert!(width >= 0. && height >= 0.);
        Self::new(unit.points(width), unit.points(height))
    }
    pub fn portrait(self) -> Self {
        Self::new(self.width.min(self.height), self.width.max(self.height))
    }
    pub fn landscape(self) -> Self {
        Self::new(self.width.max(self.height), self.width.min(self.height))
    }
    pub const ZERO: Self = Self {
        width: 0.,
        height: 0.,
    };
    pub const MAX: Self = Self {
        width: INFINITY,
        height: INFINITY,
    };
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
    pub fn negative(self) -> bool {
        self.width < -EPSILON || self.height < -EPSILON
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpacePlanType {
    Empty,
    Wrap,
    PartialRender,
    FullRender,
}
#[derive(Clone, Copy, Debug)]
pub struct SpacePlan {
    pub kind: SpacePlanType,
    pub size: Size,
}
impl SpacePlan {
    pub fn empty() -> Self {
        Self {
            kind: SpacePlanType::Empty,
            size: Size::ZERO,
        }
    }
    pub fn wrap() -> Self {
        Self {
            kind: SpacePlanType::Wrap,
            size: Size::ZERO,
        }
    }
    pub fn full(width: f32, height: f32) -> Self {
        Self {
            kind: SpacePlanType::FullRender,
            size: Size::new(width, height),
        }
    }
    pub fn partial(width: f32, height: f32) -> Self {
        Self {
            kind: SpacePlanType::PartialRender,
            size: Size::new(width, height),
        }
    }
    pub fn done(self) -> bool {
        matches!(self.kind, SpacePlanType::FullRender | SpacePlanType::Empty)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContentDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Point,
    Millimetre,
    Centimetre,
    Inch,
    Metre,
    Foot,
}
impl Unit {
    pub fn points(self, value: f32) -> f32 {
        value
            * match self {
                Self::Point => 1.,
                Self::Millimetre => 72. / 25.4,
                Self::Centimetre => 72. / 2.54,
                Self::Inch => 72.,
                Self::Metre => 72. / 0.0254,
                Self::Foot => 864.,
            }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color(pub u32);
impl Color {
    pub const BLACK: Self = Self(0xff000000);
    pub const WHITE: Self = Self(0xffffffff);
    pub const TRANSPARENT: Self = Self(0);
    pub fn hex(value: &str) -> Result<Self> {
        let v = value.trim().trim_start_matches('#');
        anyhow::ensure!(
            [3, 4, 6, 8].contains(&v.len()) && v.is_ascii(),
            "Invalid color: {value}"
        );
        if v.len() <= 4 {
            let expanded: String = v.chars().flat_map(|c| [c, c]).collect();
            return Self::hex(&expanded);
        }
        let n = u32::from_str_radix(v, 16)?;
        Ok(Self(if v.len() == 6 { n | 0xff000000 } else { n }))
    }
    pub fn from_hex(value: &str) -> Result<Self> {
        Self::hex(value)
    }
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32)
    }
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_argb(255, r, g, b)
    }
    pub const fn alpha(self) -> u8 {
        (self.0 >> 24) as u8
    }
    pub const fn red(self) -> u8 {
        (self.0 >> 16) as u8
    }
    pub const fn green(self) -> u8 {
        (self.0 >> 8) as u8
    }
    pub const fn blue(self) -> u8 {
        self.0 as u8
    }
    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self((self.0 & 0x00ffffff) | (alpha as u32) << 24)
    }
    pub fn with_alpha_fraction(self, alpha: f32) -> Self {
        assert!((0. ..=1.).contains(&alpha));
        self.with_alpha((255. * alpha) as u8)
    }
}
impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.alpha() == 255 {
            write!(f, "#{:06X}", self.0 & 0x00ffffff)
        } else {
            write!(f, "#{:08X}", self.0)
        }
    }
}
impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self(if value <= 0x00ffffff {
            value | 0xff000000
        } else {
            value
        })
    }
}
pub type PageSize = Size;
pub struct PageSizes;
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextStyle {
    pub direction: Option<TextDirection>,
    pub font_families: Option<Vec<String>>,
    pub size: Option<f32>,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub weight: Option<i32>,
    pub italic: Option<bool>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub word_spacing: Option<f32>,
    pub decoration: Option<i32>,
    pub decoration_mask: Option<i32>,
    pub decoration_color: Option<Color>,
    pub decoration_thickness: Option<f32>,
    pub decoration_style: Option<i32>,
    pub baseline_offset: Option<f32>,
    pub font_position: Option<FontPosition>,
    pub break_anywhere: Option<bool>,
    pub features: Option<Vec<(String, i32)>>,
}
impl TextStyle {
    pub fn library_default() -> Self {
        Self::default()
            .font_family("Lato")
            .font_size(12.)
            .font_color(Color::BLACK)
            .weight(400)
            .font_feature("liga", false)
    }
    pub fn merge(&self, over: &Self) -> Self {
        macro_rules! merge {($($f:ident),*)=>{Self{$($f:over.$f.clone().or_else(||self.$f.clone()),)*}}}
        let mut merged = merge!(
            direction,
            font_families,
            size,
            color,
            background,
            weight,
            italic,
            line_height,
            letter_spacing,
            word_spacing,
            decoration,
            decoration_mask,
            decoration_color,
            decoration_thickness,
            decoration_style,
            baseline_offset,
            font_position,
            break_anywhere,
            features
        );
        if let Some(families) = &over.font_families {
            let mut names = families.clone();
            for name in self.font_families.iter().flatten() {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
            merged.font_families = Some(names);
        }
        if let Some(features) = &over.features {
            let mut values = features.clone();
            for feature in self.features.iter().flatten() {
                if !values.iter().any(|f| f.0 == feature.0) {
                    values.push(feature.clone());
                }
            }
            merged.features = Some(values);
        }
        if let Some(bits) = over.decoration {
            let mask = over.decoration_mask.unwrap_or(7);
            merged.decoration = Some((self.decoration.unwrap_or(0) & !mask) | (bits & mask));
            merged.decoration_mask = Some(
                mask | self
                    .decoration
                    .map(|_| self.decoration_mask.unwrap_or(7))
                    .unwrap_or(0),
            );
        }
        merged
    }
    pub fn font_family(mut self, v: impl Into<String>) -> Self {
        self.font_families = Some(vec![v.into()]);
        self
    }
    pub fn font_size(mut self, v: f32) -> Self {
        self.size = Some(v);
        self
    }
    pub fn font_color(mut self, v: Color) -> Self {
        self.color = Some(v);
        self
    }
    pub fn weight(mut self, v: i32) -> Self {
        self.weight = Some(v);
        self
    }
    pub fn bold(self) -> Self {
        self.weight(700)
    }
    pub fn semi_bold(self) -> Self {
        self.weight(600)
    }
    pub fn extra_black(self) -> Self {
        self.weight(1000)
    }
    pub fn italic(mut self) -> Self {
        self.italic = Some(true);
        self
    }
    pub fn line_height(mut self, v: f32) -> Self {
        self.line_height = Some(v);
        self
    }
    pub fn font_families(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.font_families = Some(values.into_iter().map(Into::into).collect());
        self
    }
    pub fn background_color(mut self, v: Color) -> Self {
        self.background = Some(v);
        self
    }
    pub fn letter_spacing(mut self, v: f32) -> Self {
        self.letter_spacing = Some(v);
        self
    }
    pub fn word_spacing(mut self, v: f32) -> Self {
        self.word_spacing = Some(v);
        self
    }
    pub fn break_anywhere(mut self, v: bool) -> Self {
        self.break_anywhere = Some(v);
        self
    }
    pub fn normal_position(mut self) -> Self {
        self.font_position = Some(FontPosition::Normal);
        self
    }
    pub fn subscript(mut self) -> Self {
        self.font_position = Some(FontPosition::Subscript);
        self
    }
    pub fn superscript(mut self) -> Self {
        self.font_position = Some(FontPosition::Superscript);
        self
    }
    pub fn thin(self) -> Self {
        self.weight(100)
    }
    pub fn extra_light(self) -> Self {
        self.weight(200)
    }
    pub fn light(self) -> Self {
        self.weight(300)
    }
    pub fn normal_weight(self) -> Self {
        self.weight(400)
    }
    pub fn medium(self) -> Self {
        self.weight(500)
    }
    pub fn extra_bold(self) -> Self {
        self.weight(800)
    }
    pub fn black(self) -> Self {
        self.weight(900)
    }
    pub fn decoration(mut self, mask: i32, enabled: bool) -> Self {
        self.decoration_mask = Some(self.decoration_mask.unwrap_or(0) | mask);
        let current = self.decoration.unwrap_or(0);
        self.decoration = Some(if enabled {
            current | mask
        } else {
            current & !mask
        });
        self
    }
    pub fn underline(self) -> Self {
        self.decoration(1, true)
    }
    pub fn overline(self) -> Self {
        self.decoration(2, true)
    }
    pub fn strikethrough(self) -> Self {
        self.decoration(4, true)
    }
    pub fn decoration_color(mut self, v: Color) -> Self {
        self.decoration_color = Some(v);
        self
    }
    pub fn decoration_thickness(mut self, v: f32) -> Self {
        self.decoration_thickness = Some(v);
        self
    }
    pub fn decoration_style(mut self, v: TextDecorationStyle) -> Self {
        self.decoration_style = Some(v as i32);
        self
    }
    pub fn decoration_solid(self) -> Self {
        self.decoration_style(TextDecorationStyle::Solid)
    }
    pub fn decoration_double(self) -> Self {
        self.decoration_style(TextDecorationStyle::Double)
    }
    pub fn decoration_dotted(self) -> Self {
        self.decoration_style(TextDecorationStyle::Dotted)
    }
    pub fn decoration_dashed(self) -> Self {
        self.decoration_style(TextDecorationStyle::Dashed)
    }
    pub fn decoration_wavy(self) -> Self {
        self.decoration_style(TextDecorationStyle::Wavy)
    }
    pub fn direction_auto(mut self) -> Self {
        self.direction = Some(TextDirection::Auto);
        self
    }
    pub fn direction_from_left_to_right(mut self) -> Self {
        self.direction = Some(TextDirection::LeftToRight);
        self
    }
    pub fn direction_from_right_to_left(mut self) -> Self {
        self.direction = Some(TextDirection::RightToLeft);
        self
    }
    pub fn wrap_anywhere(self, value: bool) -> Self {
        self.break_anywhere(value)
    }
    pub fn font_type(self, value: impl Into<String>) -> Self {
        self.font_family(value)
    }
    pub fn fallback(mut self, value: &TextStyle) -> Self {
        let names = self.font_families.get_or_insert_with(Vec::new);
        for name in value.font_families.iter().flatten() {
            if !name.trim().is_empty() && !names.contains(name) {
                names.push(name.clone());
            }
        }
        self
    }
    pub fn font_feature(mut self, name: impl Into<String>, enabled: bool) -> Self {
        let name = name.into();
        let features = self.features.get_or_insert_with(Vec::new);
        features.retain(|v| v.0 != name);
        features.insert(0, (name, i32::from(enabled)));
        self
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontPosition {
    Normal,
    Subscript,
    Superscript,
}
#[derive(Clone, Copy, Debug)]
pub struct BoxShadowStyle {
    pub offset_x: f32,
    pub offset_y: f32,
    pub spread: f32,
    pub blur: f32,
    pub color: Color,
}
impl Default for BoxShadowStyle {
    fn default() -> Self {
        Self {
            offset_x: 0.,
            offset_y: 0.,
            spread: 0.,
            blur: 0.,
            color: Color(0xff9e9e9e),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecorationStyle {
    Solid = 0,
    Double = 1,
    Dotted = 2,
    Dashed = 3,
    Wavy = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    Auto,
    LeftToRight,
    RightToLeft,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
    ExtraBlack = 1000,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LicenseType {
    Evaluation,
    Community,
    Professional,
    Enterprise,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    Webp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ImageCompressionQuality {
    Best,
    VeryHigh,
    #[default]
    High,
    Medium,
    Low,
    VeryLow,
}
impl ImageCompressionQuality {
    pub fn quality(self) -> i32 {
        match self {
            Self::Best => 100,
            Self::VeryHigh => 90,
            Self::High => 75,
            Self::Medium => 50,
            Self::Low => 25,
            Self::VeryLow => 10,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ImageGenerationSettings {
    pub image_format: ImageFormat,
    pub image_compression_quality: ImageCompressionQuality,
    pub raster_dpi: f32,
    pub use_transparent_background: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct GenerateDynamicImagePayload {
    pub available_space: Size,
    pub image_size: ImageSize,
    pub dpi: f32,
}
#[derive(Clone, Debug)]
pub struct PageElementLocation {
    pub id: String,
    pub page_number: usize,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub transform: [f32; 16],
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct FontInfo {
    pub family_name: String,
    pub postscript_name: String,
    pub weight: i32,
    pub is_italic: bool,
    pub is_variable: bool,
}
impl Default for ImageGenerationSettings {
    fn default() -> Self {
        Self {
            image_format: ImageFormat::Png,
            image_compression_quality: ImageCompressionQuality::High,
            raster_dpi: 288.,
            use_transparent_background: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlignment {
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
    #[default]
    Start = 4,
    End = 5,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}
#[derive(Clone, Copy, Debug)]
pub enum AspectRatioOption {
    FitWidth,
    FitHeight,
    FitArea,
}
#[derive(Clone, Debug)]
pub enum SpanContent {
    Text(String),
    CurrentPageNumber,
    TotalPages,
    SectionPageNumber(String, SectionPageNumber),
    Element(Element, TextInjectedElementAlignment),
    Hyperlink(String, String),
    SectionLink(String, String),
}
#[derive(Clone, Copy, Debug, Default)]
pub enum TextInjectedElementAlignment {
    #[default]
    AboveBaseline = 1,
    BelowBaseline = 2,
    Top = 3,
    Bottom = 4,
    Middle = 5,
}
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub content: SpanContent,
    pub style: TextStyle,
    pub formatter: Option<PageNumberFormatter>,
}
#[derive(Clone, Copy, Debug)]
pub enum SectionPageNumber {
    Begin,
    End,
    Current,
    Total,
}
#[derive(Clone)]
pub struct PageNumberFormatter(pub std::rc::Rc<dyn Fn(Option<i32>) -> String>);
impl std::fmt::Debug for PageNumberFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Formatter({:p})", std::rc::Rc::as_ptr(&self.0))
    }
}
#[derive(Clone, Debug, Default)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub keywords: String,
    pub creator: String,
    pub producer: String,
    pub language: String,
    pub creation_date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub modified_date: Option<chrono::DateTime<chrono::FixedOffset>>,
}
#[derive(Clone, Debug)]
pub struct DocumentSettings {
    pub content_direction: ContentDirection,
    pub compress: bool,
    pub raster_dpi: f32,
    pub max_pages: usize,
    pub throw_on_missing_glyphs: bool,
    pub throw_on_missing_fonts: bool,
    pub pdfa_conformance: PdfAConformance,
    pub pdfua_conformance: PdfUaConformance,
    pub image_compression_quality: ImageCompressionQuality,
}
#[derive(Clone, Copy, Debug)]
pub struct ShowIfContext {
    pub page_number: usize,
    pub total_pages: Option<usize>,
}
impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            content_direction: ContentDirection::default(),
            compress: true,
            raster_dpi: 288.,
            max_pages: 10_000,
            throw_on_missing_glyphs: true,
            throw_on_missing_fonts: true,
            pdfa_conformance: PdfAConformance::None,
            pdfua_conformance: PdfUaConformance::None,
            image_compression_quality: ImageCompressionQuality::High,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PdfAConformance {
    #[default]
    None = 0,
    PdfA2A = 3,
    PdfA2B = 4,
    PdfA2U = 5,
    PdfA3A = 6,
    PdfA3B = 7,
    PdfA3U = 8,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PdfUaConformance {
    #[default]
    None = 0,
    PdfUa1 = 1,
}
pub trait IDocument {
    fn compose(&self, container: &mut crate::DocumentContainer);
    fn metadata(&self) -> DocumentMetadata {
        DocumentMetadata::default()
    }
    fn settings(&self) -> DocumentSettings {
        DocumentSettings::default()
    }
    fn page_number_groups(&self) -> Option<Vec<usize>> {
        None
    }
    fn continuous_page_numbers(&self) -> bool {
        true
    }
    fn get_metadata(&self) -> DocumentMetadata {
        self.metadata()
    }
    fn get_settings(&self) -> DocumentSettings {
        self.settings()
    }
    fn generate_pdf_bytes(&self) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        crate::Document::from_document(self).generate_pdf_bytes()
    }
    fn generate_pdf(&self, path: impl AsRef<std::path::Path>) -> Result<()>
    where
        Self: Sized,
    {
        crate::Document::from_document(self).generate_pdf(path)
    }
    fn generate_images(&self, dpi: f32) -> Result<Vec<Vec<u8>>>
    where
        Self: Sized,
    {
        crate::Document::from_document(self).generate_images(dpi)
    }
    fn generate_svg(&self) -> Result<Vec<String>>
    where
        Self: Sized,
    {
        crate::Document::from_document(self).generate_svg()
    }
}
pub trait IComponent {
    fn compose(&self, container: crate::Container);
}
#[derive(Clone)]
pub struct Page {
    pub size: Size,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub continuous: bool,
    pub margins: [f32; 4],
    pub style: TextStyle,
    pub direction: ContentDirection,
    pub direction_override: Option<ContentDirection>,
    pub color: Color,
    pub header: Element,
    pub content: Element,
    pub footer: Element,
    pub background: Element,
    pub foreground: Element,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            size: PageSizes::A4,
            min_size: None,
            max_size: None,
            continuous: false,
            margins: [0.; 4],
            style: TextStyle::default(),
            direction: ContentDirection::default(),
            direction_override: None,
            color: Color::TRANSPARENT,
            header: Element::empty(),
            content: Element::empty(),
            footer: Element::empty(),
            background: Element::empty(),
            foreground: Element::empty(),
        }
    }
}
