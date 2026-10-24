fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let cases: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    std::fs::create_dir_all(&args[2])?;
    for case in cases.as_array().unwrap() {
        let pdf = ris_document_reports::generate_pdf(
            case["kind"].as_str().unwrap(),
            case["payload"].clone(),
        )?;
        std::fs::write(
            std::path::Path::new(&args[2]).join(format!("{}.pdf", case["name"].as_str().unwrap())),
            pdf,
        )?;
    }
    Ok(())
}
