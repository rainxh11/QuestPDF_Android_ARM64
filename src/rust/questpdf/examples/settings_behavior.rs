use questpdf::*;
fn document() -> Document {
    Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.content()
                .text("Automatic application font discovery")
                .font_family("Bahnschrift")
                .font_size(12.);
        })
    })
}
fn main() -> anyhow::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or("output/settings-rust".into());
    let output = std::path::Path::new(&directory);
    std::fs::create_dir_all(output)?;
    Settings::set_font_discovery_path(Some(std::path::PathBuf::from("document-server/assets")));
    assert!(
        document().generate_pdf_bytes().is_err(),
        "License declaration is required before generation"
    );
    Settings::set_license(LicenseType::Evaluation);
    document().generate_pdf(output.join("evaluation.pdf"))?;
    Settings::set_license(LicenseType::Community);
    document().generate_pdf(output.join("discovery.pdf"))?;
    Settings::set_enable_caching(false);
    document().generate_pdf(output.join("uncached.pdf"))?;
    Settings::configure(|s| {
        s.throw_on_missing_font_families = false;
        s.throw_on_missing_text_glyphs = false;
    });
    Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.content()
                .text("Missing font fallback")
                .font_family("Unregistered Fixture Font")
                .font_size(12.);
        })
    })
    .generate_pdf(output.join("missing-font.pdf"))?;
    let invalid = Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(100., 100.));
            p.content()
                .debug_pointer("fixture overflow label")
                .width(200.)
                .text("Too wide");
        })
    });
    Settings::configure(|s| s.enable_detailed_layout_errors = false);
    assert!(
        !format!("{:?}", invalid.generate_pdf_bytes().unwrap_err())
            .contains("fixture overflow label")
    );
    Settings::configure(|s| s.enable_detailed_layout_errors = true);
    assert!(
        format!("{:?}", invalid.generate_pdf_bytes().unwrap_err())
            .contains("fixture overflow label")
    );
    let image = std::fs::read("verification/artifacts/compression-source.png")?;
    let dynamic = Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.content().height(60.).image_dynamic(move |payload| {
                assert_eq!(payload.dpi, 150.);
                Some(image.clone())
            });
        })
    });
    std::fs::write(
        output.join("dynamic.png"),
        &dynamic.generate_images(150.)?[0],
    )?;
    std::fs::copy(
        "verification/artifacts/compression-source.png",
        questpdf::runtime::application_files_directory().join("relative-resource.png"),
    )?;
    Document::create(|d|d.page(|p| {
        p.size(Size::new(240.,180.));p.margin(10.);
        p.content().column(|c| {
            c.item().width(80.).image_file("relative-resource.png").unwrap();
            c.item().svg(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="80" height="40"><image width="80" height="40" xlink:href="relative-resource.png"/></svg>"#);
        });
    })).generate_pdf(output.join("relative-resources.pdf"))?;
    println!(
        "License, evaluation producer, automatic discovery, caching, missing fonts, detailed errors and raster callback DPI passed"
    );
    Ok(())
}
