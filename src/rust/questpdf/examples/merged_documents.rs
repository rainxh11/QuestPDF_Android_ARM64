use anyhow::Result;
use questpdf::*;
fn part(name: &'static str, count: usize) -> Document {
    Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(240., 200.));
            p.margin(10.);
            p.default_text_style(|s| s.font_size(10.));
            p.header().text(name).bold();
            p.content().section("shared").column(|c| {
                for i in 0..count {
                    if i > 0 {
                        c.item().page_break();
                    }
                    c.item().text(format!("{name}, item {}", i + 1));
                    c.item()
                        .semantic_link("Go to section start")
                        .section_link("shared")
                        .text("Go to section start");
                }
            });
            p.footer().text_spans(|t| {
                t.current_page_number();
                t.span(" / ");
                t.total_pages();
                t.span("; section ");
                t.begin_page_number_of_section("shared");
                t.span(" - ");
                t.end_page_number_of_section("shared");
            });
        })
    })
}
fn main() -> Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let directory = std::env::args()
        .nth(1)
        .unwrap_or("output/merged-rust".into());
    std::fs::create_dir_all(&directory)?;
    for continuous in [false, true] {
        for tagged in [false, true] {
            let name = format!("merged-{continuous}-{tagged}");
            let merged = Document::merge([part("Part A", 2), part("Part B", 3)]);
            let merged = if continuous {
                merged.use_continuous_page_numbers()
            } else {
                merged.use_original_page_numbers()
            };
            let settings = DocumentSettings {
                pdfua_conformance: if tagged {
                    PdfUaConformance::PdfUa1
                } else {
                    PdfUaConformance::None
                },
                ..Default::default()
            };
            let metadata = DocumentMetadata {
                title: name.clone(),
                language: "en-US".into(),
                ..Default::default()
            };
            merged
                .with_settings(settings)
                .with_metadata(metadata)
                .generate_pdf(std::path::Path::new(&directory).join(format!("{name}.pdf")))?;
        }
    }
    Ok(())
}
