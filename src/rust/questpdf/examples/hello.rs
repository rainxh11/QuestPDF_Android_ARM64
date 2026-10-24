use questpdf::*;
fn main() -> anyhow::Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let document = Document::create(|container| {
        container.page(|page| {
            page.size(PageSizes::A4);
            page.margin_unit(2., Unit::Centimetre);
            page.default_text_style(|s| s.font_size(20.));
            page.header().text("Hello PDF!").semi_bold().font_size(36.);
            page.content()
                .padding_vertical(Unit::Centimetre.points(1.))
                .column(|column| {
                    column.spacing(20.);
                    column
                        .item()
                        .text("A Rust implementation of QuestPDF's measure / draw layout engine.");
                    column.item().row(|row| {
                        row.relative_item()
                            .background(Color(0xffeeeeee))
                            .padding(10.)
                            .text("Left");
                        row.relative_item().padding(10.).align_right().text("Right");
                    });
                });
            page.footer().align_center().text_spans(|text| {
                text.span("Page ");
                text.current_page_number();
                text.span(" of ");
                text.total_pages();
            });
        })
    });
    let output = std::env::args().nth(1).unwrap_or("hello.pdf".into());
    document.generate_pdf(output)
}
