use questpdf::*;
fn main() -> anyhow::Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let output = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or("output/exports-rust".into()),
    );
    std::fs::create_dir_all(&output)?;
    let document = Document::create(|c| {
        c.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.default_text_style(|s| s.font_size(10.));
            p.content().column(|c| {
                c.spacing(5.);
                c.item().text_spans(|t| {
                    t.span("Superscript ");
                    t.span("2").superscript();
                    t.span(" subscript ");
                    t.span("3").subscript();
                });
                c.item()
                    .text("office affinity ffi fi fl")
                    .enable_font_feature("liga");
                c.item()
                    .text("Decorated text")
                    .underline()
                    .overline()
                    .decoration_style(TextDecorationStyle::Wavy)
                    .decoration_color(Color(0xffff0000));
                c.item()
                    .width(110.)
                    .text("averylongwordwithoutspaces")
                    .break_anywhere(true);
                c.item().height(20.).background(Color(0x8088bbff));
            });
        })
    });
    document.generate_pdf(output.join("text-styles.pdf"))?;
    for (name, format, transparent) in [
        ("opaque", ImageFormat::Png, false),
        ("transparent", ImageFormat::Png, true),
        ("jpeg", ImageFormat::Jpeg, true),
        ("webp", ImageFormat::Webp, true),
    ] {
        let settings = ImageGenerationSettings {
            image_format: format,
            use_transparent_background: transparent,
            raster_dpi: 150.,
            ..Default::default()
        };
        let bytes = document.generate_images_with_settings(&settings)?;
        let extension = match format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Webp => "webp",
        };
        std::fs::write(output.join(format!("{name}.{extension}")), &bytes[0])?;
    }
    std::fs::write(output.join("page.svg"), &document.generate_svg()?[0])?;
    FontManager::register_font_from_file("document-server/assets/BAHNSCHRIFT.TTF")?;
    std::fs::write(
        output.join("fonts-registered.json"),
        serde_json::to_vec(&FontManager::get_registered_fonts()?)?,
    )?;
    std::fs::write(
        output.join("fonts-system.json"),
        serde_json::to_vec(&FontManager::get_system_fonts()?)?,
    )?;
    assert!(FontManager::is_font_family_available("Bahnschrift")?);
    let doc = Document::create(|c| {
        c.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.content()
                .text("Application font registration")
                .font_family("Bahnschrift")
                .font_size(12.);
        })
    });
    doc.generate_pdf(output.join("font-management.pdf"))?;
    Ok(())
}
