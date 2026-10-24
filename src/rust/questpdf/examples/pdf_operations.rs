use questpdf::*;
use std::path::PathBuf;
fn main() -> anyhow::Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or("output/operations".into()),
    );
    std::fs::create_dir_all(&output)?;
    let base = output.join("base.pdf");
    Document::create(|container| {
        for number in 1..=3 {
            container.page(|page| {
                page.size(Size::new(220., 180.));
                page.margin(10.);
                page.content()
                    .text(format!("Source page {number}"))
                    .font_size(24.);
            })
        }
    })
    .generate_pdf(&base)?;
    let layer = output.join("layer.pdf");
    Document::create(|c| {
        c.page(|p| {
            p.size(Size::new(220., 180.));
            p.content().align_bottom().text("OVERLAY").font_size(20.);
        })
    })
    .generate_pdf(&layer)?;
    DocumentOperation::load_file(&base)?
        .take_pages("3,1")
        .save(output.join("selected.pdf"))?;
    DocumentOperation::load_file(&base)?
        .merge_file(&layer, None)?
        .linearize()
        .save(output.join("merged.pdf"))?;
    DocumentOperation::load_file(&base)?
        .overlay_file(LayerConfiguration {
            file_path: layer.clone(),
            repeat_source_pages: Some("1".into()),
            ..Default::default()
        })?
        .save(output.join("overlay.pdf"))?;
    DocumentOperation::load_file(&base)?
        .underlay_file(LayerConfiguration {
            file_path: layer.clone(),
            repeat_source_pages: Some("1".into()),
            ..Default::default()
        })?
        .save(output.join("underlay.pdf"))?;
    let encryption = Encryption {
        user_password: "fixture-read".into(),
        owner_password: "fixture-owner".into(),
        allow_printing: false,
        ..Default::default()
    };
    DocumentOperation::load_file(&base)?
        .encrypt(encryption)?
        .save(output.join("encrypted.pdf"))?;
    DocumentOperation::load_file(output.join("encrypted.pdf"))?
        .password("fixture-owner")
        .decrypt()
        .save(output.join("decrypted.pdf"))?;
    for bits in [40, 128, 256] {
        DocumentOperation::load_file(&base)?
            .encrypt(Encryption {
                bits,
                user_password: "fixture-read".into(),
                owner_password: "fixture-owner".into(),
                allow_annotation: false,
                allow_assembly: false,
                allow_content_extraction: false,
                allow_filling_forms: false,
                allow_modification: false,
                allow_printing: false,
                encrypt_metadata: false,
            })?
            .save(output.join(format!("encrypted-{bits}.pdf")))?;
    }
    let note = output.join("note.txt");
    std::fs::write(&note, "Rust attachment fixture")?;
    DocumentOperation::load_file(&base)?
        .add_attachment(DocumentAttachment {
            file_path: note,
            relationship: Some(DocumentAttachmentRelationship::Supplement),
            ..Default::default()
        })?
        .save(output.join("attached.pdf"))?;
    let mut archival = Document::create(|c| {
        c.page(|p| {
            p.content().text("Archival metadata fixture");
        })
    });
    archival.settings.pdfa_conformance = PdfAConformance::PdfA3B;
    archival.generate_pdf(output.join("archival.pdf"))?;
    DocumentOperation::load_file(output.join("archival.pdf"))?
        .extend_metadata("<rdf:Description xmlns:test=\"urn:questpdf-rust:test\"><test:value>Verified XMP extension</test:value></rdf:Description>")
        .save(output.join("extended.pdf"))?;
    Ok(())
}
