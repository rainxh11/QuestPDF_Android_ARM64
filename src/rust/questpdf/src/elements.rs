//! Stateful Measure/Draw layout elements, ported from QuestPDF/Elements.
use crate::{
    infrastructure::*,
    native::{self, Handle, Ptr},
};
use anyhow::{Result, bail};
use std::{cell::RefCell, ptr, rc::Rc};

#[derive(Clone)]
pub struct Element(Rc<RefCell<Node>>);
impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Element({:p})", Rc::as_ptr(&self.0))
    }
}
pub(crate) struct Node {
    pub kind: Kind,
    state: State,
    paragraph: Option<Paragraph>,
    image: Option<Media>,
    fit: Option<AspectRatioOption>,
    stretch: bool,
    use_original_image: bool,
    image_no_resize: bool,
    image_dpi: Option<f32>,
    image_quality: Option<ImageCompressionQuality>,
    dynamic_media: Vec<(Size, Element)>,
    semantic_id: Option<usize>,
    semantic_snapshots: Vec<usize>,
    repeat_optimized: bool,
    snapshot_enabled: bool,
    snapshot_draws: std::collections::HashMap<usize, Vec<(i32, Handle)>>,
    snapshot_measurements: std::collections::HashMap<(usize, u32, u32), SpacePlan>,
    debug_label: Option<String>,
    lazy_cacheable: bool,
    layout_locations: Vec<crate::companion::PageLocation>,
    box_parent: Option<std::rc::Weak<RefCell<Node>>>,
}
#[derive(Default, Clone)]
struct State {
    index: usize,
    line: usize,
    rendered: bool,
    visited: bool,
}
struct Paragraph {
    handle: Handle,
    lines: Vec<native::LineExtent>,
    width: f32,
    total_width: f32,
    key: String,
    whitespace: bool,
    placeholders: Vec<native::Rect>,
    injected: Vec<(usize, Element)>,
    links: Vec<(usize, usize, String, String, bool)>,
}
struct Media {
    handle: Handle,
    size: Size,
    svg_size: Option<native::SvgSize>,
    viewbox: native::Rect,
    encoded_length: i32,
    font_revision: u64,
    versions: Vec<(i32, i32, ImageCompressionQuality, Handle)>,
}
#[derive(Default)]
pub(crate) struct TextBlock {
    pub spans: Vec<TextSpan>,
    pub style: TextStyle,
    pub alignment: TextAlignment,
    pub clamp: Option<(usize, String)>,
    pub paragraph_spacing: f32,
    pub indentation: f32,
}
#[derive(Clone, Copy)]
pub(crate) enum RowItemType {
    Relative,
    Constant,
    Auto,
}
pub(crate) struct RowItem {
    pub element: Element,
    pub kind: RowItemType,
    pub value: f32,
    pub rendered: bool,
}
#[derive(Clone)]
pub(crate) enum ColumnDefinition {
    Relative(f32),
    Constant(f32),
}
#[derive(Clone, Copy)]
pub(crate) struct CellPosition {
    pub column: usize,
    pub row: usize,
    pub column_span: usize,
    pub row_span: usize,
    pub semantic_horizontal_header: bool,
}
impl Default for CellPosition {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            column_span: 1,
            row_span: 1,
            semantic_horizontal_header: false,
        }
    }
}
pub(crate) struct TableCell {
    pub element: Element,
    pub position: Rc<RefCell<CellPosition>>,
    pub rendered: std::cell::Cell<bool>,
    pub semantic_id: std::cell::Cell<Option<usize>>,
}
impl TableCell {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            position: Rc::new(RefCell::new(CellPosition::default())),
            rendered: std::cell::Cell::new(false),
            semantic_id: std::cell::Cell::new(None),
        }
    }
}
#[derive(Default)]
pub(crate) struct Table {
    pub extend_last_cells: bool,
    pub columns: Vec<ColumnDefinition>,
    pub cells: Vec<TableCell>,
    pub header: Vec<TableCell>,
    pub footer: Vec<TableCell>,
    pub semantic_sections: [std::cell::Cell<Option<usize>>; 3],
    pub header_repeat: std::cell::Cell<bool>,
    pub footer_repeat: std::cell::Cell<bool>,
}
#[derive(Clone, Default)]
pub(crate) struct BoxStyle {
    pub background: Color,
    pub border_color: Color,
    pub border: [f32; 4],
    pub radius: [f32; 4],
    pub extended: bool,
    pub border_alignment: Option<f32>,
    pub background_gradient: Vec<Color>,
    pub background_angle: f32,
    pub border_gradient: Vec<Color>,
    pub border_angle: f32,
    pub shadow: Option<BoxShadowStyle>,
}
#[derive(Clone, Copy, Default)]
pub enum InlinedAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
    SpaceAround,
}
pub(crate) struct Inlined {
    pub items: Vec<Element>,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
    pub alignment: Option<InlinedAlignment>,
    pub baseline: VerticalAlignment,
}
impl Default for Inlined {
    fn default() -> Self {
        Self {
            items: vec![],
            horizontal_spacing: 0.,
            vertical_spacing: 0.,
            alignment: None,
            baseline: VerticalAlignment::Top,
        }
    }
}
pub(crate) enum Kind {
    Empty,
    Text(TextBlock),
    Column(Vec<Element>, f32),
    Inlined(Inlined),
    Row(Vec<RowItem>, f32),
    Table(Table),
    Padding([f32; 4], Element),
    Constrained {
        min: Size,
        max: Size,
        child: Element,
    },
    Alignment(
        Option<HorizontalAlignment>,
        Option<VerticalAlignment>,
        Element,
    ),
    DefaultTextStyle(TextStyle, Element),
    Direction(ContentDirection, Element),
    Hyperlink(String, Option<String>, Element),
    Section(String, Element),
    SectionLink(String, Option<String>, Element),
    StyledBox(BoxStyle, Element),
    AspectRatio(f32, AspectRatioOption, Element),
    ScaleToFit(Element),
    Scale(f32, f32, Element),
    Rotate(f32, Element),
    RotateLayout(i32, Element),
    Offset(f32, f32, Element),
    Extend(bool, bool, Element),
    Shrink(bool, bool, Element),
    ShowEntire(Element),
    EnsureSpace(f32, Element),
    ShowOnce(Element),
    SkipOnce(Element),
    Repeat(i32, Element),
    ShowIfPredicate(Rc<dyn Fn(ShowIfContext) -> bool>, Element),
    Unconstrained(Element),
    StopPaging(Element),
    PreventPageBreak(Element),
    Layers(Vec<(bool, Element)>),
    Decoration {
        before: Element,
        content: Element,
        after: Element,
    },
    MultiColumn {
        content: Element,
        spacer: Element,
        count: usize,
        spacing: f32,
        balance: bool,
    },
    PageBreak,
    Image(Vec<u8>),
    Svg(String),
    SvgPath(String, Color),
    DynamicSvg(Rc<dyn Fn(Size) -> String>),
    DynamicImage(Rc<dyn Fn(GenerateDynamicImagePayload) -> Option<Vec<u8>>>),
    Lazy {
        source: Rc<dyn Fn(crate::Container)>,
        child: Option<Element>,
        retain_content: bool,
    },
    Dynamic(Box<dyn crate::dynamic::ErasedComponent>),
    Proxy(Element),
    ZIndex(i32, Element),
    CapturePosition(String, Element),
    Semantic(crate::SemanticInfo, Element),
    Artifact(i32, Element),
    Line {
        width: f32,
        color: Color,
        vertical: bool,
        dash: Vec<f32>,
        gradient: Vec<Color>,
    },
}
pub(crate) struct Context<'a> {
    pub collect_layout: bool,
    pub document_id: Option<usize>,
    pub fonts: &'a native::Fonts,
    pub page: usize,
    pub total: usize,
    pub sections: &'a RefCell<std::collections::HashMap<String, (usize, usize)>>,
    pub captured_positions: &'a RefCell<Vec<PageElementLocation>>,
    pub semantic_tree: &'a RefCell<Option<crate::semantic::SemanticTree>>,
    pub page_size: std::cell::Cell<Size>,
    pub in_footer: std::cell::Cell<bool>,
    pub image_dpi: f32,
    pub image_quality: ImageCompressionQuality,
    pub use_original_images: bool,
    pub throw_on_missing_glyphs: bool,
    pub throw_on_missing_fonts: bool,
}
#[derive(Clone)]
pub(crate) struct Environment {
    pub style: TextStyle,
    pub direction: ContentDirection,
}
impl Context<'_> {
    fn location_name(&self, name: &str) -> String {
        self.document_id
            .map_or_else(|| name.to_owned(), |id| format!("{id}:{name}"))
    }
}
impl Element {
    pub fn empty() -> Self {
        Self(Rc::new(RefCell::new(Node {
            kind: Kind::Empty,
            state: State::default(),
            paragraph: None,
            image: None,
            fit: None,
            stretch: false,
            use_original_image: false,
            image_no_resize: false,
            image_dpi: None,
            image_quality: None,
            dynamic_media: vec![],
            semantic_id: None,
            semantic_snapshots: vec![],
            repeat_optimized: false,
            snapshot_enabled: false,
            snapshot_draws: Default::default(),
            snapshot_measurements: Default::default(),
            debug_label: None,
            lazy_cacheable: false,
            layout_locations: vec![],
            box_parent: None,
        })))
    }
    pub(crate) fn set(&self, kind: Kind) {
        let mut n = self.0.borrow_mut();
        if !matches!(n.kind, Kind::Empty) {
            panic!("A QuestPDF container can contain only one child")
        };
        match &kind {
            Kind::StyledBox(_, c)
            | Kind::Constrained { child: c, .. }
            | Kind::Padding(_, c)
            | Kind::Scale(_, _, c)
            | Kind::Offset(_, _, c)
            | Kind::Rotate(_, c)
            | Kind::RotateLayout(_, c)
            | Kind::Alignment(_, _, c) => {
                c.0.borrow_mut().box_parent = Some(Rc::downgrade(&self.0))
            }
            _ => {}
        }
        n.kind = kind;
    }
    pub(crate) fn set_debug_label(&self, label: String) {
        self.0.borrow_mut().debug_label = Some(label);
    }
    pub(crate) fn set_lazy_cacheable(&self) {
        self.0.borrow_mut().lazy_cacheable = true;
    }
    pub(crate) fn modify_transform(&self, x: f32, y: f32, scale: bool) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut parent = parent.borrow_mut();
        match &mut parent.kind {
            Kind::Scale(px, py, child) if scale => {
                *px *= x;
                *py *= y;
                Some(child.clone())
            }
            Kind::Offset(px, py, child) if !scale => {
                *px += x;
                *py += y;
                Some(child.clone())
            }
            _ => None,
        }
    }
    pub(crate) fn modify_rotation(&self, angle: f32, turns: i32, layout: bool) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut parent = parent.borrow_mut();
        match &mut parent.kind {
            Kind::Rotate(value, child) if !layout => {
                *value += angle;
                Some(child.clone())
            }
            Kind::RotateLayout(value, child) if layout => {
                *value += turns;
                Some(child.clone())
            }
            _ => None,
        }
    }
    pub(crate) fn modify_box(&self, change: &impl Fn(&mut BoxStyle)) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut p = parent.borrow_mut();
        if let Kind::StyledBox(s, c) = &mut p.kind {
            change(s);
            Some(c.clone())
        } else {
            None
        }
    }
    pub(crate) fn modify_padding(&self, values: [f32; 4]) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut p = parent.borrow_mut();
        if let Kind::Padding(p, c) = &mut p.kind {
            for (v, add) in p.iter_mut().zip(values) {
                *v += add
            }
            Some(c.clone())
        } else {
            None
        }
    }
    pub(crate) fn modify_constraints(&self, values: [Option<f32>; 4]) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut p = parent.borrow_mut();
        if let Kind::Constrained { min, max, child } = &mut p.kind {
            if let Some(v) = values[0] {
                min.width = v;
            }
            if let Some(v) = values[1] {
                max.width = v;
            }
            if let Some(v) = values[2] {
                min.height = v;
            }
            if let Some(v) = values[3] {
                max.height = v;
            }
            Some(child.clone())
        } else {
            None
        }
    }
    pub(crate) fn modify_alignment(
        &self,
        h: Option<HorizontalAlignment>,
        v: Option<VerticalAlignment>,
    ) -> Option<Element> {
        let parent = self.0.borrow().box_parent.clone()?.upgrade()?;
        let mut p = parent.borrow_mut();
        if let Kind::Alignment(ph, pv, c) = &mut p.kind {
            if h.is_some() {
                *ph = h
            }
            if v.is_some() {
                *pv = v
            }
            Some(c.clone())
        } else {
            None
        }
    }
    pub(crate) fn add_span(&self, span: TextSpan) -> usize {
        if let Kind::Text(t) = &mut self.0.borrow_mut().kind {
            t.spans.push(span);
            t.spans.len() - 1
        } else {
            unreachable!()
        }
    }
    pub(crate) fn change_text_style(
        &self,
        index: Option<usize>,
        change: impl FnOnce(&mut TextStyle),
    ) {
        if let Kind::Text(t) = &mut self.0.borrow_mut().kind {
            change(if let Some(i) = index {
                &mut t.spans[i].style
            } else {
                &mut t.style
            })
        }
    }
    pub(crate) fn align_text(&self, a: TextAlignment) {
        if let Kind::Text(t) = &mut self.0.borrow_mut().kind {
            t.alignment = a
        }
    }
    pub(crate) fn configure_text(&self, change: impl FnOnce(&mut TextBlock)) {
        if let Kind::Text(t) = &mut self.0.borrow_mut().kind {
            change(t);
        }
    }
    pub(crate) fn format_page_number(&self, index: usize, format: PageNumberFormatter) {
        self.configure_text(|t| t.spans[index].formatter = Some(format));
    }
    pub(crate) fn set_image_fit(&self, a: AspectRatioOption) {
        self.0.borrow_mut().fit = Some(a)
    }
    pub(crate) fn configure_image(
        &self,
        original: Option<bool>,
        dpi: Option<f32>,
        quality: Option<ImageCompressionQuality>,
        stretch: Option<bool>,
    ) {
        let mut n = self.0.borrow_mut();
        if let Some(v) = original {
            n.use_original_image = v;
        }
        if dpi.is_some() {
            n.image_dpi = dpi;
        }
        if quality.is_some() {
            n.image_quality = quality;
        }
        if let Some(v) = stretch {
            n.stretch = v;
        }
    }
    pub(crate) fn infer_image_fit(&self) -> AspectRatioOption {
        if let Some(p) = self
            .0
            .borrow()
            .box_parent
            .as_ref()
            .and_then(|p| p.upgrade())
            && let Kind::Constrained { min, max, .. } = &p.borrow().kind
        {
            let w = min.width > 0. || max.width < INFINITY;
            let h = min.height > 0. || max.height < INFINITY;
            return if w && h {
                AspectRatioOption::FitArea
            } else if h {
                AspectRatioOption::FitHeight
            } else {
                AspectRatioOption::FitWidth
            };
        }
        AspectRatioOption::FitWidth
    }
    pub(crate) fn infer_text_alignment(&self) -> TextAlignment {
        if let Some(p) = self
            .0
            .borrow()
            .box_parent
            .as_ref()
            .and_then(|p| p.upgrade())
            && let Kind::Alignment(Some(h), _, _) = &p.borrow().kind
        {
            return match h {
                HorizontalAlignment::Left => TextAlignment::Left,
                HorizontalAlignment::Center => TextAlignment::Center,
                HorizontalAlignment::Right => TextAlignment::Right,
            };
        }
        TextAlignment::Start
    }
    pub(crate) fn line_color(&self, a: Color) {
        if let Kind::Line { color, .. } = &mut self.0.borrow_mut().kind {
            *color = a
        }
    }
    pub(crate) fn configure_line(&self, dash: Option<Vec<f32>>, gradient: Option<Vec<Color>>) {
        if let Kind::Line {
            dash: d,
            gradient: g,
            ..
        } = &mut self.0.borrow_mut().kind
        {
            if let Some(v) = dash {
                *d = v;
            }
            if let Some(v) = gradient {
                *g = v;
            }
        }
    }
    fn snapshot(&self) -> Vec<ElementSnapshot> {
        let n = self.0.borrow();
        let row = if let Kind::Row(v, _) = &n.kind {
            v.iter().map(|i| i.rendered).collect()
        } else {
            vec![]
        };
        let table = if let Kind::Table(t) = &n.kind {
            t.cells
                .iter()
                .chain(&t.header)
                .chain(&t.footer)
                .map(|i| i.rendered.get())
                .collect()
        } else {
            vec![]
        };
        let mut states = vec![ElementSnapshot {
            element: self.clone(),
            state: n.state.clone(),
            row,
            table,
            dynamic_state: if let Kind::Dynamic(component) = &n.kind {
                Some(component.snapshot())
            } else {
                None
            },
            table_repeat: if let Kind::Table(t) = &n.kind {
                Some([t.header_repeat.get(), t.footer_repeat.get()])
            } else {
                None
            },
        }];
        // MultiColumn registers its state observers before a Lazy subtree exists.
        // Match the source: Lazy state is observed, dynamically created descendants are not.
        let children = if matches!(n.kind, Kind::Lazy { .. }) {
            vec![]
        } else {
            n.children()
        };
        drop(n);
        for child in children {
            states.extend(child.snapshot())
        }
        states
    }
    pub(crate) fn reset(&self, repeat: bool) {
        let mut n = self.0.borrow_mut();
        if repeat && matches!(n.kind, Kind::ShowOnce(_) | Kind::SkipOnce(_)) {
            return;
        }
        if !(repeat
            && matches!(
                n.kind,
                Kind::EnsureSpace(..)
                    | Kind::PreventPageBreak(..)
                    | Kind::Section(..)
                    | Kind::Lazy { .. }
                    | Kind::Repeat(..)
            ))
        {
            n.state = State::default()
        }
        if let Kind::Row(items, _) = &mut n.kind {
            for i in items {
                i.rendered = false;
                if matches!(i.kind, RowItemType::Auto) {
                    i.value = 0.;
                }
            }
        }
        if let Kind::Dynamic(component) = &mut n.kind {
            component.reset();
        }
        if let Kind::Table(t) = &n.kind {
            if !repeat {
                t.header_repeat.set(false);
                t.footer_repeat.set(false);
            }
            for cell in t.cells.iter().chain(&t.header).chain(&t.footer) {
                cell.rendered.set(false)
            }
        }
        let children = n.children();
        drop(n);
        for c in children {
            c.reset(repeat)
        }
    }
    pub(crate) fn measure(&self, s: Size, c: &Context, e: &Environment) -> Result<SpacePlan> {
        let mut n = self.0.borrow_mut();
        let key = (c.page, s.width.to_bits(), s.height.to_bits());
        if n.snapshot_enabled
            && let Some(plan) = n.snapshot_measurements.get(&key)
        {
            return Ok(*plan);
        }
        let plan = n.measure(s, c, e)?;
        if n.snapshot_enabled {
            n.snapshot_measurements.insert(key, plan);
        }
        Ok(plan)
    }
    pub(crate) fn prepare_caching(&self) {
        self.clear_snapshots();
        if self.traverse_caching() {
            self.0.borrow_mut().snapshot_enabled = true;
        }
    }
    pub(crate) fn clear_snapshots(&self) {
        let mut n = self.0.borrow_mut();
        n.snapshot_enabled = false;
        n.snapshot_draws.clear();
        n.snapshot_measurements.clear();
        let children = n.children();
        drop(n);
        for child in children {
            child.clear_snapshots();
        }
    }
    fn traverse_caching(&self) -> bool {
        let n = self.0.borrow();
        match &n.kind {
            Kind::Dynamic(_) => return false,
            Kind::Lazy { .. } => return n.lazy_cacheable,
            Kind::Text(t)
                if t.spans.iter().any(|s| {
                    matches!(
                        s.content,
                        SpanContent::CurrentPageNumber
                            | SpanContent::TotalPages
                            | SpanContent::SectionPageNumber(..)
                    )
                }) =>
            {
                return false;
            }
            _ => (),
        }
        let multi = matches!(n.kind, Kind::MultiColumn { .. });
        let auto_row = matches!(&n.kind, Kind::Row(items, _) if items.iter().any(|i| matches!(i.kind, RowItemType::Auto)));
        let children = n.children();
        drop(n);
        let cacheable: Vec<_> = children.iter().map(Element::traverse_caching).collect();
        if multi {
            for child in &children {
                child.clear_snapshots();
            }
        }
        if cacheable.iter().all(|v| *v) {
            return true;
        }
        if auto_row {
            for child in &children {
                child.clear_snapshots();
            }
            return false;
        }
        if !multi {
            for (child, cacheable) in children.iter().zip(cacheable) {
                if cacheable {
                    child.0.borrow_mut().snapshot_enabled = true;
                }
            }
        }
        false
    }
    fn is_cacheable(&self) -> bool {
        let n = self.0.borrow();
        match &n.kind {
            Kind::Dynamic(_) => false,
            Kind::Lazy { .. } => n.lazy_cacheable,
            Kind::Text(t)
                if t.spans.iter().any(|s| {
                    matches!(
                        s.content,
                        SpanContent::CurrentPageNumber
                            | SpanContent::TotalPages
                            | SpanContent::SectionPageNumber(..)
                    )
                }) =>
            {
                false
            }
            _ => n.children().iter().all(Element::is_cacheable),
        }
    }
    pub(crate) fn draw(&self, s: Size, canvas: Ptr, c: &Context, e: &Environment) -> Result<()> {
        if c.collect_layout && !canvas.is_null() {
            let intrinsic = matches!(
                self.0.borrow().kind,
                Kind::Text(_)
                    | Kind::AspectRatio(..)
                    | Kind::Unconstrained(_)
                    | Kind::Semantic(..)
                    | Kind::Artifact(..)
            );
            let size = if intrinsic {
                self.measure(s, c, e)?.size
            } else {
                s
            };
            self.draw_uncached(s, canvas, c, e)?;
            let mut matrix = native::Matrix::default();
            unsafe {
                c.fonts
                    .api
                    .questpdf_skia_canvas_get_matrix9(canvas, &mut matrix);
            }
            self.0
                .borrow_mut()
                .layout_locations
                .push(crate::companion::PageLocation::from_matrix(
                    c.page, size, matrix,
                ));
            return Ok(());
        }
        if !canvas.is_null() && self.0.borrow().snapshot_enabled {
            if let Some(snapshot) = self.0.borrow_mut().snapshot_draws.remove(&c.page) {
                return crate::canvas::draw_snapshot(canvas, &snapshot, &c.fonts.api);
            }
            let recording = crate::canvas::CanvasSession::new(&c.fonts.api, Size::MAX)?;
            crate::canvas::set_z_index(recording.ptr, crate::canvas::z_index(canvas))?;
            crate::canvas::set_semantic_id(recording.ptr, crate::canvas::semantic_id(canvas));
            let mut matrix = native::Matrix::default();
            unsafe {
                c.fonts
                    .api
                    .questpdf_skia_canvas_get_matrix9(canvas, &mut matrix);
                c.fonts
                    .api
                    .questpdf_skia_canvas_set_matrix9(recording.ptr, &matrix);
            }
            self.draw_uncached(s, recording.ptr, c, e)?;
            let snapshot = recording.finish_snapshot()?;
            self.0.borrow_mut().snapshot_draws.insert(c.page, snapshot);
            return Ok(());
        }
        self.draw_uncached(s, canvas, c, e)
    }
    fn draw_uncached(&self, s: Size, canvas: Ptr, c: &Context, e: &Environment) -> Result<()> {
        let (mut info, existing_id, page_number_artifact) = {
            let n = self.0.borrow();
            let tree = c.semantic_tree.borrow();
            let suppressed = tree.as_ref().is_none_or(|t| t.artifacts > 0);
            let info = if suppressed || canvas.is_null() {
                None
            } else {
                match &n.kind {
                    Kind::Semantic(info, _) => Some(info.clone()),
                    Kind::Text(_) if !tree.as_ref().unwrap().paragraph_suppressed() => {
                        Some(crate::SemanticInfo::new("P"))
                    }
                    _ => None,
                }
            };
            let page_number_artifact = c.in_footer.get()
                && matches!(&n.kind,Kind::Text(t) if t.spans.iter().any(|s|matches!(s.content,SpanContent::CurrentPageNumber|SpanContent::TotalPages|SpanContent::SectionPageNumber(..))));
            (info, n.semantic_id, page_number_artifact)
        };
        if page_number_artifact && !canvas.is_null() && c.semantic_tree.borrow().is_some() {
            let _scope = crate::semantic::Scope::artifact(c, canvas, -2);
            return self.0.borrow_mut().draw(s, canvas, c, e);
        }
        let mut scope = None;
        if let Some(info) = info.as_mut() {
            if matches!(
                info.kind.as_str(),
                "H" | "H1" | "H2" | "H3" | "H4" | "H5" | "H6"
            ) && info
                .alternative_text
                .as_ref()
                .is_none_or(|t| t.trim().is_empty())
            {
                info.alternative_text = Some(self.text_content());
            }
            if info.kind == "Link"
                && let Some(alt) = &info.alternative_text
            {
                self.assign_link_description(alt);
            }
            if c.total == 0
                && matches!(info.kind.as_str(), "Figure" | "Formula")
                && self.measure(s, c, e)?.kind == SpacePlanType::FullRender
            {
                let mut matrix = native::Matrix::default();
                unsafe {
                    c.fonts
                        .api
                        .questpdf_skia_canvas_get_matrix9(canvas, &mut matrix);
                }
                let points =
                    [(0., 0.), (s.width, 0.), (s.width, s.height), (0., s.height)].map(|(x, y)| {
                        (
                            x * matrix.scale_x + y * matrix.skew_x + matrix.translate_x,
                            x * matrix.skew_y + y * matrix.scale_y + matrix.translate_y,
                        )
                    });
                let left = points.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
                let right = points.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
                let top = points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
                let bottom = points.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);
                info.attributes.push(crate::SemanticAttribute {
                    owner: "Layout".into(),
                    name: "BBox".into(),
                    value: crate::SemanticAttributeValue::FloatArray(vec![
                        left.floor(),
                        (c.page_size.get().height - bottom).floor(),
                        right.ceil(),
                        (c.page_size.get().height - top).ceil(),
                    ]),
                });
            }
            let id = if let Some(id) = existing_id {
                id
            } else {
                let id = c
                    .semantic_tree
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .register(info.clone(), c.total == 0);
                self.0.borrow_mut().semantic_id = Some(id);
                id
            };
            scope = Some(crate::semantic::Scope::node(c, canvas, id));
        }
        let result = self.0.borrow_mut().draw(s, canvas, c, e);
        drop(scope);
        result
    }
    pub(crate) fn clear_semantics(&self) {
        let mut n = self.0.borrow_mut();
        n.semantic_id = None;
        n.semantic_snapshots.clear();
        n.layout_locations.clear();
        if let Kind::Table(t) = &n.kind {
            for section in &t.semantic_sections {
                section.set(None);
            }
            for cell in t.cells.iter().chain(&t.header).chain(&t.footer) {
                cell.semantic_id.set(None);
            }
        }
        let children = n.children();
        drop(n);
        for child in children {
            child.clear_semantics();
        }
    }
    pub(crate) fn retain_lazy_content(&self) {
        let mut n = self.0.borrow_mut();
        if let Kind::Lazy { retain_content, .. } = &mut n.kind {
            *retain_content = true;
        }
        let children = n.children();
        drop(n);
        for child in children {
            child.retain_lazy_content();
        }
    }
    fn text_content(&self) -> String {
        let n = self.0.borrow();
        if let Kind::Text(t) = &n.kind {
            return t
                .spans
                .iter()
                .filter_map(|s| match &s.content {
                    SpanContent::Text(t)
                    | SpanContent::Hyperlink(t, _)
                    | SpanContent::SectionLink(t, _) => Some(t.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
        n.children()
            .iter()
            .map(Element::text_content)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
    fn assign_link_description(&self, description: &str) {
        let mut n = self.0.borrow_mut();
        if let Kind::Hyperlink(_, alt, _) | Kind::SectionLink(_, alt, _) = &mut n.kind {
            *alt = Some(description.into());
            return;
        }
        let children = n.children();
        let descend = matches!(
            n.kind,
            Kind::Semantic(..)
                | Kind::Proxy(..)
                | Kind::Padding(..)
                | Kind::Alignment(..)
                | Kind::DefaultTextStyle(..)
                | Kind::Constrained { .. }
                | Kind::StyledBox(..)
                | Kind::ZIndex(..)
        );
        drop(n);
        if descend && let Some(child) = children.first() {
            child.assign_link_description(description);
        }
    }
    pub(crate) fn debug_tree(&self, depth: usize) -> String {
        let n = self.0.borrow();
        let label = match &n.kind {
            Kind::Text(t) => format!(
                "Text {:?}",
                t.spans
                    .iter()
                    .map(|s| match &s.content {
                        SpanContent::Text(t) => t.as_str(),
                        _ => "#",
                    })
                    .collect::<String>()
            ),
            Kind::Column(..) => "Column".into(),
            Kind::Row(..) => "Row".into(),
            Kind::Table(..) => "Table".into(),
            Kind::Padding(p, ..) => format!("Padding {p:?}"),
            Kind::Constrained { min, max, .. } => format!("Constrained {min:?} {max:?}"),
            _ => "Wrapper".into(),
        };
        let label = n
            .debug_label
            .as_ref()
            .map_or(label.clone(), |value| format!("{label}: {value}"));
        let mut out = format!(
            "{}{label} state={:?}/{:?}\n",
            " ".repeat(depth),
            n.state.index,
            n.state.rendered
        );
        for child in n.children() {
            out += &child.debug_tree(depth + 1)
        }
        out
    }
    pub(crate) fn hierarchy(&self) -> crate::companion::HierarchyElement {
        let n = self.0.borrow();
        let kind = match &n.kind {
            Kind::Empty => "Empty",
            Kind::Text(_) => "TextBlock",
            Kind::Column(..) => "Column",
            Kind::Inlined(_) => "Inlined",
            Kind::Row(..) => "Row",
            Kind::Table(_) => "Table",
            Kind::Padding(..) => "Padding",
            Kind::Constrained { .. } => "Constrained",
            Kind::Alignment(..) => "Alignment",
            Kind::DefaultTextStyle(..) => "DefaultTextStyle",
            Kind::Direction(..) => "ContentDirectionSetter",
            Kind::Hyperlink(..) => "Hyperlink",
            Kind::Section(..) => "Section",
            Kind::SectionLink(..) => "SectionLink",
            Kind::StyledBox(..) => "StyledBox",
            Kind::AspectRatio(..) => "AspectRatio",
            Kind::ScaleToFit(..) => "ScaleToFit",
            Kind::Scale(..) => "Scale",
            Kind::Rotate(..) => "Rotate",
            Kind::RotateLayout(..) => "RotateLayout",
            Kind::Offset(..) => "Offset",
            Kind::Extend(..) => "Extend",
            Kind::Shrink(..) => "Shrink",
            Kind::ShowEntire(..) => "ShowEntire",
            Kind::EnsureSpace(..) => "EnsureSpace",
            Kind::ShowOnce(..) => "ShowOnce",
            Kind::SkipOnce(..) => "SkipOnce",
            Kind::Repeat(..) => "RepeatContent",
            Kind::ShowIfPredicate(..) => "ShowIf",
            Kind::Unconstrained(..) => "Unconstrained",
            Kind::StopPaging(..) => "StopPaging",
            Kind::PreventPageBreak(..) => "PreventPageBreak",
            Kind::Layers(..) => "Layers",
            Kind::Decoration { .. } => "Decoration",
            Kind::MultiColumn { .. } => "MultiColumn",
            Kind::PageBreak => "PageBreak",
            Kind::Image(_) => "Image",
            Kind::Svg(_) => "SvgImage",
            Kind::SvgPath(..) => "SvgPath",
            Kind::DynamicSvg(_) => "DynamicSvgImage",
            Kind::DynamicImage(_) => "DynamicImage",
            Kind::Lazy { .. } => "Lazy",
            Kind::Dynamic(_) => "DynamicHost",
            Kind::Proxy(_) => "Container",
            Kind::ZIndex(..) => "ZIndex",
            Kind::CapturePosition(..) => "ElementPositionLocator",
            Kind::Semantic(..) => "SemanticTag",
            Kind::Artifact(..) => "ArtifactTag",
            Kind::Line { .. } => "Line",
        };
        let children = n.children();
        let hint = n.debug_label.clone().or_else(|| match &n.kind {
            Kind::Section(value, _)
            | Kind::SectionLink(value, ..)
            | Kind::Hyperlink(value, ..)
            | Kind::CapturePosition(value, _) => Some(value.clone()),
            Kind::Semantic(info, _) => Some(info.kind.clone()),
            _ => None,
        });
        let searchable_content = if matches!(n.kind, Kind::Text(_)) {
            Some(self.text_content())
        } else {
            None
        };
        let source = hint
            .as_ref()
            .and_then(|s| s.rsplit_once(':'))
            .and_then(|(path, line)| {
                line.parse::<u32>()
                    .ok()
                    .map(|line| serde_json::json!({"filePath":path,"lineNumber":line}))
            });
        let page_locations = n.layout_locations.clone();
        drop(n);
        let mut result = crate::companion::HierarchyElement::group(
            kind,
            children.iter().map(Element::hierarchy).collect(),
        );
        result.hint = hint;
        result.searchable_content = searchable_content;
        result.page_locations = page_locations;
        result.source_code_declaration_path = source;
        result
    }
}
impl Node {
    fn children(&self) -> Vec<Element> {
        match &self.kind {
            Kind::Text(t) => t
                .spans
                .iter()
                .filter_map(|s| {
                    if let SpanContent::Element(e, _) = &s.content {
                        Some(e.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            Kind::Lazy { child, .. } => child.iter().cloned().collect(),
            Kind::Proxy(c) => vec![c.clone()],
            Kind::ZIndex(_, c) => vec![c.clone()],
            Kind::CapturePosition(_, c) => vec![c.clone()],
            Kind::Semantic(_, c) | Kind::Artifact(_, c) => vec![c.clone()],
            Kind::Column(v, _) => v.clone(),
            Kind::Inlined(v) => v.items.clone(),
            Kind::MultiColumn {
                content, spacer, ..
            } => vec![content.clone(), spacer.clone()],
            Kind::Layers(v) => v.iter().map(|i| i.1.clone()).collect(),
            Kind::Decoration {
                before,
                content,
                after,
            } => vec![before.clone(), content.clone(), after.clone()],
            Kind::Row(v, _) => v.iter().map(|i| i.element.clone()).collect(),
            Kind::Table(t) => t
                .cells
                .iter()
                .chain(&t.header)
                .chain(&t.footer)
                .map(|i| i.element.clone())
                .collect(),
            Kind::Padding(_, c)
            | Kind::Constrained { child: c, .. }
            | Kind::Alignment(_, _, c)
            | Kind::DefaultTextStyle(_, c)
            | Kind::Direction(_, c)
            | Kind::Hyperlink(_, _, c)
            | Kind::Section(_, c)
            | Kind::SectionLink(_, _, c)
            | Kind::StyledBox(_, c)
            | Kind::AspectRatio(_, _, c)
            | Kind::ScaleToFit(c)
            | Kind::Scale(_, _, c)
            | Kind::Rotate(_, c)
            | Kind::RotateLayout(_, c)
            | Kind::Offset(_, _, c)
            | Kind::Extend(_, _, c)
            | Kind::Shrink(_, _, c)
            | Kind::ShowEntire(c)
            | Kind::EnsureSpace(_, c)
            | Kind::ShowOnce(c)
            | Kind::SkipOnce(c)
            | Kind::Repeat(_, c)
            | Kind::ShowIfPredicate(_, c)
            | Kind::Unconstrained(c)
            | Kind::StopPaging(c)
            | Kind::PreventPageBreak(c) => vec![c.clone()],
            _ => vec![],
        }
    }
    fn init_paragraph(&mut self, width: f32, c: &Context, e: &Environment) -> Result<()> {
        let Kind::Text(t) = &self.kind else {
            return Ok(());
        };
        let style = e.style.merge(&t.style);
        let key = format!(
            "{style:?}{:?}{:?}{}:{}{}{:?}{:?}{}:{}{:?}",
            e.direction,
            t.alignment,
            c.page,
            c.total,
            c.fonts.revision,
            t.spans,
            t.clamp,
            t.paragraph_spacing,
            t.indentation,
            c.sections.borrow()
        );
        if let Some(p) = &mut self.paragraph
            && p.key == key
        {
            if (p.width - width).abs() > EPSILON {
                layout_paragraph(p, width, c)?
            }
            return Ok(());
        }
        let a = &c.fonts.api;
        let ellipsis = native::cstr(t.clamp.as_ref().map(|v| v.1.as_str()).unwrap_or(""));
        let cfg = native::ParagraphConfig {
            alignment: t.alignment as i32,
            direction: if e.direction == ContentDirection::LeftToRight {
                1
            } else {
                0
            },
            max_lines: t.clamp.as_ref().map(|v| v.0 as i32).unwrap_or(1_000_000),
            ellipsis: ellipsis.as_ptr() as Ptr,
        };
        unsafe {
            let builder = Handle::new(
                (a.questpdf_skia_paragraph_builder_create)(
                    &cfg,
                    c.fonts.unicode,
                    c.fonts.collection.ptr,
                ),
                a.questpdf_skia_paragraph_builder_delete,
                a,
            )?;
            let mut whitespace = true;
            let mut block_index = 0;
            let mut text_index = 0usize;
            let mut injected = vec![];
            let mut links = vec![];
            let marker_name = native::cstr("Lato");
            let marker_config = native::TextStyleConfig {
                size: 0.,
                line_height: 1.,
                weight: 400,
                foreground: Color::BLACK.0,
                families: {
                    let mut names = [ptr::null_mut(); 16];
                    names[0] = marker_name.as_ptr() as Ptr;
                    names
                },
                ..Default::default()
            };
            let marker = Handle::new(
                (a.questpdf_skia_text_style_create)(&marker_config),
                a.questpdf_skia_text_style_delete,
                a,
            )?;
            drop(marker_name);
            let marker_text = |value: &str, index: &mut usize| {
                (a.questpdf_skia_paragraph_builder_add_text)(
                    builder.ptr,
                    native::cstr(value).as_ptr(),
                    marker.ptr,
                );
                *index += value.encode_utf16().count();
            };
            let marker_placeholder = |w, h, text_index: &mut usize, block_index: &mut usize| {
                (a.questpdf_skia_paragraph_builder_add_placeholder)(
                    builder.ptr,
                    &native::Placeholder {
                        width: w,
                        height: h,
                        alignment: 5,
                        ..Default::default()
                    },
                );
                *text_index += 1;
                *block_index += 1;
            };
            let indent = |text_index: &mut usize, block_index: &mut usize| {
                if t.indentation > EPSILON {
                    marker_text("\u{200b}", text_index);
                    marker_placeholder(t.indentation, 0., text_index, block_index);
                }
            };
            if !t.spans.is_empty() {
                indent(&mut text_index, &mut block_index);
            }
            for span in &t.spans {
                if let SpanContent::Element(child, alignment) = &span.content {
                    child.reset(true);
                    let size = child.measure(Size::MAX, c, e)?.size;
                    injected.push((block_index, child.clone()));
                    (a.questpdf_skia_paragraph_builder_add_placeholder)(
                        builder.ptr,
                        &native::Placeholder {
                            width: size.width,
                            height: size.height,
                            alignment: *alignment as i32,
                            ..Default::default()
                        },
                    );
                    block_index += 1;
                    text_index += 1;
                    whitespace = false;
                    continue;
                }
                let page_number = match &span.content {
                    SpanContent::CurrentPageNumber => Some(c.page as i32),
                    SpanContent::TotalPages => {
                        if c.total == 0 {
                            None
                        } else {
                            Some(c.total as i32)
                        }
                    }
                    SpanContent::SectionPageNumber(name, kind) => c
                        .sections
                        .borrow()
                        .get(&c.location_name(name))
                        .map(|(start, end)| match kind {
                            SectionPageNumber::Begin => *start as i32,
                            SectionPageNumber::End => *end as i32,
                            SectionPageNumber::Current => c.page as i32 + 1 - *start as i32,
                            SectionPageNumber::Total => (end + 1 - start) as i32,
                        }),
                    _ => None,
                };
                let value = match &span.content {
                    SpanContent::Text(s) => s.clone(),
                    SpanContent::Hyperlink(s, _) | SpanContent::SectionLink(s, _) => s.clone(),
                    SpanContent::CurrentPageNumber
                    | SpanContent::TotalPages
                    | SpanContent::SectionPageNumber(..) => span
                        .formatter
                        .as_ref()
                        .map(|f| (f.0)(page_number))
                        .unwrap_or_else(|| {
                            page_number
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "123".into())
                        }),
                    SpanContent::Element(..) => unreachable!(),
                };
                if !value.trim().is_empty() {
                    whitespace = false
                }
                let st = style.merge(&span.style);
                let families = st
                    .font_families
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| vec!["Lato".into()]);
                if c.throw_on_missing_fonts {
                    let missing: Vec<_> = families
                        .iter()
                        .filter(|name| !c.fonts.registered.contains(&name.to_lowercase()))
                        .collect();
                    if !missing.is_empty() {
                        bail!(
                            "Font families are not registered: {missing:?}. Register their font files on Renderer before generation."
                        )
                    }
                }
                let names: Vec<_> = families.iter().map(|s| native::cstr(s)).collect();
                let mut pointers = [ptr::null_mut(); 16];
                for (i, n) in names.iter().take(16).enumerate() {
                    pointers[i] = n.as_ptr() as Ptr
                }
                // Page.Compose disables standard ligatures by default, matching the .NET engine.
                let features = st
                    .features
                    .clone()
                    .unwrap_or_else(|| vec![("liga".into(), 0)]);
                let feature_names: Vec<_> = features.iter().map(|x| native::cstr(&x.0)).collect();
                let mut feature_cfg = [native::FontFeature::default(); 16];
                for (i, f) in features.iter().take(16).enumerate() {
                    feature_cfg[i] = native::FontFeature {
                        name: feature_names[i].as_ptr(),
                        value: f.1,
                    }
                }
                let config = native::TextStyleConfig {
                    size: st.size.unwrap_or(12.)
                        * if matches!(
                            st.font_position,
                            Some(FontPosition::Subscript | FontPosition::Superscript)
                        ) {
                            0.6
                        } else {
                            1.
                        },
                    weight: st.weight.unwrap_or(400),
                    flags: u32::from(st.italic.unwrap_or(false))
                        | (u32::from(st.break_anywhere.unwrap_or(false)) << 8),
                    families: pointers,
                    features: feature_cfg,
                    foreground: st.color.unwrap_or(Color::BLACK).0,
                    background: st.background.unwrap_or(Color::TRANSPARENT).0,
                    decoration_color: st.decoration_color.unwrap_or(Color::BLACK).0,
                    decoration_thickness: st.decoration_thickness.unwrap_or(1.),
                    decoration: st.decoration.unwrap_or(0),
                    decoration_mode: 1,
                    decoration_style: st.decoration_style.unwrap_or(0),
                    line_height: st.line_height.unwrap_or(0.),
                    letter_spacing: st.letter_spacing.unwrap_or(0.) * st.size.unwrap_or(12.),
                    word_spacing: st.word_spacing.unwrap_or(0.) * st.size.unwrap_or(12.),
                    baseline_offset: st.baseline_offset.unwrap_or(match st.font_position {
                        Some(FontPosition::Subscript) => st.size.unwrap_or(12.) * 0.25,
                        Some(FontPosition::Superscript) => -st.size.unwrap_or(12.) * 0.35,
                        _ => 0.,
                    }),
                };
                let text_style = Handle::new(
                    (a.questpdf_skia_text_style_create)(&config),
                    a.questpdf_skia_text_style_delete,
                    a,
                )?;
                let whole_whitespace = t
                    .spans
                    .iter()
                    .all(|s| matches!(&s.content,SpanContent::Text(v) if v.trim().is_empty()));
                let value = if whole_whitespace { "\u{a0}" } else { &value };
                let begin = text_index;
                for (i, line) in value
                    .replace("\r\n", "\n")
                    .replace('\r', "\n")
                    .split('\n')
                    .enumerate()
                {
                    if i > 0 {
                        if t.paragraph_spacing > EPSILON || t.indentation > EPSILON {
                            if t.paragraph_spacing > EPSILON {
                                marker_text("\n ", &mut text_index);
                                marker_placeholder(
                                    0.,
                                    t.paragraph_spacing,
                                    &mut text_index,
                                    &mut block_index,
                                );
                                marker_text(" \n", &mut text_index);
                            } else {
                                marker_text("\n", &mut text_index);
                            }
                            indent(&mut text_index, &mut block_index);
                        } else {
                            (a.questpdf_skia_paragraph_builder_add_text)(
                                builder.ptr,
                                native::cstr("\n").as_ptr(),
                                text_style.ptr,
                            );
                            text_index += 1;
                        }
                    }
                    (a.questpdf_skia_paragraph_builder_add_text)(
                        builder.ptr,
                        native::cstr(line).as_ptr(),
                        text_style.ptr,
                    );
                    text_index += line.encode_utf16().count();
                }
                if let SpanContent::Hyperlink(text, target)
                | SpanContent::SectionLink(text, target) = &span.content
                {
                    links.push((
                        begin,
                        text_index,
                        text.clone(),
                        target.clone(),
                        matches!(span.content, SpanContent::SectionLink(..)),
                    ));
                }
            }
            if !injected.is_empty() && injected.len() == t.spans.len() {
                let cfg = native::TextStyleConfig {
                    size: style.size.unwrap_or(12.),
                    weight: 400,
                    foreground: style.color.unwrap_or(Color::BLACK).0,
                    ..Default::default()
                };
                let st = Handle::new(
                    (a.questpdf_skia_text_style_create)(&cfg),
                    a.questpdf_skia_text_style_delete,
                    a,
                )?;
                (a.questpdf_skia_paragraph_builder_add_text)(
                    builder.ptr,
                    native::cstr("\u{200b}").as_ptr(),
                    st.ptr,
                );
            }
            let handle = Handle::new(
                (a.questpdf_skia_paragraph_builder_create_paragraph)(builder.ptr),
                a.questpdf_skia_paragraph_delete,
                a,
            )?;
            let mut p = Paragraph {
                handle,
                lines: vec![],
                width: f32::NAN,
                total_width: 0.,
                key,
                whitespace,
                placeholders: vec![],
                injected,
                links,
            };
            layout_paragraph(&mut p, width, c)?;
            self.paragraph = Some(p);
        }
        Ok(())
    }
    fn init_media(&mut self, c: &Context) -> Result<()> {
        if self
            .image
            .as_ref()
            .is_some_and(|m| m.svg_size.is_some() && m.font_revision != c.fonts.revision)
        {
            self.image = None;
        }
        if self.image.is_some() {
            return Ok(());
        }
        let a = &c.fonts.api;
        match &self.kind {
            Kind::Image(bytes) => {
                let d = native::data(a, bytes)?;
                unsafe {
                    let h = Handle::new(
                        (a.questpdf_skia_image_create_from_data)(d.ptr),
                        a.questpdf_skia_image_unref,
                        a,
                    )?;
                    let mut details = native::ImageDetails::default();
                    (a.questpdf_skia_image_get_details)(h.ptr, &mut details);
                    self.image = Some(Media {
                        handle: h,
                        size: Size::new(details.width as f32, details.height as f32),
                        svg_size: None,
                        viewbox: native::Rect::default(),
                        encoded_length: details.encoded_length,
                        font_revision: c.fonts.revision,
                        versions: vec![],
                    })
                }
            }
            Kind::Svg(svg) => {
                let d = native::data(a, svg.as_bytes())?;
                unsafe {
                    native::trace("SVG data created");
                    // Resource provider is a process singleton, matching SkResourceProvider.Local.
                    static RESOURCES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
                    let resources = *RESOURCES.get_or_init(|| {
                        (a.questpdf_skia_resource_provider_create)(
                            native::cstr(
                                &crate::runtime::application_files_directory().to_string_lossy(),
                            )
                            .as_ptr(),
                        ) as usize
                    }) as Ptr;
                    let svg = (a.questpdf_skia_svg_create)(
                        d.ptr,
                        resources,
                        c.fonts.provider.ptr,
                        c.fonts.system,
                    );
                    let h = Handle::new(svg, a.questpdf_skia_svg_unref, a)?;
                    native::trace("SVG decoded");
                    let mut s = native::SvgSize::default();
                    let mut viewbox = native::Rect::default();
                    (a.questpdf_skia_svg_get_size)(h.ptr, &mut s, &mut viewbox);
                    let size = if s.width_unit == 2 || s.height_unit == 2 {
                        Size::new(viewbox.right - viewbox.left, viewbox.bottom - viewbox.top)
                    } else {
                        Size::new(s.width, s.height)
                    };
                    self.image = Some(Media {
                        handle: h,
                        size,
                        svg_size: Some(s),
                        viewbox,
                        encoded_length: 0,
                        font_revision: c.fonts.revision,
                        versions: vec![],
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn measure(&mut self, s: Size, c: &Context, e: &Environment) -> Result<SpacePlan> {
        use SpacePlanType::*;
        if s.negative() {
            return Ok(SpacePlan::wrap());
        }
        if matches!(self.kind, Kind::Text(_)) {
            if self.state.rendered {
                return Ok(SpacePlan::empty());
            }
            if let Kind::Text(t) = &self.kind
                && t.spans.is_empty()
            {
                return Ok(SpacePlan::empty());
            }
            self.init_paragraph(s.width.max(EPSILON), c, e)?;
            let p = self.paragraph.as_ref().unwrap();
            if p.whitespace {
                let h = p.lines.iter().map(|l| l.bottom - l.top).fold(0., f32::max);
                return Ok(if h <= s.height + EPSILON {
                    SpacePlan::full(0., h)
                } else {
                    SpacePlan::wrap()
                });
            }
            if s.width < EPSILON || s.height < EPSILON || p.total_width > s.width + EPSILON {
                return Ok(SpacePlan::wrap());
            }
            let (count, h) = lines_to_draw(p, self.state.line, s.height);
            return Ok(if count == 0 {
                SpacePlan::wrap()
            } else if self.state.line + count < p.lines.len() {
                SpacePlan::partial(p.total_width.min(s.width), h.min(s.height))
            } else {
                SpacePlan::full(p.total_width.min(s.width), h.min(s.height))
            });
        }
        if matches!(self.kind, Kind::Image(_) | Kind::Svg(_)) {
            if self.state.rendered {
                return Ok(SpacePlan::empty());
            }
            self.init_media(c)?;
            if self.stretch {
                return Ok(SpacePlan::full(0., 0.));
            }
            let m = self.image.as_ref().unwrap();
            let ratio = m.size.width / m.size.height;
            if !ratio.is_finite() || ratio <= 0. {
                return Ok(SpacePlan::full(0., 0.));
            }
            let target = aspect_size(s, ratio, self.fit.unwrap_or(AspectRatioOption::FitWidth));
            return Ok(
                if target.width > s.width + EPSILON || target.height > s.height + EPSILON {
                    SpacePlan::wrap()
                } else {
                    SpacePlan::full(target.width, target.height)
                },
            );
        }
        Ok(match &mut self.kind {
            Kind::Empty => SpacePlan::full(0., 0.),
            Kind::SvgPath(_, _) => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else {
                    SpacePlan::full(0., 0.)
                }
            }
            Kind::Proxy(child) => child.measure(s, c, e)?,
            Kind::ZIndex(_, child) => child.measure(s, c, e)?,
            Kind::CapturePosition(_, child) => child.measure(s, c, e)?,
            Kind::Semantic(_, child) | Kind::Artifact(_, child) => child.measure(s, c, e)?,
            Kind::Dynamic(component) => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else if s.width.abs() < EPSILON && s.height.abs() < EPSILON {
                    SpacePlan::partial(0., 0.)
                } else {
                    let context = crate::DynamicContext {
                        page_number: c.page,
                        total_pages: if c.total == 0 {
                            i32::MAX as usize
                        } else {
                            c.total
                        },
                        available_size: s,
                        context: c,
                        environment: e,
                    };
                    let state = component.snapshot();
                    let result = component.compose(&context);
                    component.restore(&*state)?;
                    let result = result?;
                    let measurement = if let Some(content) = result.content {
                        content.element.measure(s, c, e)?
                    } else {
                        SpacePlan::full(0., 0.)
                    };
                    if !measurement.done() {
                        bail!(
                            "Dynamic component generated content that does not fit on a single page"
                        );
                    }
                    if result.has_more_content {
                        SpacePlan::partial(measurement.size.width, measurement.size.height)
                    } else {
                        SpacePlan::full(measurement.size.width, measurement.size.height)
                    }
                }
            }
            Kind::DynamicSvg(_) | Kind::DynamicImage(_) => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else {
                    SpacePlan::full(s.width, s.height)
                }
            }
            Kind::Lazy { source, child, .. } => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else {
                    if child.is_none() {
                        let container = crate::Container::new();
                        source(container.clone());
                        *child = Some(container.0);
                    }
                    child.as_ref().unwrap().measure(s, c, e)?
                }
            }
            Kind::Inlined(v) => {
                if self.state.index == v.items.len() {
                    return Ok(SpacePlan::empty());
                }
                let (commands, size) = inlined_plan(v, self.state.index, s, c, e)?;
                if commands.is_empty() {
                    SpacePlan::wrap()
                } else if self.state.index + commands.len() == v.items.len() {
                    SpacePlan::full(size.width, size.height)
                } else {
                    SpacePlan::partial(size.width, size.height)
                }
            }
            Kind::Column(items, spacing) => {
                let commands = column_plan(items, *spacing, self.state.index, s, c, e)?;
                if commands.is_empty() {
                    if self.state.index == items.len() {
                        SpacePlan::empty()
                    } else {
                        SpacePlan::wrap()
                    }
                } else {
                    let mut w = 0f32;
                    let mut h = 0f32;
                    let mut full = 0;
                    for (_, p, y) in &commands {
                        w = w.max(p.size.width);
                        h = y + p.size.height;
                        if p.done() {
                            full += 1
                        }
                    }
                    if w > s.width + EPSILON || h > s.height + EPSILON {
                        SpacePlan::wrap()
                    } else if self.state.index + full == items.len() {
                        SpacePlan::full(w, h)
                    } else {
                        SpacePlan::partial(w, h)
                    }
                }
            }
            Kind::Row(items, spacing) => {
                if items.is_empty() || items.iter().all(|i| i.rendered) {
                    SpacePlan::empty()
                } else {
                    let commands = row_plan(items, *spacing, s, c, e)?;
                    if commands.iter().any(|(_, p, _)| p.kind == Wrap) {
                        SpacePlan::wrap()
                    } else {
                        let w = commands.last().map(|(sz, _, x)| sz.width + x).unwrap_or(0.);
                        let h = commands
                            .iter()
                            .map(|(sz, _, _)| sz.height)
                            .fold(0., f32::max);
                        if w > s.width + EPSILON || h > s.height + EPSILON {
                            SpacePlan::wrap()
                        } else if commands.iter().any(|(_, p, _)| p.kind == PartialRender) {
                            SpacePlan::partial(w, h)
                        } else {
                            SpacePlan::full(w, h)
                        }
                    }
                }
            }
            Kind::Padding(p, child) => {
                let m = child.measure(
                    Size::new(s.width - p[0] - p[2], s.height - p[1] - p[3]),
                    c,
                    e,
                )?;
                if m.kind == Empty || m.kind == Wrap {
                    m
                } else {
                    SpacePlan {
                        kind: m.kind,
                        size: Size::new(
                            (m.size.width + p[0] + p[2]).max(0.),
                            (m.size.height + p[1] + p[3]).max(0.),
                        ),
                    }
                }
            }
            Kind::Constrained { min, max, child } => {
                if child.measure(Size::ZERO, c, e)?.kind == Empty {
                    SpacePlan::empty()
                } else if min.width > max.width
                    || min.height > max.height
                    || min.width > s.width + EPSILON
                    || min.height > s.height + EPSILON
                {
                    SpacePlan::wrap()
                } else {
                    let m = child.measure(
                        Size::new(s.width.min(max.width), s.height.min(max.height)),
                        c,
                        e,
                    )?;
                    if m.kind == Wrap {
                        m
                    } else {
                        SpacePlan {
                            kind: m.kind,
                            size: Size::new(
                                m.size.width.max(min.width),
                                m.size.height.max(min.height),
                            ),
                        }
                    }
                }
            }
            Kind::DefaultTextStyle(style, child) => child.measure(
                s,
                c,
                &Environment {
                    style: e.style.merge(style),
                    direction: e.direction,
                },
            )?,
            Kind::Section(_, child)
            | Kind::Hyperlink(_, _, child)
            | Kind::SectionLink(_, _, child) => child.measure(s, c, e)?,
            Kind::Direction(dir, child) => child.measure(
                s,
                c,
                &Environment {
                    style: e.style.clone(),
                    direction: *dir,
                },
            )?,
            Kind::AspectRatio(ratio, fit, child) => {
                let target = aspect_size(s, *ratio, *fit);
                if target.width > s.width + EPSILON || target.height > s.height + EPSILON {
                    SpacePlan::wrap()
                } else {
                    let m = child.measure(target, c, e)?;
                    if m.kind == Wrap || m.kind == Empty {
                        m
                    } else {
                        SpacePlan {
                            kind: m.kind,
                            size: target,
                        }
                    }
                }
            }
            Kind::ScaleToFit(child) => {
                if let Some(k) = perfect_scale(child, s, c, e)? {
                    let m = child.measure(Size::new(s.width / k, s.height / k), c, e)?;
                    SpacePlan::full(m.size.width * k, m.size.height * k)
                } else {
                    SpacePlan::wrap()
                }
            }
            Kind::Scale(x, y, child) => {
                let m = child.measure(Size::new(s.width / x.abs(), s.height / y.abs()), c, e)?;
                SpacePlan {
                    kind: m.kind,
                    size: Size::new(m.size.width * x.abs(), m.size.height * y.abs()),
                }
            }
            Kind::Extend(x, y, child) => {
                let mut m = child.measure(s, c, e)?;
                if m.kind != Wrap && m.kind != Empty {
                    if *x {
                        m.size.width = s.width
                    }
                    if *y {
                        m.size.height = s.height
                    }
                }
                m
            }
            Kind::Shrink(_, _, child)
            | Kind::Alignment(_, _, child)
            | Kind::StyledBox(_, child)
            | Kind::Rotate(_, child)
            | Kind::Offset(_, _, child) => child.measure(s, c, e)?,
            Kind::ShowEntire(child) => {
                let m = child.measure(s, c, e)?;
                if m.kind == PartialRender {
                    SpacePlan::wrap()
                } else {
                    m
                }
            }
            Kind::EnsureSpace(h, child) => {
                let m = child.measure(s, c, e)?;
                if self.state.visited
                    || m.done()
                    || (m.kind == PartialRender && m.size.height >= *h)
                {
                    m
                } else {
                    SpacePlan::partial(0., 0.)
                }
            }
            Kind::PreventPageBreak(child) => {
                let m = child.measure(s, c, e)?;
                if self.state.visited || m.done() {
                    m
                } else {
                    SpacePlan::partial(0., 0.)
                }
            }
            Kind::ShowOnce(child) => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else {
                    child.measure(s, c, e)?
                }
            }
            Kind::SkipOnce(child) => {
                if !self.state.visited {
                    SpacePlan::empty()
                } else {
                    child.measure(s, c, e)?
                }
            }
            Kind::Repeat(_, child) => child.measure(s, c, e)?,
            Kind::ShowIfPredicate(predicate, child) => {
                if predicate(ShowIfContext {
                    page_number: c.page,
                    total_pages: (c.total > 0).then_some(c.total),
                }) {
                    child.measure(s, c, e)?
                } else {
                    SpacePlan::empty()
                }
            }
            Kind::Unconstrained(child) => {
                let mut plan = child.measure(Size::MAX, c, e)?;
                if matches!(plan.kind, FullRender | PartialRender) {
                    plan.size = Size::ZERO;
                }
                plan
            }
            Kind::StopPaging(child) => {
                let mut m = child.measure(s, c, e)?;
                if m.kind == PartialRender {
                    m.kind = FullRender
                }
                if m.kind == Wrap {
                    m = SpacePlan::full(0., 0.);
                }
                m
            }
            Kind::PageBreak => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else {
                    SpacePlan::partial(0., 0.)
                }
            }
            Kind::Line {
                width, vertical, ..
            } => {
                if self.state.rendered {
                    SpacePlan::empty()
                } else if *width
                    > if *vertical {
                        s.width + EPSILON
                    } else {
                        s.height + EPSILON
                    }
                {
                    SpacePlan::wrap()
                } else if *vertical {
                    SpacePlan::full(*width, 0.)
                } else {
                    SpacePlan::full(0., *width)
                }
            }
            Kind::Table(t) => table_plan(t, self.state.index, s, c, e)?.measurement,
            Kind::MultiColumn {
                content,
                count,
                spacing,
                balance,
                ..
            } => multi_column_measure(content, *count, *spacing, *balance, s, c, e)?,
            Kind::Layers(layers) => {
                let primary: Vec<_> = layers.iter().filter(|i| i.0).collect();
                if primary.len() != 1 {
                    bail!("Layers require exactly one primary layer")
                };
                primary[0].1.measure(s, c, e)?
            }
            Kind::Decoration {
                before,
                content,
                after,
            } => {
                let (a, b, d) = decoration_plan(before, content, after, s, c, e)?;
                if b.kind == Empty {
                    b
                } else if !a.done() || !d.done() || b.kind == Wrap {
                    SpacePlan::wrap()
                } else {
                    SpacePlan {
                        kind: b.kind,
                        size: Size::new(
                            a.size.width.max(b.size.width).max(d.size.width),
                            a.size.height + b.size.height + d.size.height,
                        ),
                    }
                }
            }
            Kind::RotateLayout(turns, child) => {
                if turns.rem_euclid(2) == 0 {
                    child.measure(s, c, e)?
                } else {
                    let m = child.measure(Size::new(s.height, s.width), c, e)?;
                    SpacePlan {
                        kind: m.kind,
                        size: Size::new(m.size.height, m.size.width),
                    }
                }
            }
            Kind::Image(_) | Kind::Svg(_) | Kind::Text(_) => unreachable!(),
        })
    }
    fn draw(&mut self, s: Size, canvas: Ptr, c: &Context, e: &Environment) -> Result<()> {
        let a = &c.fonts.api;
        if matches!(self.kind, Kind::Text(_)) {
            if self.state.rendered {
                return Ok(());
            }
            self.init_paragraph(s.width.max(EPSILON), c, e)?;
            let p = self.paragraph.as_ref().unwrap();
            let (count, taken_height) = lines_to_draw(p, self.state.line, s.height);
            if count == 0 {
                return Ok(());
            }
            if !p.whitespace && !canvas.is_null() {
                unsafe {
                    let multiple = count != p.lines.len();
                    if multiple {
                        a.questpdf_skia_canvas_save(canvas);
                        a.questpdf_skia_canvas_translate(canvas, 0., -p.lines[self.state.line].top)
                    }
                    a.questpdf_skia_canvas_draw_paragraph(
                        canvas,
                        p.handle.ptr,
                        self.state.line as i32,
                        (self.state.line + count - 1) as i32,
                    );
                    let page_top = p.lines[self.state.line].top;
                    if multiple {
                        a.questpdf_skia_canvas_clip_rectangle(
                            canvas,
                            &native::Rect {
                                left: 0.,
                                top: page_top,
                                right: s.width,
                                bottom: page_top + taken_height,
                            },
                        );
                    }
                    for (index, child) in &p.injected {
                        if let Some(r) = p.placeholders.get(*index)
                            && page_top <= r.bottom
                            && r.top <= page_top + taken_height
                        {
                            child.reset(true);
                            draw_at(
                                child,
                                Size::new(r.right - r.left, r.bottom - r.top),
                                r.left,
                                r.top,
                                canvas,
                                c,
                                e,
                            )?;
                        }
                    }
                    for (begin, end, text, target, section) in &p.links {
                        let mut positions = ptr::null_mut();
                        let mut length = 0;
                        (a.questpdf_skia_paragraph_get_text_range_positions)(
                            p.handle.ptr,
                            *begin as i32,
                            *end as i32,
                            &mut positions,
                            &mut length,
                        );
                        let rects = if length > 0 {
                            std::slice::from_raw_parts(positions, length as usize).to_vec()
                        } else {
                            vec![]
                        };
                        (a.questpdf_skia_paragraph_delete_positions)(positions);
                        for r in rects {
                            if page_top > r.bottom || r.top > page_top + taken_height {
                                continue;
                            }
                            a.questpdf_skia_canvas_translate(canvas, r.left, r.top);
                            let annotate = if *section {
                                a.questpdf_skia_canvas_annotate_destination_link
                            } else {
                                a.questpdf_skia_canvas_annotate_url
                            };
                            annotate(
                                crate::canvas::route(canvas),
                                r.right - r.left,
                                r.bottom - r.top,
                                native::cstr(
                                    if *section {
                                        c.location_name(target)
                                    } else {
                                        target.clone()
                                    }
                                    .as_str(),
                                )
                                .as_ptr(),
                                native::cstr(text).as_ptr(),
                            );
                            a.questpdf_skia_canvas_translate(canvas, -r.left, -r.top);
                        }
                    }
                    if multiple {
                        a.questpdf_skia_canvas_restore(canvas)
                    }
                }
            }
            self.state.line += count;
            self.state.rendered = self.state.line >= p.lines.len();
            return Ok(());
        }
        if matches!(self.kind, Kind::Image(_) | Kind::Svg(_)) {
            if self.state.rendered {
                return Ok(());
            }
            self.init_media(c)?;
            let m = self.image.as_ref().unwrap();
            let target = if self.stretch {
                s
            } else {
                aspect_size(
                    s,
                    m.size.width / m.size.height,
                    self.fit.unwrap_or(AspectRatioOption::FitWidth),
                )
            };
            let mut image_ptr = m.handle.ptr;
            if m.svg_size.is_none() && !c.use_original_images && !self.use_original_image {
                let quality = self.image_quality.unwrap_or(c.image_quality);
                let scale = self.image_dpi.unwrap_or(c.image_dpi) / 72.;
                let (mut width, mut height) = (
                    (target.width * scale) as i32,
                    (target.height * scale) as i32,
                );
                if self.image_no_resize
                    || m.size.width < width as f32
                    || m.size.height < height as f32
                {
                    width = m.size.width as i32;
                    height = m.size.height as i32;
                }
                if width == 0 || height == 0 {
                    width = 1;
                    height = 1;
                }
                let m = self.image.as_mut().unwrap();
                if !m
                    .versions
                    .iter()
                    .any(|(w, h, q, _)| *w == width && *h == height && *q == quality)
                {
                    unsafe {
                        let handle = Handle::new(
                            (a.questpdf_skia_image_resize_and_compress)(
                                m.handle.ptr,
                                width,
                                height,
                                quality.quality(),
                                !matches!(
                                    quality,
                                    ImageCompressionQuality::Best
                                        | ImageCompressionQuality::VeryHigh
                                ),
                            ),
                            a.questpdf_skia_image_unref,
                            a,
                        )?;
                        m.versions.push((width, height, quality, handle));
                    }
                }
                let compressed = &m
                    .versions
                    .iter()
                    .find(|(w, h, q, _)| *w == width && *h == height && *q == quality)
                    .unwrap()
                    .3;
                let mut details = native::ImageDetails::default();
                unsafe {
                    (a.questpdf_skia_image_get_details)(compressed.ptr, &mut details);
                }
                if details.encoded_length <= m.encoded_length {
                    image_ptr = compressed.ptr;
                }
            }
            let m = self.image.as_ref().unwrap();
            if !canvas.is_null() {
                unsafe {
                    a.questpdf_skia_canvas_save(canvas);
                    if let Some(sz) = &m.svg_size {
                        fn scale(av: f32, dim: f32, unit: i32, vb: bool) -> f32 {
                            if unit == 2 {
                                return 1.;
                            }
                            let dpi = if vb { 90. } else { 96. };
                            let natural = match unit {
                                4 => dim / 2.54 * dpi,
                                5 => dim / 25.4 * dpi,
                                6 => dim * dpi,
                                7 => dim * dpi / 72.,
                                8 => dim * 12. * dpi / 72.,
                                _ => dim,
                            };
                            av / natural
                        }
                        let vb = m.viewbox.right != 0. || m.viewbox.bottom != 0.;
                        a.questpdf_skia_canvas_scale(
                            canvas,
                            scale(target.width, sz.width, sz.width_unit, vb),
                            scale(target.height, sz.height, sz.height_unit, vb),
                        );
                        a.questpdf_skia_canvas_draw_svg(
                            canvas,
                            image_ptr,
                            target.width,
                            target.height,
                        );
                    } else {
                        a.questpdf_skia_canvas_draw_image(
                            canvas,
                            image_ptr,
                            target.width,
                            target.height,
                        )
                    }
                    a.questpdf_skia_canvas_restore(canvas);
                }
            }
            self.state.rendered = true;
            return Ok(());
        }
        let section_link = matches!(self.kind, Kind::SectionLink(..));
        match &mut self.kind {
            Kind::Empty => {}
            Kind::SvgPath(path, color) => {
                if !self.state.rendered && !canvas.is_null() {
                    unsafe {
                        a.questpdf_skia_canvas_draw_svg_path(
                            canvas,
                            native::cstr(path).as_ptr(),
                            color.0,
                        );
                    }
                }
                self.state.rendered = true;
            }
            Kind::Proxy(child) => child.draw(s, canvas, c, e)?,
            Kind::Semantic(_, child) => child.draw(s, canvas, c, e)?,
            Kind::Artifact(id, child) => {
                let _scope = crate::semantic::Scope::artifact(c, canvas, *id);
                child.draw(s, canvas, c, e)?;
            }
            Kind::ZIndex(index, child) => {
                let previous = crate::canvas::z_index(canvas);
                crate::canvas::set_z_index(canvas, *index)?;
                let result = child.draw(s, canvas, c, e);
                crate::canvas::set_z_index(canvas, previous)?;
                result?;
            }
            Kind::CapturePosition(id, child) => {
                child.draw(s, canvas, c, e)?;
                if !canvas.is_null() && (c.total == 0 || !child.is_cacheable()) {
                    let mut m = native::Matrix::default();
                    unsafe {
                        a.questpdf_skia_canvas_get_matrix9(canvas, &mut m);
                    }
                    c.captured_positions.borrow_mut().push(PageElementLocation {
                        id: id.clone(),
                        page_number: c.page,
                        width: s.width * (m.scale_x * m.scale_x + m.skew_y * m.skew_y).sqrt(),
                        height: s.height * (m.skew_x * m.skew_x + m.scale_y * m.scale_y).sqrt(),
                        x: m.translate_x,
                        y: m.translate_y,
                        transform: [
                            m.scale_x,
                            m.skew_y,
                            0.,
                            0.,
                            m.skew_x,
                            m.scale_y,
                            0.,
                            0.,
                            0.,
                            0.,
                            m.perspective3,
                            0.,
                            m.translate_x,
                            m.translate_y,
                            0.,
                            1.,
                        ],
                    });
                }
            }
            Kind::Dynamic(component) => {
                let _scope = crate::semantic::CounterScope::new(
                    c,
                    &mut self.semantic_snapshots,
                    self.state.index,
                );
                let context = crate::DynamicContext {
                    page_number: c.page,
                    total_pages: if c.total == 0 {
                        i32::MAX as usize
                    } else {
                        c.total
                    },
                    available_size: s,
                    context: c,
                    environment: e,
                };
                let result = component.compose(&context)?;
                if let Some(content) = result.content {
                    content.element.draw(s, canvas, c, e)?;
                }
                self.state.rendered = !result.has_more_content;
                self.state.index += 1;
            }
            Kind::Lazy {
                source,
                child,
                retain_content,
            } => {
                let _scope = crate::semantic::CounterScope::new(
                    c,
                    &mut self.semantic_snapshots,
                    self.state.index,
                );
                if !self.state.rendered {
                    if child.is_none() {
                        let container = crate::Container::new();
                        source(container.clone());
                        *child = Some(container.0);
                    }
                    let el = child.as_ref().unwrap();
                    let full = el.measure(s, c, e)?.kind == SpacePlanType::FullRender;
                    el.draw(s, canvas, c, e)?;
                    self.state.index += 1;
                    if full && !*retain_content {
                        self.state.rendered = true;
                        *child = None;
                    }
                }
            }
            Kind::DynamicImage(source) => {
                if !self.state.rendered && !canvas.is_null() {
                    let cached = self
                        .dynamic_media
                        .iter()
                        .find(|(size, _)| {
                            (size.width - s.width).abs() < EPSILON
                                && (size.height - s.height).abs() < EPSILON
                        })
                        .map(|(_, e)| e.clone());
                    if let Some(el) = cached {
                        el.reset(true);
                        el.draw(s, canvas, c, e)?;
                        self.state.rendered = true;
                        return Ok(());
                    }
                    let dpi = self.image_dpi.unwrap_or(c.image_dpi);
                    let scale = dpi / 72.;
                    let payload = GenerateDynamicImagePayload {
                        available_space: s,
                        image_size: ImageSize {
                            width: (s.width * scale) as u32,
                            height: (s.height * scale) as u32,
                        },
                        dpi,
                    };
                    if let Some(bytes) = source(payload) {
                        let el = Element::empty();
                        el.set(Kind::Image(bytes));
                        {
                            let mut node = el.0.borrow_mut();
                            node.stretch = true;
                            node.image_no_resize = true;
                            node.use_original_image = self.use_original_image;
                            node.image_quality = self.image_quality;
                        }
                        el.draw(s, canvas, c, e)?;
                        self.dynamic_media.push((s, el));
                    }
                }
                self.state.rendered = true;
            }
            Kind::DynamicSvg(source) => {
                if !self.state.rendered && !canvas.is_null() {
                    let cached = self
                        .dynamic_media
                        .iter()
                        .find(|(size, _)| {
                            (size.width - s.width).abs() < EPSILON
                                && (size.height - s.height).abs() < EPSILON
                        })
                        .map(|(_, e)| e.clone());
                    if let Some(el) = cached {
                        el.reset(true);
                        el.draw(s, canvas, c, e)?;
                        self.state.rendered = true;
                        return Ok(());
                    }
                    let el = Element::empty();
                    el.set(Kind::Svg(source(s)));
                    el.0.borrow_mut().stretch = true;
                    el.draw(s, canvas, c, e)?;
                    self.dynamic_media.push((s, el));
                }
                self.state.rendered = true
            }
            Kind::Inlined(v) => {
                let (commands, _) = inlined_plan(v, self.state.index, s, c, e)?;
                for (index, size, x, y) in &commands {
                    draw_at(&v.items[*index], *size, *x, *y, canvas, c, e)?;
                }
                self.state.index += commands.len();
            }
            Kind::Column(items, spacing) => {
                let commands = column_plan(items, *spacing, self.state.index, s, c, e)?;
                for (index, m, y) in commands {
                    draw_at(
                        &items[index],
                        Size::new(s.width, m.size.height),
                        0.,
                        y,
                        canvas,
                        c,
                        e,
                    )?;
                    if m.done() {
                        self.state.index += 1
                    }
                }
            }
            Kind::Row(items, spacing) => {
                if items.iter().all(|i| i.rendered) {
                    return Ok(());
                }
                let plan = row_plan(items, *spacing, s, c, e)?;
                for (i, (sz, m, x)) in items.iter_mut().zip(plan) {
                    if m.kind != SpacePlanType::Wrap {
                        let pos = if e.direction == ContentDirection::RightToLeft {
                            s.width - x - sz.width
                        } else {
                            x
                        };
                        draw_at(
                            &i.element,
                            Size::new(sz.width, s.height),
                            pos,
                            0.,
                            canvas,
                            c,
                            e,
                        )?;
                        if m.done() {
                            i.rendered = true
                        }
                    }
                }
            }
            Kind::Padding(p, child) => draw_at(
                child,
                Size::new(s.width - p[0] - p[2], s.height - p[1] - p[3]),
                p[0],
                p[1],
                canvas,
                c,
                e,
            )?,
            Kind::Constrained { max, child, .. } => {
                let target = Size::new(s.width.min(max.width), s.height.min(max.height));
                let x = if e.direction == ContentDirection::RightToLeft {
                    s.width - target.width
                } else {
                    0.
                };
                draw_at(child, target, x, 0., canvas, c, e)?
            }
            Kind::Alignment(h, v, child) => {
                let m = child.measure(s, c, e)?;
                if m.kind != SpacePlanType::Empty && m.kind != SpacePlanType::Wrap {
                    let target = Size::new(
                        if h.is_some() { m.size.width } else { s.width },
                        if v.is_some() { m.size.height } else { s.height },
                    );
                    let x = match h {
                        Some(HorizontalAlignment::Center) => (s.width - target.width) / 2.,
                        Some(HorizontalAlignment::Right) => s.width - target.width,
                        _ => 0.,
                    };
                    let y = match v {
                        Some(VerticalAlignment::Middle) => (s.height - target.height) / 2.,
                        Some(VerticalAlignment::Bottom) => s.height - target.height,
                        _ => 0.,
                    };
                    draw_at(child, target, x, y, canvas, c, e)?
                }
            }
            Kind::DefaultTextStyle(style, child) => child.draw(
                s,
                canvas,
                c,
                &Environment {
                    style: e.style.merge(style),
                    direction: e.direction,
                },
            )?,
            Kind::Section(name, child) => {
                if c.total == 0 {
                    c.sections
                        .borrow_mut()
                        .entry(c.location_name(name))
                        .and_modify(|v| {
                            v.0 = v.0.min(c.page);
                            v.1 = v.1.max(c.page);
                        })
                        .or_insert((c.page, c.page));
                }
                if !self.state.rendered && !canvas.is_null() {
                    unsafe {
                        a.questpdf_skia_canvas_annotate_destination(
                            canvas,
                            native::cstr(&c.location_name(name)).as_ptr(),
                        );
                    }
                }
                self.state.rendered = true;
                child.draw(s, canvas, c, e)?;
            }
            Kind::Hyperlink(target, description, child)
            | Kind::SectionLink(target, description, child) => {
                let plan = child.measure(s, c, e)?;
                if matches!(plan.kind, SpacePlanType::Empty | SpacePlanType::Wrap) {
                    return Ok(());
                }
                let section = section_link;
                if !canvas.is_null() {
                    let x = if !section && e.direction == ContentDirection::RightToLeft {
                        s.width - plan.size.width
                    } else {
                        0.
                    };
                    let area = if section { plan.size } else { s };
                    unsafe {
                        a.questpdf_skia_canvas_translate(canvas, x, 0.);
                        let annotate = if section {
                            a.questpdf_skia_canvas_annotate_destination_link
                        } else {
                            a.questpdf_skia_canvas_annotate_url
                        };
                        let target = native::cstr(
                            if section {
                                c.location_name(target)
                            } else {
                                target.clone()
                            }
                            .as_str(),
                        );
                        let description = description.as_ref().map(|v| native::cstr(v));
                        annotate(
                            crate::canvas::route(canvas),
                            area.width,
                            area.height,
                            target.as_ptr(),
                            description
                                .as_ref()
                                .map(|v| v.as_ptr())
                                .unwrap_or(ptr::null()),
                        );
                        a.questpdf_skia_canvas_translate(canvas, -x, 0.);
                    }
                }
                child.draw(s, canvas, c, e)?;
            }
            Kind::Direction(dir, child) => child.draw(
                s,
                canvas,
                c,
                &Environment {
                    style: e.style.clone(),
                    direction: *dir,
                },
            )?,
            Kind::StyledBox(style, child) => draw_styled_box(style, child, s, canvas, c, e)?,
            Kind::AspectRatio(ratio, fit, child) => {
                let target = aspect_size(s, *ratio, *fit);
                let x = if e.direction == ContentDirection::RightToLeft {
                    s.width - target.width
                } else {
                    0.
                };
                draw_at(child, target, x, 0., canvas, c, e)?
            }
            Kind::ScaleToFit(child) => {
                if let Some(k) = perfect_scale(child, s, c, e)? {
                    draw_scaled(
                        child,
                        Size::new(s.width / k, s.height / k),
                        k,
                        k,
                        canvas,
                        c,
                        e,
                    )?
                }
            }
            Kind::Scale(x, y, child) => draw_scaled(
                child,
                Size::new(s.width / x.abs(), s.height / y.abs()),
                *x,
                *y,
                canvas,
                c,
                e,
            )?,
            Kind::Rotate(deg, child) => {
                let measured = child.measure(s, c, e)?.size;
                let pivot = (measured.width / 2., measured.height / 2.);
                if !canvas.is_null() {
                    unsafe {
                        a.questpdf_skia_canvas_save(canvas);
                        a.questpdf_skia_canvas_translate(canvas, pivot.0, pivot.1);
                        a.questpdf_skia_canvas_rotate(canvas, *deg);
                        a.questpdf_skia_canvas_translate(canvas, -pivot.0, -pivot.1);
                    }
                }
                child.draw(s, canvas, c, e)?;
                if !canvas.is_null() {
                    unsafe { a.questpdf_skia_canvas_restore(canvas) }
                }
            }
            Kind::Offset(x, y, child) => draw_at(child, s, *x, *y, canvas, c, e)?,
            Kind::Shrink(x, y, child) => {
                let m = child.measure(s, c, e)?;
                if matches!(m.kind, SpacePlanType::Empty | SpacePlanType::Wrap) {
                    return Ok(());
                }
                let target = Size::new(
                    if *x { m.size.width } else { s.width },
                    if *y { m.size.height } else { s.height },
                );
                let offset = if e.direction == ContentDirection::RightToLeft {
                    s.width - target.width
                } else {
                    0.
                };
                draw_at(child, target, offset, 0., canvas, c, e)?;
            }
            Kind::Extend(_, _, child) | Kind::ShowEntire(child) => child.draw(s, canvas, c, e)?,
            Kind::StopPaging(child) => {
                if child.measure(s, c, e)?.kind != SpacePlanType::Wrap {
                    child.draw(s, canvas, c, e)?;
                }
            }
            Kind::EnsureSpace(h, child) => {
                let m = child.measure(s, c, e)?;
                if self.state.visited
                    || m.done()
                    || (m.kind == SpacePlanType::PartialRender && m.size.height >= *h)
                {
                    child.draw(s, canvas, c, e)?
                }
                self.state.visited = true
            }
            Kind::PreventPageBreak(child) => {
                if self.state.visited || child.measure(s, c, e)?.done() {
                    child.draw(s, canvas, c, e)?
                }
                self.state.visited = true
            }
            Kind::ShowOnce(child) => {
                if !self.state.rendered {
                    let m = child.measure(s, c, e)?;
                    child.draw(s, canvas, c, e)?;
                    self.state.rendered = m.done()
                }
            }
            Kind::SkipOnce(child) => {
                if self.state.visited {
                    child.draw(s, canvas, c, e)?
                }
                self.state.visited = true
            }
            Kind::Repeat(id, child) => {
                if !self.repeat_optimized {
                    child.retain_lazy_content();
                    self.repeat_optimized = true;
                }
                let m = child.measure(s, c, e)?;
                let _scope = if self.state.visited && c.semantic_tree.borrow().is_some() {
                    Some(crate::semantic::Scope::artifact(c, canvas, *id))
                } else {
                    None
                };
                child.draw(s, canvas, c, e)?;
                if m.kind == SpacePlanType::FullRender {
                    child.reset(true);
                    self.state.visited = true;
                }
            }
            Kind::Unconstrained(child) => {
                let m = child.measure(Size::MAX, c, e)?;
                if matches!(m.kind, SpacePlanType::Empty | SpacePlanType::Wrap) {
                    return Ok(());
                }
                let offset = if e.direction == ContentDirection::RightToLeft {
                    -m.size.width
                } else {
                    0.
                };
                draw_at(child, m.size, offset, 0., canvas, c, e)?;
            }
            Kind::ShowIfPredicate(predicate, child) => {
                if predicate(ShowIfContext {
                    page_number: c.page,
                    total_pages: (c.total > 0).then_some(c.total),
                }) {
                    child.draw(s, canvas, c, e)?;
                }
            }
            Kind::PageBreak => self.state.rendered = true,
            Kind::Line {
                width,
                color,
                vertical,
                dash,
                gradient,
            } => {
                if self.state.rendered {
                    return Ok(());
                }
                let _scope = crate::semantic::Scope::artifact(c, canvas, -6);
                if !canvas.is_null() {
                    let p = paint(*color, Some(*width), c)?;
                    let start = native::Point { x: 0., y: 0. };
                    let end = if *vertical {
                        native::Point { x: 0., y: s.height }
                    } else {
                        native::Point { x: s.width, y: 0. }
                    };
                    let (x, y) = if *vertical {
                        (*width / 2., 0.)
                    } else {
                        (0., *width / 2.)
                    };
                    unsafe {
                        if !gradient.is_empty() {
                            let colors: Vec<_> = gradient.iter().map(|c| c.0).collect();
                            (a.questpdf_skia_paint_set_linear_gradient)(
                                p.ptr,
                                &start,
                                &end,
                                colors.len() as i32,
                                colors.as_ptr(),
                            );
                        }
                        if !dash.is_empty() {
                            (a.questpdf_skia_paint_set_dashed_path_effect)(
                                p.ptr,
                                dash.len() as i32,
                                dash.as_ptr(),
                            );
                        }
                        a.questpdf_skia_canvas_translate(canvas, x, y);
                        a.questpdf_skia_canvas_draw_line(canvas, &start, &end, p.ptr);
                        a.questpdf_skia_canvas_translate(canvas, -x, -y);
                    }
                }
                self.state.rendered = true;
            }
            Kind::Table(t) => {
                let header_repeated = t.header_repeat.get();
                let footer_repeated = t.footer_repeat.get();
                if !header_repeated {
                    for cell in &t.header {
                        cell.element.retain_lazy_content();
                    }
                }
                if !footer_repeated {
                    for cell in &t.footer {
                        cell.element.retain_lazy_content();
                    }
                }
                let width = table_plan(t, self.state.index, s, c, e)?
                    .measurement
                    .size
                    .width;
                let mut plan = table_plan(t, self.state.index, Size::new(width, s.height), c, e)?;
                plan.commands.sort_by_key(|cmd| (cmd.section, cmd.index));
                for section in 0..3 {
                    let section_y = plan.section_offsets[section];
                    if !canvas.is_null() {
                        unsafe {
                            a.questpdf_skia_canvas_translate(canvas, 0., section_y);
                        }
                    }
                    for item in plan
                        .commands
                        .iter()
                        .filter(|item| item.section as usize == section)
                    {
                        let _repeat_scope = if item.section == 0
                            && header_repeated
                            && c.semantic_tree.borrow().is_some()
                        {
                            Some(crate::semantic::Scope::artifact(c, canvas, -3))
                        } else if item.section == 2
                            && footer_repeated
                            && c.semantic_tree.borrow().is_some()
                        {
                            Some(crate::semantic::Scope::artifact(c, canvas, -4))
                        } else {
                            None
                        };
                        if !canvas.is_null() {
                            register_table_section(t, item.section as usize, c)?;
                        }
                        let cell = match item.section {
                            0 => &t.header[item.index],
                            1 => &t.cells[item.index],
                            _ => &t.footer[item.index],
                        };
                        if item.measurement.done() {
                            cell.rendered.set(true)
                        }
                        let x = if e.direction == ContentDirection::RightToLeft {
                            width - item.x - item.size.width
                        } else {
                            item.x
                        };
                        let _scope = cell
                            .semantic_id
                            .get()
                            .filter(|_| {
                                !canvas.is_null()
                                    && c.semantic_tree
                                        .borrow()
                                        .as_ref()
                                        .is_some_and(|t| t.artifacts == 0)
                            })
                            .map(|id| crate::semantic::Scope::node(c, canvas, id));
                        draw_at(&cell.element, item.size, x, item.y, canvas, c, e)?
                    }
                    if !canvas.is_null() {
                        unsafe {
                            a.questpdf_skia_canvas_translate(canvas, 0., -section_y);
                        }
                    }
                }
                self.state.index = plan.next_row;
                if !t.header.is_empty() {
                    t.header_repeat.set(true);
                }
                if !t.footer.is_empty() {
                    t.footer_repeat.set(true);
                }
                for cell in t.header.iter().chain(&t.footer) {
                    cell.element.reset(true);
                    cell.rendered.set(false)
                }
            }
            Kind::MultiColumn {
                content,
                spacer,
                count,
                spacing,
                ..
            } => {
                let width = (s.width - *spacing * count.saturating_sub(1) as f32) / *count as f32;
                let area = Size::new(width, s.height);
                let mut offset = 0.;
                for i in 0..*count {
                    let m = content.measure(area, c, e)?;
                    let x = if e.direction == ContentDirection::RightToLeft {
                        s.width - offset - width
                    } else {
                        offset
                    };
                    draw_at(
                        content,
                        Size::new(width, m.size.height),
                        x,
                        0.,
                        canvas,
                        c,
                        e,
                    )?;
                    offset += width;
                    if m.done() {
                        break;
                    }
                    let space = Size::new(*spacing, s.height);
                    if i + 1 < *count && spacer.measure(space, c, e)?.kind != SpacePlanType::Wrap {
                        let x = if e.direction == ContentDirection::RightToLeft {
                            s.width - offset - *spacing
                        } else {
                            offset
                        };
                        draw_at(spacer, space, x, 0., canvas, c, e)?;
                        offset += *spacing
                    }
                }
            }
            Kind::Layers(layers) => {
                for (_, el) in layers {
                    if el.measure(s, c, e)?.kind != SpacePlanType::Wrap {
                        el.draw(s, canvas, c, e)?
                    }
                }
            }
            Kind::Decoration {
                before,
                content,
                after,
            } => {
                let (h, m, f) = decoration_plan(before, content, after, s, c, e)?;
                let width = h.size.width.max(m.size.width).max(f.size.width);
                let x = if e.direction == ContentDirection::RightToLeft {
                    s.width - width
                } else {
                    0.
                };
                draw_at(before, Size::new(width, h.size.height), x, 0., canvas, c, e)?;
                draw_at(
                    content,
                    Size::new(width, m.size.height),
                    x,
                    h.size.height,
                    canvas,
                    c,
                    e,
                )?;
                draw_at(
                    after,
                    Size::new(width, f.size.height),
                    x,
                    h.size.height + m.size.height,
                    canvas,
                    c,
                    e,
                )?
            }
            Kind::RotateLayout(turns, child) => {
                let turns = turns.rem_euclid(4);
                if !canvas.is_null() {
                    unsafe {
                        a.questpdf_skia_canvas_save(canvas);
                        a.questpdf_skia_canvas_translate(
                            canvas,
                            if turns == 1 || turns == 2 {
                                s.width
                            } else {
                                0.
                            },
                            if turns == 2 || turns == 3 {
                                s.height
                            } else {
                                0.
                            },
                        );
                        a.questpdf_skia_canvas_rotate(canvas, turns as f32 * 90.)
                    }
                }
                let target = if turns % 2 == 1 {
                    Size::new(s.height, s.width)
                } else {
                    s
                };
                child.draw(target, canvas, c, e)?;
                if !canvas.is_null() {
                    unsafe { a.questpdf_skia_canvas_restore(canvas) }
                }
            }
            Kind::Text(_) | Kind::Image(_) | Kind::Svg(_) => unreachable!(),
        }
        Ok(())
    }
}
struct ElementSnapshot {
    element: Element,
    state: State,
    row: Vec<bool>,
    table: Vec<bool>,
    dynamic_state: Option<Box<dyn std::any::Any>>,
    table_repeat: Option<[bool; 2]>,
}
fn restore(states: Vec<ElementSnapshot>) {
    for saved in states {
        let mut n = saved.element.0.borrow_mut();
        n.state = saved.state;
        if let (Kind::Table(t), Some([header, footer])) = (&n.kind, saved.table_repeat) {
            t.header_repeat.set(header);
            t.footer_repeat.set(footer);
        }
        if let (Kind::Dynamic(component), Some(state)) = (&mut n.kind, saved.dynamic_state) {
            component
                .restore(&*state)
                .expect("matching dynamic component state");
        }
        if let Kind::Row(items, _) = &mut n.kind {
            for (i, v) in items.iter_mut().zip(saved.row) {
                i.rendered = v
            }
        }
        if let Kind::Table(t) = &n.kind {
            for (i, v) in t
                .cells
                .iter()
                .chain(&t.header)
                .chain(&t.footer)
                .zip(saved.table)
            {
                i.rendered.set(v)
            }
        }
    }
}
type InlinedCommand = (usize, Size, f32, f32);
fn inlined_plan(
    v: &Inlined,
    start: usize,
    s: Size,
    c: &Context,
    e: &Environment,
) -> Result<(Vec<InlinedCommand>, Size)> {
    let rtl = e.direction == ContentDirection::RightToLeft;
    let align = v.alignment.unwrap_or(if rtl {
        InlinedAlignment::Right
    } else {
        InlinedAlignment::Left
    });
    let mut commands = vec![];
    let (mut index, mut top, mut width, mut height, mut lines) = (start, 0f32, 0f32, 0f64, 0usize);
    loop {
        let begin = commands.len();
        let (mut line_width, mut line_height) = (0f64, 0f32);
        let mut left = if matches!(align, InlinedAlignment::SpaceAround) {
            v.horizontal_spacing * 2.
        } else {
            0.
        };
        while index < v.items.len() {
            let plan = v.items[index].measure(Size::new(s.width, INFINITY), c, e)?;
            if !plan.done() || left + plan.size.width > s.width + EPSILON {
                break;
            }
            commands.push((index, plan.size, 0., 0.));
            index += 1;
            left += plan.size.width + v.horizontal_spacing;
            line_width += plan.size.width as f64;
            line_height = line_height.max(plan.size.height);
        }
        let count = commands.len() - begin;
        if count == 0 {
            break;
        }
        if top + line_height > s.height + EPSILON {
            commands.truncate(begin);
            break;
        }
        let line_width = line_width as f32;
        let difference = s.width - line_width;
        let offset = if count == 1 {
            0.
        } else {
            match align {
                InlinedAlignment::Justify => difference / (count - 1) as f32,
                InlinedAlignment::SpaceAround => difference / (count + 1) as f32,
                _ => v.horizontal_spacing,
            }
        };
        let empty = difference - (count - 1) as f32 * v.horizontal_spacing;
        let mut left = match align {
            InlinedAlignment::Left => {
                if rtl {
                    empty
                } else {
                    0.
                }
            }
            InlinedAlignment::Right => {
                if rtl {
                    0.
                } else {
                    empty
                }
            }
            InlinedAlignment::Center => empty / 2.,
            InlinedAlignment::Justify => 0.,
            InlinedAlignment::SpaceAround => offset,
        };
        for (_, size, x, y) in &mut commands[begin..] {
            let delta = line_height - size.height;
            let baseline = match v.baseline {
                VerticalAlignment::Top => 0.,
                VerticalAlignment::Middle => delta / 2.,
                VerticalAlignment::Bottom => delta,
            };
            if size.height == 0. {
                size.height = line_height;
            }
            *x = if rtl {
                s.width - size.width - left
            } else {
                left
            };
            *y = top + baseline;
            left += size.width + offset;
        }
        width = width.max(line_width + (count - 1) as f32 * v.horizontal_spacing);
        height += line_height as f64;
        lines += 1;
        top += line_height + v.vertical_spacing;
    }
    Ok((
        commands,
        Size::new(
            width,
            height as f32 + lines.saturating_sub(1) as f32 * v.vertical_spacing,
        ),
    ))
}
fn multi_column_measure(
    content: &Element,
    count: usize,
    spacing: f32,
    balance: bool,
    s: Size,
    c: &Context,
    e: &Environment,
) -> Result<SpacePlan> {
    if count < 2 {
        bail!("MultiColumn requires at least two columns")
    }
    let column_width = (s.width - spacing * count.saturating_sub(1) as f32) / count as f32;
    let measure = |height: f32| -> Result<(SpacePlan, SpacePlan, f32)> {
        let states = content.snapshot();
        let result = (|| {
            let mut first = SpacePlan::empty();
            let mut last = SpacePlan::empty();
            let mut max = 0f32;
            let area = Size::new(column_width, height);
            for i in 0..count {
                let m = content.measure(area, c, e)?;
                content.draw(area, ptr::null_mut(), c, e)?;
                if i == 0 {
                    first = m
                }
                last = m;
                max = max.max(m.size.height)
            }
            Ok((first, last, max))
        })();
        restore(states);
        result
    };
    let (first, last, height) = measure(s.height)?;
    if matches!(first.kind, SpacePlanType::Empty | SpacePlanType::Wrap) {
        return Ok(first);
    }
    if matches!(
        last.kind,
        SpacePlanType::PartialRender | SpacePlanType::Wrap
    ) {
        return Ok(SpacePlan::partial(s.width, height));
    }
    if !balance {
        return Ok(SpacePlan::full(s.width, height));
    }
    let (mut min, mut max) = (0., s.height);
    for _ in 0..8 {
        let mid = (min + max) / 2.;
        if measure(mid)?.1.done() {
            max = mid
        } else {
            min = mid
        }
    }
    Ok(SpacePlan::full(s.width, max))
}
fn layout_paragraph(p: &mut Paragraph, width: f32, c: &Context) -> Result<()> {
    let a = &c.fonts.api;
    unsafe {
        (a.questpdf_skia_paragraph_plan_layout)(p.handle.ptr, width);
        let mut w = 0.;
        let mut h = 0.;
        (a.questpdf_skia_paragraph_get_size)(p.handle.ptr, &mut w, &mut h);
        p.total_width = w;
        p.width = width;
        let mut array = ptr::null_mut();
        let mut len = 0;
        (a.questpdf_skia_paragraph_get_line_extents)(p.handle.ptr, &mut array, &mut len);
        p.lines = if len > 0 {
            std::slice::from_raw_parts(array, len as usize).to_vec()
        } else {
            vec![]
        };
        (a.questpdf_skia_paragraph_delete_line_extents)(array);
        let mut positions = ptr::null_mut();
        let mut length = 0;
        (a.questpdf_skia_paragraph_get_placeholder_positions)(
            p.handle.ptr,
            &mut positions,
            &mut length,
        );
        p.placeholders = if length > 0 {
            std::slice::from_raw_parts(positions, length as usize).to_vec()
        } else {
            vec![]
        };
        (a.questpdf_skia_paragraph_delete_positions)(positions);
        if c.throw_on_missing_glyphs {
            let mut array = ptr::null_mut();
            let mut len = 0;
            (a.questpdf_skia_paragraph_get_unresolved_codepoints)(
                p.handle.ptr,
                &mut array,
                &mut len,
            );
            let missing = if len > 0 {
                std::slice::from_raw_parts(array, len as usize).to_vec()
            } else {
                vec![]
            };
            (a.questpdf_skia_paragraph_delete_unresolved_codepoints)(array);
            if !missing.is_empty() {
                bail!("missing text glyphs: {missing:?}")
            }
        }
    }
    Ok(())
}
fn lines_to_draw(p: &Paragraph, start: usize, height: f32) -> (usize, f32) {
    if start >= p.lines.len() {
        return (0, 0.);
    }
    let top = p.lines[start].top;
    let count = p.lines[start..]
        .iter()
        .take_while(|l| l.bottom - top < height + EPSILON)
        .count();
    (
        count,
        if count > 0 {
            p.lines[start + count - 1].bottom - top
        } else {
            0.
        },
    )
}
fn aspect_size(s: Size, r: f32, fit: AspectRatioOption) -> Size {
    if r == 0. {
        return Size::ZERO;
    }
    match fit {
        AspectRatioOption::FitWidth => Size::new(s.width, s.width / r),
        AspectRatioOption::FitHeight => Size::new(s.height * r, s.height),
        AspectRatioOption::FitArea => {
            if r < s.width / s.height {
                Size::new(s.height * r, s.height)
            } else {
                Size::new(s.width, s.width / r)
            }
        }
    }
}
fn column_plan(
    items: &[Element],
    spacing: f32,
    start: usize,
    s: Size,
    c: &Context,
    e: &Environment,
) -> Result<Vec<(usize, SpacePlan, f32)>> {
    let mut out = vec![];
    let mut top = 0.;
    for (i, item) in items.iter().enumerate().skip(start) {
        let mut height = s.height - top;
        if height < -EPSILON {
            break;
        }
        height = height.max(0.);
        if !out.is_empty() {
            height -= spacing
        }
        let zero = height < EPSILON;
        let m = item.measure(
            if zero {
                Size::ZERO
            } else {
                Size::new(s.width, height)
            },
            c,
            e,
        )?;
        if m.kind == SpacePlanType::Wrap {
            break;
        }
        let takes = m.size.width.abs() > EPSILON || m.size.height.abs() > EPSILON;
        if zero && takes {
            break;
        }
        if !out.is_empty() && takes {
            top += spacing
        }
        out.push((i, m, top));
        if m.kind == SpacePlanType::PartialRender {
            break;
        }
        top += m.size.height;
    }
    Ok(out)
}
fn row_plan(
    items: &mut [RowItem],
    spacing: f32,
    s: Size,
    c: &Context,
    e: &Environment,
) -> Result<Vec<(Size, SpacePlan, f32)>> {
    let mut constant = 0.;
    let mut relative = 0.;
    for i in items.iter_mut() {
        match i.kind {
            RowItemType::Auto => {
                if i.value == 0. {
                    i.value = i.element.measure(Size::MAX, c, e)?.size.width
                }
                constant += i.value
            }
            RowItemType::Constant => constant += i.value,
            RowItemType::Relative => relative += i.value,
        }
    }
    let unit = if relative > 0. {
        (s.width - constant - spacing * items.len().saturating_sub(1) as f32) / relative
    } else {
        0.
    };
    let mut out = vec![];
    let mut x = 0.;
    let mut tallest = 0f32;
    for i in items.iter() {
        let w = match i.kind {
            RowItemType::Relative => unit * i.value,
            _ => i.value,
        };
        let sz = Size::new(w, s.height);
        let m = if w < -EPSILON {
            SpacePlan::wrap()
        } else {
            i.element.measure(sz, c, e)?
        };
        if !i.rendered {
            tallest = tallest.max(m.size.height)
        }
        out.push((sz, m, x));
        x += w + spacing;
    }
    if out.iter().any(|(_, m, _)| m.kind == SpacePlanType::Wrap) {
        return Ok(out);
    }
    for (i, (sz, m, _)) in items.iter().zip(out.iter_mut()) {
        sz.height = tallest;
        *m = i.element.measure(*sz, c, e)?;
    }
    Ok(out)
}
fn perfect_scale(child: &Element, s: Size, c: &Context, e: &Environment) -> Result<Option<f32>> {
    let fits = |k| -> Result<bool> {
        Ok(child
            .measure(Size::new(s.width / k, s.height / k), c, e)?
            .done())
    };
    if fits(1.)? {
        return Ok(Some(1.));
    }
    let (mut max, mut min, mut good) = (1., EPSILON, None);
    for _ in 0..8 {
        let half = (max + min) / 2.;
        if fits(half)? {
            min = half;
            good = Some(half)
        } else {
            max = half
        }
    }
    Ok(good)
}
fn decoration_plan(
    before: &Element,
    content: &Element,
    after: &Element,
    s: Size,
    c: &Context,
    e: &Environment,
) -> Result<(SpacePlan, SpacePlan, SpacePlan)> {
    let h = before.measure(s, c, e)?;
    let f = after.measure(s, c, e)?;
    let m = content.measure(
        Size::new(s.width, s.height - h.size.height - f.size.height),
        c,
        e,
    )?;
    Ok((h, m, f))
}
pub(crate) fn draw_at(
    child: &Element,
    s: Size,
    x: f32,
    y: f32,
    canvas: Ptr,
    c: &Context,
    e: &Environment,
) -> Result<()> {
    let a = &c.fonts.api;
    if !canvas.is_null() {
        unsafe { a.questpdf_skia_canvas_translate(canvas, x, y) }
    }
    let result = child.draw(s, canvas, c, e);
    if !canvas.is_null() {
        unsafe { a.questpdf_skia_canvas_translate(canvas, -x, -y) }
    }
    result
}
fn draw_scaled(
    child: &Element,
    s: Size,
    x: f32,
    y: f32,
    canvas: Ptr,
    c: &Context,
    e: &Environment,
) -> Result<()> {
    let a = &c.fonts.api;
    let offset = (
        if x < 0. { s.width * x.abs() } else { 0. },
        if y < 0. { s.height * y.abs() } else { 0. },
    );
    if !canvas.is_null() {
        unsafe {
            a.questpdf_skia_canvas_translate(canvas, offset.0, offset.1);
            a.questpdf_skia_canvas_scale(canvas, x, y)
        }
    }
    let result = child.draw(s, canvas, c, e);
    if !canvas.is_null() {
        unsafe {
            a.questpdf_skia_canvas_scale(canvas, 1. / x, 1. / y);
            a.questpdf_skia_canvas_translate(canvas, -offset.0, -offset.1);
        }
    }
    result
}
fn paint(color: Color, stroke: Option<f32>, c: &Context) -> Result<Handle> {
    let a = &c.fonts.api;
    unsafe {
        let p = Handle::new(
            (a.questpdf_skia_paint_create)(),
            a.questpdf_skia_paint_delete,
            a,
        )?;
        (a.questpdf_skia_paint_set_solid_color)(p.ptr, color.0);
        if let Some(w) = stroke {
            (a.questpdf_skia_paint_set_stroke)(p.ptr, w)
        }
        Ok(p)
    }
}
pub(crate) fn paint_rect(
    canvas: Ptr,
    s: Size,
    color: Color,
    stroke: Option<f32>,
    c: &Context,
) -> Result<()> {
    if canvas.is_null() || color.0 == 0 {
        return Ok(());
    }
    let p = paint(color, stroke, c)?;
    unsafe {
        c.fonts.api.questpdf_skia_canvas_draw_rectangle(
            canvas,
            &native::Rect {
                left: 0.,
                top: 0.,
                right: s.width,
                bottom: s.height,
            },
            p.ptr,
        )
    }
    Ok(())
}
fn box_paint(
    s: Size,
    color: Color,
    colors: &[Color],
    angle: f32,
    c: &Context,
) -> Result<Option<Handle>> {
    if colors.is_empty() {
        return if color.0 == 0 {
            Ok(None)
        } else {
            Ok(Some(paint(color, None, c)?))
        };
    }
    let point = |x, y| native::Point { x, y };
    let (start, end) = if angle == 0. {
        (point(0., 0.), point(s.width, 0.))
    } else if angle == 90. {
        (point(0., 0.), point(0., s.height))
    } else if angle == 180. {
        (point(s.width, 0.), point(0., 0.))
    } else if angle == 270. {
        (point(0., s.height), point(s.width, s.height))
    } else {
        let radians = (std::f64::consts::PI * angle as f64 / 180.) as f32;
        let (dx, dy) = ((radians as f64).cos() as f32, (radians as f64).sin() as f32);
        let center = point(s.width / 2., s.height / 2.);
        let mut projected = vec![];
        for p in [
            point(0., 0.),
            point(s.width, 0.),
            point(s.width, s.height),
            point(0., s.height),
        ] {
            let t = (p.x - center.x) * dx + (p.y - center.y) * dy;
            projected.push(point(center.x + dx * t, center.y + dy * t));
        }
        let mut start = projected[0];
        let mut end = start;
        for p in projected {
            if p.x < start.x {
                start = p;
            }
            if p.x > end.x {
                end = p;
            }
        }
        (start, end)
    };
    let paint = paint(Color::BLACK, None, c)?;
    let colors: Vec<_> = colors.iter().map(|c| c.0).collect();
    unsafe {
        (c.fonts.api.questpdf_skia_paint_set_linear_gradient)(
            paint.ptr,
            &start,
            &end,
            colors.len() as i32,
            colors.as_ptr(),
        );
    }
    Ok(Some(paint))
}
fn draw_styled_box(
    style: &BoxStyle,
    child: &Element,
    s: Size,
    canvas: Ptr,
    c: &Context,
    e: &Environment,
) -> Result<()> {
    if canvas.is_null() {
        return child.draw(s, canvas, c, e);
    }
    let a = &c.fonts.api;
    let has_radius = style.radius.iter().any(|r| *r > 0.);
    let simple = !style.extended && style.border.iter().all(|v| *v == style.border[0]);
    if simple {
        {
            let _scope = crate::semantic::Scope::artifact(c, canvas, -8);
            paint_rect(canvas, s, style.background, None, c)?;
        }
        child.draw(s, canvas, c, e)?;
        if style.border[0] > 0. {
            let _scope = crate::semantic::Scope::artifact(c, canvas, -6);
            paint_rect(canvas, s, style.border_color, Some(style.border[0]), c)?;
        }
        return Ok(());
    }
    let background = box_paint(
        s,
        style.background,
        &style.background_gradient,
        style.background_angle,
        c,
    )?;
    let border = box_paint(
        s,
        style.border_color,
        &style.border_gradient,
        style.border_angle,
        c,
    )?;
    let align = style
        .border_alignment
        .unwrap_or(if has_radius { 0. } else { 0.5 });
    let primary = rounded(s, style.radius, [0.; 4]);
    if let Some(shadow) = style.shadow {
        let _scope = crate::semantic::Scope::artifact(c, canvas, -8);
        let rect = rounded(s, style.radius, [shadow.spread; 4]);
        let config = native::BoxShadow {
            offset_x: shadow.offset_x,
            offset_y: shadow.offset_y,
            blur: shadow.blur,
            color: shadow.color.0,
        };
        unsafe {
            a.questpdf_skia_canvas_draw_shadow(canvas, &rect, &config);
        }
    }
    if has_radius {
        unsafe {
            a.questpdf_skia_canvas_save(canvas);
            a.questpdf_skia_canvas_clip_rounded_rectangle(canvas, &primary);
        }
    }
    if let Some(background) = background {
        let _scope = crate::semantic::Scope::artifact(c, canvas, -8);
        unsafe {
            a.questpdf_skia_canvas_draw_rectangle(canvas, &primary.rect, background.ptr);
        }
    }
    child.draw(s, canvas, c, e)?;
    if has_radius {
        unsafe {
            a.questpdf_skia_canvas_restore(canvas);
        }
    }
    if let Some(border) = border {
        let _scope = crate::semantic::Scope::artifact(c, canvas, -6);
        let inner = rounded(s, style.radius, style.border.map(|v| v * (align - 1.)));
        let outer = rounded(s, style.radius, style.border.map(|v| v * align));
        unsafe {
            a.questpdf_skia_canvas_draw_complex_border(canvas, &inner, &outer, border.ptr);
        }
    }
    Ok(())
}
fn rounded(s: Size, r: [f32; 4], expand: [f32; 4]) -> native::RoundedRect {
    let has_radius = r.iter().any(|r| *r > 0.);
    let point = |v: f32, x: usize, y: usize| native::Point {
        x: if has_radius {
            (v + expand[x]).max(0.)
        } else {
            0.
        },
        y: if has_radius {
            (v + expand[y]).max(0.)
        } else {
            0.
        },
    };
    native::RoundedRect {
        rect: native::Rect {
            left: -expand[0],
            top: -expand[1],
            right: s.width + expand[2],
            bottom: s.height + expand[3],
        },
        tl: point(r[0], 0, 1),
        tr: point(r[1], 2, 1),
        br: point(r[2], 2, 3),
        bl: point(r[3], 0, 3),
    }
}
fn register_table_section(table: &Table, section: usize, c: &Context) -> Result<()> {
    if table.semantic_sections[section].get().is_some() {
        return Ok(());
    }
    let cells = match section {
        0 => &table.header,
        1 => &table.cells,
        _ => &table.footer,
    };
    if cells.is_empty() {
        return Ok(());
    }
    let positions = place_cells(cells, table.columns.len())?;
    let mut tree = c.semantic_tree.borrow_mut();
    let Some(tree) = tree.as_mut() else {
        return Ok(());
    };
    if tree.artifacts > 0
        || tree
            .stack
            .last()
            .and_then(|id| tree.nodes.get(id - 1))
            .is_none_or(|n| n.info.kind != "Table")
    {
        return Ok(());
    }
    let initial = c.total == 0;
    let section_id = tree.register(
        crate::SemanticInfo::new(match section {
            0 => "THead",
            1 => "TBody",
            _ => "TFoot",
        }),
        initial,
    );
    table.semantic_sections[section].set(Some(section_id));
    tree.stack.push(section_id);
    let mut order: Vec<_> = (0..cells.len()).collect();
    order.sort_by_key(|i| (positions[*i].row, positions[*i].column));
    let mut row = None;
    for index in order {
        let position = positions[index];
        if row != Some(position.row) {
            if row.is_some() {
                tree.stack.pop();
            }
            let row_id = tree.register(crate::SemanticInfo::new("TR"), initial);
            tree.stack.push(row_id);
            row = Some(position.row);
        }
        let mut info =
            crate::SemanticInfo::new(if section == 0 || position.semantic_horizontal_header {
                "TH"
            } else {
                "TD"
            });
        for (name, value) in [
            ("ColSpan", position.column_span),
            ("RowSpan", position.row_span),
        ] {
            if value > 1 {
                info.attributes.push(crate::SemanticAttribute {
                    owner: "Table".into(),
                    name: name.into(),
                    value: crate::SemanticAttributeValue::Integer(value as i32),
                });
            }
        }
        let id = tree.register(info, initial);
        cells[index].semantic_id.set(Some(id));
    }
    if row.is_some() {
        tree.stack.pop();
    }
    tree.stack.pop();
    if !initial || section == 2 {
        return Ok(());
    }
    let advanced = table.header.iter().chain(&table.cells).any(|cell| {
        let p = cell.position.borrow();
        p.row_span > 1 || p.column_span > 1
    });
    let header_positions = place_cells(&table.header, table.columns.len())?;
    for (index, cell) in cells.iter().enumerate() {
        let position = positions[index];
        let id = cell.semantic_id.get().unwrap();
        let mut attributes = vec![];
        if advanced {
            let headers: Vec<_> = if section == 0 {
                cells
                    .iter()
                    .zip(&positions)
                    .filter(|(_, p)| {
                        p.row < position.row
                            && p.column < position.column + position.column_span
                            && position.column < p.column + p.column_span
                    })
                    .filter_map(|(cell, _)| cell.semantic_id.get())
                    .collect()
            } else {
                table
                    .header
                    .iter()
                    .zip(&header_positions)
                    .filter(|(_, p)| {
                        p.column < position.column + position.column_span
                            && position.column < p.column + p.column_span
                    })
                    .filter_map(|(cell, _)| cell.semantic_id.get())
                    .chain(
                        cells
                            .iter()
                            .zip(&positions)
                            .filter(|(_, p)| {
                                p.semantic_horizontal_header
                                    && p.column < position.column
                                    && p.row < position.row + position.row_span
                                    && position.row < p.row + p.row_span
                            })
                            .filter_map(|(cell, _)| cell.semantic_id.get()),
                    )
                    .collect()
            };
            if !headers.is_empty() {
                attributes.push(crate::SemanticAttribute {
                    owner: "Table".into(),
                    name: "Headers".into(),
                    value: crate::SemanticAttributeValue::NodeIds(
                        headers.into_iter().map(|id| id as i32).collect(),
                    ),
                });
            }
        } else if section == 0 || position.semantic_horizontal_header {
            attributes.push(crate::SemanticAttribute {
                owner: "Table".into(),
                name: "Scope".into(),
                value: crate::SemanticAttributeValue::Text(
                    match (section == 0, position.semantic_horizontal_header) {
                        (true, true) => "Both",
                        (true, false) => "Column",
                        _ => "Row",
                    }
                    .into(),
                ),
            });
        }
        tree.nodes[id - 1].info.attributes.extend(attributes);
    }
    Ok(())
}
struct TableCommand {
    section: u8,
    index: usize,
    size: Size,
    x: f32,
    y: f32,
    measurement: SpacePlan,
}
struct TablePlan {
    commands: Vec<TableCommand>,
    measurement: SpacePlan,
    next_row: usize,
    section_offsets: [f32; 3],
}
fn table_plan(t: &Table, start: usize, s: Size, c: &Context, e: &Environment) -> Result<TablePlan> {
    let n = t.columns.len();
    if n == 0 {
        bail!("Table requires at least one column")
    }
    let constant = t
        .columns
        .iter()
        .map(|x| {
            if let ColumnDefinition::Constant(v) = x {
                *v as f64
            } else {
                0.
            }
        })
        .sum::<f64>() as f32;
    let relative = t
        .columns
        .iter()
        .map(|x| {
            if let ColumnDefinition::Relative(v) = x {
                *v as f64
            } else {
                0.
            }
        })
        .sum::<f64>() as f32;
    let unit = if relative > 0. {
        (s.width - constant) / relative
    } else {
        0.
    };
    let raw_widths: Vec<f32> = t
        .columns
        .iter()
        .map(|x| match x {
            ColumnDefinition::Constant(v) => *v,
            ColumnDefinition::Relative(v) => v * unit,
        })
        .collect();
    let width = raw_widths.iter().map(|w| *w as f64).sum::<f64>() as f32;
    let mut offsets = vec![0f32];
    for w in &raw_widths {
        offsets.push(offsets.last().unwrap() + w)
    }
    let header = table_section(&t.header, n, &offsets, width, 1, s.height, 0, c, e)?;
    let footer = table_section(&t.footer, n, &offsets, width, 1, s.height, 2, c, e)?;
    if !header.measurement.done() || !footer.measurement.done() {
        return Ok(TablePlan {
            section_offsets: [0.; 3],
            commands: vec![],
            measurement: SpacePlan::wrap(),
            next_row: start,
        });
    }
    let mut content = table_section(
        &t.cells,
        n,
        &offsets,
        width,
        start.max(1),
        s.height - header.measurement.size.height - footer.measurement.size.height,
        1,
        c,
        e,
    )?;
    if t.extend_last_cells {
        let positions = place_cells(&t.cells, n)?;
        let mut bottom_rows = vec![0; n];
        let mut height = 0f32;
        for command in &content.commands {
            height = height.max(command.y + command.size.height);
            let cell = positions[command.index];
            for value in bottom_rows
                .iter_mut()
                .skip(cell.column - 1)
                .take(cell.column_span)
            {
                *value = (*value).max(cell.row + cell.row_span);
            }
        }
        for command in &mut content.commands {
            let cell = positions[command.index];
            if (cell.column - 1..cell.column - 1 + cell.column_span)
                .any(|column| bottom_rows[column] == cell.row + cell.row_span)
            {
                command.size.height = height - command.y;
            }
        }
    }
    if matches!(
        content.measurement.kind,
        SpacePlanType::Empty | SpacePlanType::Wrap
    ) {
        return Ok(content);
    }
    let head_h = header.measurement.size.height;
    let content_h = content.measurement.size.height;
    let mut commands = header.commands;
    commands.extend(content.commands);
    commands.extend(footer.commands);
    Ok(TablePlan {
        section_offsets: [0., head_h, head_h + content_h],
        commands,
        measurement: SpacePlan {
            kind: content.measurement.kind,
            size: Size::new(width, head_h + content_h + footer.measurement.size.height),
        },
        next_row: content.next_row,
    })
}
fn place_cells(cells: &[TableCell], columns: usize) -> Result<Vec<CellPosition>> {
    let mut free = vec![1; columns + 1];
    let (mut cursor_column, mut cursor_row) = (1, 1);
    let mut out = vec![];
    for cell in cells {
        let mut p = *cell.position.borrow();
        if p.column_span == 0 || p.row_span == 0 || p.column_span > columns {
            bail!("Table cell span is outside table bounds")
        }
        let unoccupied = |column: usize, row: usize| -> bool {
            (column..column + p.column_span).all(|i| free[i] <= row)
        };
        if p.column == 0 && p.row == 0 {
            let (mut column, mut row) = (cursor_column.max(1), cursor_row.max(1));
            loop {
                if column > columns - p.column_span + 1 {
                    column = 1;
                    row += 1;
                    continue;
                }
                if unoccupied(column, row) {
                    p.column = column;
                    p.row = row;
                    break;
                }
                column += 1
            }
        } else if p.row == 0 {
            p.row = cursor_row.max(1);
            if p.column + p.column_span - 1 <= columns {
                for value in free.iter().skip(p.column).take(p.column_span) {
                    p.row = p.row.max(*value)
                }
            }
        } else if p.column == 0 {
            p.column = (1..=columns - p.column_span + 1)
                .find(|col| unoccupied(*col, p.row))
                .unwrap_or(1)
        }
        if p.column < 1
            || p.column + p.column_span - 1 > columns
            || p.row > 1_000_000
            || p.row_span > 1_000_000
        {
            bail!("Table cell position is outside valid bounds")
        }
        for value in free.iter_mut().skip(p.column).take(p.column_span) {
            *value = (*value).max(p.row + p.row_span)
        }
        cursor_column = p.column;
        cursor_row = p.row;
        out.push(p);
    }
    Ok(out)
}
// These explicit inputs mirror the source planner's layout and environment state.
#[allow(clippy::too_many_arguments)]
fn table_section(
    cells: &[TableCell],
    columns: usize,
    offsets: &[f32],
    width: f32,
    start: usize,
    height: f32,
    section: u8,
    c: &Context,
    e: &Environment,
) -> Result<TablePlan> {
    use SpacePlanType::*;
    let positions = place_cells(cells, columns)?;
    let last = positions
        .iter()
        .map(|p| p.row + p.row_span - 1)
        .max()
        .unwrap_or(0);
    if cells.is_empty() || start > last {
        return Ok(TablePlan {
            section_offsets: [0.; 3],
            commands: vec![],
            measurement: SpacePlan::empty(),
            next_row: last + 1,
        });
    }
    let max_span = positions.iter().map(|p| p.row_span).max().unwrap_or(1);
    let mut bottoms = vec![0f32; last + 1];
    let mut commands: Vec<TableCommand> = vec![];
    let mut current = start;
    let mut max_row = last;
    for row in start..=last {
        let mut bucket: Vec<_> = positions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.row + p.row_span - 1 == row)
            .map(|(i, _)| i)
            .collect();
        bucket.sort_by_key(|i| (positions[*i].column, positions[*i].row, *i));
        if bucket.is_empty() {
            continue;
        }
        if row > current {
            bottoms[current] = bottoms[current].max(bottoms[current - 1]);
            if bottoms[current - 1] > height + EPSILON {
                break;
            }
            for gap in current + 1..row {
                bottoms[gap] = bottoms[gap].max(bottoms[gap - 1])
            }
            current = row
        }
        if row > max_row + max_span {
            break;
        }
        for index in bucket {
            let p = positions[index];
            let y = bottoms[p.row - 1];
            let w = offsets[p.column + p.column_span - 1] - offsets[p.column - 1];
            let space = Size::new(w, height - y);
            let m = if cells[index].rendered.get() {
                SpacePlan::empty()
            } else {
                cells[index].element.measure(space, c, e)?
            };
            if m.kind == PartialRender {
                max_row = max_row.min(p.row + p.row_span - 1)
            }
            if m.kind == Wrap {
                max_row = max_row.min(p.row - 1);
                continue;
            }
            let end = p.row + p.row_span - 1;
            bottoms[end] = bottoms[end].max(y + m.size.height);
            commands.push(TableCommand {
                section,
                index,
                size: Size::new(w, m.size.height),
                x: offsets[p.column - 1],
                y,
                measurement: m,
            });
        }
    }
    for row in start..=last {
        bottoms[row] = bottoms[row].max(bottoms[row - 1])
    }
    for cmd in &mut commands {
        let p = positions[cmd.index];
        cmd.size.height = bottoms[p.row + p.row_span - 1] - cmd.y;
        cmd.y = bottoms[p.row - 1]
    }
    commands.retain(|cmd| positions[cmd.index].row <= max_row);
    if commands.is_empty() {
        return Ok(TablePlan {
            section_offsets: [0.; 3],
            commands,
            measurement: SpacePlan::wrap(),
            next_row: start,
        });
    }
    let mut next = start;
    while next <= last {
        if positions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.row + p.row_span - 1 == next)
            .all(|(i, _)| {
                cells[i].rendered.get()
                    || commands
                        .iter()
                        .any(|cmd| cmd.index == i && cmd.measurement.done())
            })
        {
            next += 1
        } else {
            break;
        }
    }
    let h = commands
        .iter()
        .map(|cmd| cmd.y + cmd.size.height)
        .fold(0., f32::max);
    Ok(TablePlan {
        section_offsets: [0.; 3],
        commands,
        measurement: if next > last {
            SpacePlan::full(width, h)
        } else {
            SpacePlan::partial(width, h)
        },
        next_row: next,
    })
}
