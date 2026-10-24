//! Compare the printer algorithm independently of PDF image generation.
fn main() -> anyhow::Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let args: Vec<_> = std::env::args().collect();
    let source = std::path::Path::new(&args[1]);
    let output = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(output)?;
    for name in ["receipt", "debt-receipt", "large-receipt"] {
        let png = std::fs::read(source.join(format!("{name}-1.png")))?;
        let bytes = ris_document_reports::printer::encode_image(&png)?;
        std::fs::write(output.join(format!("{name}-escpos.bin")), bytes)?;
    }
    Ok(())
}
