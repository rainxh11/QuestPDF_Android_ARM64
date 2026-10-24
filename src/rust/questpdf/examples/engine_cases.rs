//! Differential fixtures are also interpreted by the independent C# oracle.
use anyhow::{Result, bail};
use questpdf::*;
use serde_json::Value;
fn main() -> Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let args: Vec<_> = std::env::args().collect();
    let cases: Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    FontManager::set_use_system_fonts(
        cases
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["system_fonts"].as_bool().unwrap_or(false)),
    );
    std::fs::create_dir_all(&args[2])?;
    for case in cases.as_array().unwrap() {
        let name = s(case, "name");
        for extension in ["pdf", "error"] {
            let path = std::path::Path::new(&args[2]).join(format!("{name}.{extension}"));
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        let result = {
            let mut doc = Document::create(|c| {
                c.page(|p| {
                    let size = case["size"].as_array().unwrap();
                    let width = size[0].as_f64().unwrap() as f32;
                    let height = size[1].as_f64().unwrap() as f32;
                    if case["continuous"].as_bool().unwrap_or(false) {
                        p.continuous_size(width, Unit::Point)
                    } else {
                        p.size(Size::new(width, height))
                    }
                    if let Some(min) = case["min_size"].as_array() {
                        p.min_size(Size::new(
                            min[0].as_f64().unwrap() as f32,
                            min[1].as_f64().unwrap() as f32,
                        ));
                    }
                    if let Some(max) = case["max_size"].as_array() {
                        p.max_size(Size::new(
                            max[0].as_f64().unwrap() as f32,
                            max[1].as_f64().unwrap() as f32,
                        ));
                    }
                    p.margin(10.);
                    p.default_text_style(|s| s.font_size(10.));
                    if case["rtl"].as_bool().unwrap_or(false) {
                        p.content_from_right_to_left()
                    }
                    if !case["header"].is_null() {
                        compose(&case["header"], p.header()).expect("valid header");
                    }
                    compose(&case["content"], p.content()).expect("valid fixture");
                    p.footer().align_center().text_spans(|t| {
                        t.span("Page ");
                        t.current_page_number();
                        t.span(" of ");
                        t.total_pages();
                    });
                })
            });
            doc.metadata.title = name.into();
            if case["global_rtl"].as_bool().unwrap_or(false) {
                doc.settings.content_direction = ContentDirection::RightToLeft;
            }
            doc.metadata.language = "en-US".into();
            doc.settings.pdfa_conformance = match s(case, "pdfa") {
                "PDFA_2A" => PdfAConformance::PdfA2A,
                "PDFA_2B" => PdfAConformance::PdfA2B,
                "PDFA_2U" => PdfAConformance::PdfA2U,
                "PDFA_3A" => PdfAConformance::PdfA3A,
                "PDFA_3B" => PdfAConformance::PdfA3B,
                "PDFA_3U" => PdfAConformance::PdfA3U,
                _ => PdfAConformance::None,
            };
            if case["pdfua"].as_bool().unwrap_or(false) {
                doc.settings.pdfua_conformance = PdfUaConformance::PdfUa1;
            }
            doc.generate_pdf(format!("{}/{name}.pdf", args[2]))
        };
        if let Err(error) = result {
            std::fs::write(format!("{}/{name}.error", args[2]), error.to_string())?;
            println!("{name} ERROR")
        } else {
            println!("{name}")
        }
    }
    Ok(())
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn f(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map(|x| x as f32).unwrap_or(default)
}
fn compose(n: &Value, c: Container) -> Result<()> {
    let child = |c: Container| compose(&n["child"], c);
    match s(n, "kind") {
        "empty" => {}
        "text" => {
            let t = c.text(s(n, "text")).font_size(f(n, "size", 10.));
            if n["bold"].as_bool().unwrap_or(false) {
                t.bold();
            }
            if n["line_height"].is_number() {
                t.line_height(f(n, "line_height", 1.));
            }
            if let Some(family) = n["font_family"].as_str() {
                t.font_family(family);
            }
            if let Some(clamp) = n["clamp"].as_u64() {
                t.clamp_lines(clamp as usize, n["ellipsis"].as_str().unwrap_or("…"));
            }
            if n["paragraph_spacing"].is_number() {
                t.paragraph_spacing(f(n, "paragraph_spacing", 0.));
            }
            if n["indent"].is_number() {
                t.paragraph_first_line_indentation(f(n, "indent", 0.));
            }
            match s(n, "align") {
                "center" => {
                    t.align_center();
                }
                "right" => {
                    t.align_right();
                }
                "justify" => {
                    t.justify();
                }
                _ => {}
            }
        }
        "inlined" => c.inlined(|d| {
            d.horizontal_spacing(f(n, "horizontal", 0.));
            d.vertical_spacing(f(n, "vertical", 0.));
            match s(n, "align") {
                "left" => d.align_left(),
                "right" => d.align_right(),
                "center" => d.align_center(),
                "justify" => d.align_justify(),
                "around" => d.align_space_around(),
                _ => {}
            }
            match s(n, "baseline") {
                "middle" => d.baseline_middle(),
                "bottom" => d.baseline_bottom(),
                _ => {}
            }
            for item in n["items"].as_array().unwrap() {
                compose(item, d.item()).unwrap();
            }
        }),
        "rich_text" => {
            c.text_spans(|t| {
                for item in n["items"].as_array().unwrap() {
                    match s(item, "kind") {
                        "inline" => {
                            let alignment = match s(item, "alignment") {
                                "BelowBaseline" => TextInjectedElementAlignment::BelowBaseline,
                                "Top" => TextInjectedElementAlignment::Top,
                                "Bottom" => TextInjectedElementAlignment::Bottom,
                                "Middle" => TextInjectedElementAlignment::Middle,
                                _ => TextInjectedElementAlignment::AboveBaseline,
                            };
                            compose(&item["content"], t.element(alignment)).unwrap();
                        }
                        "page_number" => {
                            let span = match s(item, "type") {
                                "begin" => t.begin_page_number_of_section(s(item, "name")),
                                "end" => t.end_page_number_of_section(s(item, "name")),
                                "within" => t.page_number_within_section(s(item, "name")),
                                "section_total" => t.total_pages_within_section(s(item, "name")),
                                "total" => t.total_pages(),
                                _ => t.current_page_number(),
                            };
                            if s(item, "format") == "padded" {
                                span.format(|v| {
                                    v.map(|v| format!("{v:03}")).unwrap_or("000".into())
                                });
                            }
                        }
                        "section_link" => {
                            t.section_link(s(item, "text"), s(item, "name")).underline();
                        }
                        "link" => {
                            t.hyperlink(s(item, "text"), s(item, "url")).underline();
                        }
                        _ => {
                            t.span(s(item, "text"));
                        }
                    }
                }
            });
        }
        "column" => c.column(|d| {
            d.spacing(f(n, "spacing", 0.));
            for i in n["items"].as_array().unwrap() {
                compose(i, d.item()).unwrap()
            }
        }),
        "lazy" => {
            let node = n["child"].clone();
            c.lazy(move |c| compose(&node, c).unwrap());
        }
        "dynamic" => c.dynamic(DynamicRows {
            state: 0,
            count: f(n, "count", 30.) as usize,
        }),
        "position_probe" => c.dynamic(PositionProbe(s(n, "name").into())),
        "grid" => c.grid(|d| {
            d.columns(f(n, "count", 12.) as usize);
            d.spacing(f(n, "spacing", 0.));
            match s(n, "align") {
                "center" => d.align_center(),
                "right" => d.align_right(),
                _ => {}
            }
            for item in n["items"].as_array().unwrap() {
                compose(&item["content"], d.item(f(item, "span", 1.) as usize)).unwrap();
            }
        }),
        "dynamic_svg" => {
            let text = s(n, "text").to_owned();
            c.svg_dynamic(move |_| text.clone());
        }
        "image" => {
            let mut image = c.image_file(s(n, "path"))?;
            if n["dpi"].is_number() {
                image = image.with_raster_dpi(f(n, "dpi", 72.));
            }
            if let Some(value) = n["quality"].as_str() {
                image = image.with_compression_quality(match value {
                    "Best" => ImageCompressionQuality::Best,
                    "VeryHigh" => ImageCompressionQuality::VeryHigh,
                    "Medium" => ImageCompressionQuality::Medium,
                    "Low" => ImageCompressionQuality::Low,
                    "VeryLow" => ImageCompressionQuality::VeryLow,
                    _ => ImageCompressionQuality::High,
                });
            }
            if let Some(v) = n["original"].as_bool() {
                image = image.use_original_image(v);
            }
            if n["stretch"].as_bool().unwrap_or(false) {
                image.fit_unproportionally();
            }
        }
        "dynamic_image" => {
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD.decode(s(n, "data"))?;
            let dpi = n["dpi"].as_f64().map(|x| x as f32);
            let mut image = c.image_dynamic(move |payload| {
                if let Some(dpi) = dpi {
                    assert_eq!(payload.dpi, dpi);
                }
                Some(bytes.clone())
            });
            if let Some(dpi) = dpi {
                image = image.with_raster_dpi(dpi);
            }
            if let Some(value) = n["quality"].as_str() {
                image = image.with_compression_quality(match value {
                    "Best" => ImageCompressionQuality::Best,
                    "VeryHigh" => ImageCompressionQuality::VeryHigh,
                    "Medium" => ImageCompressionQuality::Medium,
                    "Low" => ImageCompressionQuality::Low,
                    "VeryLow" => ImageCompressionQuality::VeryLow,
                    _ => ImageCompressionQuality::High,
                });
            }
            if let Some(value) = n["original"].as_bool() {
                image.use_original_image(value);
            }
        }
        "row" => c.row(|d| {
            d.spacing(f(n, "spacing", 0.));
            for i in n["items"].as_array().unwrap() {
                let c = match s(i, "type") {
                    "constant" => d.constant_item(f(i, "value", 0.)),
                    "auto" => d.auto_item(),
                    _ => d.relative_item_weight(f(i, "value", 1.)),
                };
                compose(&i["content"], c).unwrap()
            }
        }),
        "padding" => {
            let p = n["value"].as_array().unwrap();
            child(
                c.padding_left(p[0].as_f64().unwrap() as f32)
                    .padding_top(p[1].as_f64().unwrap() as f32)
                    .padding_right(p[2].as_f64().unwrap() as f32)
                    .padding_bottom(p[3].as_f64().unwrap() as f32),
            )?
        }
        "width" => child(c.width(f(n, "value", 0.)))?,
        "section" => child(c.section(s(n, "name")))?,
        "z_index" => child(c.z_index(f(n, "value", 0.) as i32))?,
        "capture" => child(c.capture_content_position(s(n, "name")))?,
        "semantic" => {
            let container = if s(n, "role") == "Ignore" {
                c.semantic_ignore()
            } else {
                let mut info = SemanticInfo::new(s(n, "role"));
                info.alternative_text = n["alt"].as_str().map(Into::into);
                info.language = n["language"].as_str().map(Into::into);
                c.semantic_tag(info)
            };
            child(container)?;
        }
        "section_link" => child(c.section_link(s(n, "name")))?,
        "hyperlink" => child(c.hyperlink(s(n, "url")))?,
        "height" => child(c.height(f(n, "value", 0.)))?,
        "min_height" => child(c.min_height(f(n, "value", 0.)))?,
        "align_center" => child(c.align_center())?,
        "align_right" => child(c.align_right())?,
        "align_middle" => child(c.align_middle())?,
        "background" => child(c.background(Color::hex(s(n, "value"))?))?,
        "border" => child(c.border(f(n, "value", 1.)).border_color(Color::BLACK))?,
        "styled_box" => {
            let mut b = c;
            if let Some(v) = n["border"].as_array() {
                b = b
                    .border_left(v[0].as_f64().unwrap() as f32)
                    .border_top(v[1].as_f64().unwrap() as f32)
                    .border_right(v[2].as_f64().unwrap() as f32)
                    .border_bottom(v[3].as_f64().unwrap() as f32)
                    .border_color(Color(0xff224466));
            }
            if let Some(v) = n["radius"].as_array() {
                b = b
                    .corner_radius_top_left(v[0].as_f64().unwrap() as f32)
                    .corner_radius_top_right(v[1].as_f64().unwrap() as f32)
                    .corner_radius_bottom_right(v[2].as_f64().unwrap() as f32)
                    .corner_radius_bottom_left(v[3].as_f64().unwrap() as f32);
            }
            b = match s(n, "border_alignment") {
                "inside" => b.border_alignment_inside(),
                "middle" => b.border_alignment_middle(),
                "outside" => b.border_alignment_outside(),
                _ => b,
            };
            b = if n["gradient"].is_number() {
                b.background_linear_gradient(
                    f(n, "gradient", 0.),
                    vec![Color(0xff99aaff), Color(0xffffdd66), Color(0xffaa3388)],
                )
            } else {
                b.background(Color(0xff99bbdd))
            };
            if n["border_gradient"].is_number() {
                b = b.border_linear_gradient(
                    f(n, "border_gradient", 0.),
                    vec![Color(0xff33aa66), Color(0xffdd3333)],
                );
            }
            if n["shadow"].as_bool().unwrap_or(false) {
                b = b.shadow(BoxShadowStyle {
                    offset_x: 4.,
                    offset_y: 6.,
                    spread: 2.,
                    blur: 3.,
                    color: Color(0x88775544),
                });
            }
            child(b)?;
        }
        "stop_paging" => child(c.stop_paging())?,
        "show_bool" => child(c.show_if(n["value"].as_bool().unwrap()))?,
        "border_edges" => {
            let target = match s(n, "edge") {
                "horizontal" => c.border_horizontal(2.),
                "vertical" => c.border_vertical(2.),
                "before_transparent" => c.border_color(Color::TRANSPARENT).border(2.),
                "after_transparent" => c.border(2.).border_color(Color::TRANSPARENT),
                _ => c.border(2.),
            };
            child(target)?;
        }
        "component_border" => c.border(1.).component(&BorderComponent),
        "show_entire" => child(c.show_entire())?,
        "ensure_space" => child(c.ensure_space(f(n, "value", 0.)))?,
        "prevent_page_break" => child(c.prevent_page_break())?,
        "show_once" => child(c.show_once())?,
        "skip_once" => child(c.skip_once())?,
        "repeat" => child(c.repeat())?,
        "page_break" => c.page_break(),
        "scale_to_fit" => child(c.scale_to_fit())?,
        "scale" => child(c.scale(f(n, "value", 1.)))?,
        "scale_xy" => child(
            c.scale_horizontal(f(n, "x", 1.))
                .scale_vertical(f(n, "y", 1.)),
        )?,
        "rotate" => child(c.rotate(f(n, "value", 0.)))?,
        "shrink" => child(c.shrink())?,
        "unconstrained" => child(c.unconstrained())?,
        "debug_area" => child(c.debug_area(s(n, "text"), None))?,
        "show_predicate" => child(c.show_if_predicate(|ctx| ctx.page_number % 2 == 1))?,
        "lazy_cached" => {
            let node = n["child"].clone();
            c.lazy_with_cache(move |child| compose(&node, child).unwrap());
        }

        "rotate_left" => child(c.rotate_left())?,
        "rotate_right" => child(c.rotate_right())?,
        "line" => {
            let line = if n["vertical"].as_bool().unwrap_or(false) {
                c.line_vertical(f(n, "value", 1.))
            } else {
                c.line_horizontal(f(n, "value", 1.))
            };
            if let Some(dash) = n["dash"].as_array() {
                line.line_dash_pattern(dash.iter().map(|x| x.as_f64().unwrap() as f32).collect());
            }
            if n["gradient"].as_bool().unwrap_or(false) {
                line.line_gradient(vec![Color(0xffff3344), Color(0xff33aa66)]);
            }
        }
        "placeholder" => c.placeholder(s(n, "text")),
        "svg" => {
            c.svg(s(n, "text")).fit_area();
        }
        "decoration" => c.decoration(|d| {
            compose(&n["before"], d.before()).unwrap();
            compose(&n["content"], d.content()).unwrap();
            compose(&n["after"], d.after()).unwrap();
        }),
        "layers" => c.layers(|d| {
            for i in n["items"].as_array().unwrap() {
                compose(
                    &i["content"],
                    if i["primary"].as_bool().unwrap() {
                        d.primary_layer()
                    } else {
                        d.layer()
                    },
                )
                .unwrap()
            }
        }),
        "multi_column" => c.multi_column(|d| {
            d.columns(f(n, "count", 2.) as usize);
            d.spacing(f(n, "spacing", 0.));
            d.balance_height(n["balance"].as_bool().unwrap_or(false));
            compose(&n["content"], d.content()).unwrap();
            compose(&n["spacer"], d.spacer()).unwrap();
        }),
        "table" => c.table(|d| {
            if n["extend_last"].as_bool().unwrap_or(false) {
                d.extend_last_cells_to_table_bottom();
            }
            d.columns_definition(|cols| {
                for col in n["columns"].as_array().unwrap() {
                    cols.relative_column_weight(col.as_f64().unwrap() as f32)
                }
            });
            if let Some(h) = n["header"].as_array() {
                d.header(|hdesc| {
                    for cell in h {
                        compose(cell, hdesc.cell().container).unwrap()
                    }
                })
            }
            if let Some(h) = n["footer"].as_array() {
                d.footer(|desc| {
                    for cell in h {
                        compose(cell, desc.cell().container).unwrap();
                    }
                });
            }
            if let Some(h) = n["header_cells"].as_array() {
                d.header(|desc| {
                    for item in h {
                        compose_cell(item, desc);
                    }
                });
            }
            for item in n["cells"].as_array().unwrap() {
                compose_cell(item, d);
            }
        }),
        k => bail!("Unknown fixture node {k}"),
    }
    Ok(())
}
fn compose_cell(item: &Value, d: &mut TableDescriptor) {
    let mut cell = d.cell();
    for key in ["column", "row", "column_span", "row_span"] {
        if let Some(v) = item[key].as_u64() {
            cell = match key {
                "column" => cell.column(v as usize),
                "row" => cell.row(v as usize),
                "column_span" => cell.column_span(v as usize),
                _ => cell.row_span(v as usize),
            };
        }
    }
    if item["semantic_header"].as_bool().unwrap_or(false) {
        cell = cell.semantic_horizontal_header();
    }
    compose(&item["content"], cell.container).unwrap();
}
struct DynamicRows {
    state: usize,
    count: usize,
}
struct PositionProbe(String);
impl IDynamicComponent for PositionProbe {
    type State = ();
    fn state(&self) {}
    fn set_state(&mut self, _: ()) {}
    fn compose(
        &mut self,
        context: &DynamicContext<'_, '_>,
    ) -> Result<DynamicComponentComposeResult> {
        let positions = context.get_content_captured_positions(&self.0);
        let lines = positions
            .iter()
            .map(|p| {
                format!(
                    "Page {}: X={:.3}, Y={:.3}, W={:.3}, H={:.3}",
                    p.page_number, p.x, p.y, p.width, p.height
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let content = context.create_element(|c| {
            c.text(format!("Captured {}\n{lines}", positions.len()))
                .font_size(8.);
        })?;
        Ok(DynamicComponentComposeResult::new(content, false))
    }
}
impl IDynamicComponent for DynamicRows {
    type State = usize;
    fn state(&self) -> usize {
        self.state
    }
    fn set_state(&mut self, state: usize) {
        self.state = state;
    }
    fn compose(
        &mut self,
        context: &DynamicContext<'_, '_>,
    ) -> Result<DynamicComponentComposeResult> {
        let total = if context.total_pages == i32::MAX as usize {
            "?".into()
        } else {
            context.total_pages.to_string()
        };
        let header = context.create_element(|c| {
            c.text(format!("Dynamic page {} / {total}", context.page_number))
                .bold();
        })?;
        let mut height = header.size.height;
        let mut rows = vec![];
        for index in self.state..self.count {
            let row = context.create_element(|c| {
                c.width(context.available_size.width).text(format!(
                    "Row {index:03}: dynamically measured content keeps its state."
                ));
            })?;
            if height + 3. + row.size.height > context.available_size.height + 0.01 {
                break;
            }
            height += 3. + row.size.height;
            rows.push(row);
        }
        self.state += rows.len();
        let content = context.create_element(|c| {
            c.column(|col| {
                col.spacing(3.);
                col.item().dynamic_element(&header);
                for row in &rows {
                    col.item().dynamic_element(row);
                }
            })
        })?;
        Ok(DynamicComponentComposeResult::new(
            content,
            self.state < self.count,
        ))
    }
}

struct BorderComponent;
impl IComponent for BorderComponent {
    fn compose(&self, container: Container) {
        container.border(2.).padding(4.).text("Component boundary");
    }
}
