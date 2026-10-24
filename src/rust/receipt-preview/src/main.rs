use anyhow::{Context, Result};
use std::path::PathBuf;
fn main() -> Result<()> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let args: Vec<_> = std::env::args().skip(1).collect();
    let option = |name: &str| {
        args.iter()
            .position(|a| a.eq_ignore_ascii_case(name))
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let has = |name: &str| args.iter().any(|a| a.eq_ignore_ascii_case(name));
    let output = PathBuf::from(option("--output").unwrap_or("output/rust".into()));
    std::fs::create_dir_all(&output)?;
    let fixtures = option("--fixtures").map(PathBuf::from).unwrap_or_else(|| {
        if let Some(directory) = questpdf::runtime::library_directory() {
            let path = directory.join("fixtures");
            if path.is_dir() {
                return path;
            }
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../verification/fixtures")
    });
    let selected = option("--report")
        .unwrap_or_else(|| {
            if has("--debt") {
                "debt-receipt".into()
            } else {
                "receipt".into()
            }
        })
        .to_ascii_lowercase();
    let reports = [
        "receipt",
        "debt-receipt",
        "large-receipt",
        "debt-report",
        "debt-out-report",
        "income-report",
        "substitute-report",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();
    let mut selected_document = None;
    for report in &reports {
        let payload = std::fs::read(fixtures.join(format!("{report}.json")))
            .with_context(|| format!("fixture for {report}"))?;
        let value = serde_json::from_slice(&payload)?;
        let doc = ris_document_reports::compose(report, value)?;
        let pdf = ris_document_reports::documents::render_document(&doc)?;
        std::fs::write(output.join(format!("{report}.pdf")), pdf)?;
        println!("Generated {report}");
        if has("--images") {
            let mut renderer = quest_renderer()?;
            for (i, image) in renderer.generate_images(&doc, 150.)?.iter().enumerate() {
                std::fs::write(output.join(format!("{report}-{}.png", i + 1)), image)?;
            }
        }
        if has("--escpos")
            && ["receipt", "debt-receipt", "large-receipt"].contains(&report.as_str())
        {
            std::fs::write(
                output.join(format!("{report}-escpos.bin")),
                ris_document_reports::printer::receipt_bytes(&doc)?,
            )?;
        }
        if *report == selected {
            selected_document = Some(doc)
        }
    }
    let document = selected_document
        .with_context(|| format!("Unknown report '{selected}'. Use: {}.", reports.join(", ")))?;
    if has("--preview") {
        let port = option("--companion-port")
            .map(|s| s.parse::<u16>())
            .transpose()?
            .unwrap_or(12500);
        quest_renderer()?.show_in_companion(&document, port)?;
    }
    if has("--viewer") {
        let path = output.join(format!("{selected}.pdf")).canonicalize()?;
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer.exe")
                .arg(path)
                .spawn()?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(path).spawn()?;
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            std::process::Command::new("xdg-open").arg(path).spawn()?;
        }
    }
    if has("--print") {
        anyhow::ensure!(
            ["receipt", "debt-receipt", "large-receipt"].contains(&selected.as_str()),
            "Only receipt previews can be printed."
        );
        let host = option("--printer").unwrap_or("192.168.1.100".into());
        let port = option("--printer-port")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(9100);
        let port = u16::try_from(port).context("Printer port must be between 0 and 65535")?;
        ris_document_reports::printer::print_receipt(&document, &host, port)?;
        println!("Sent {selected} preview to {host}:{port}.");
    }
    Ok(())
}
fn quest_renderer() -> Result<questpdf::Renderer> {
    let mut renderer = questpdf::Renderer::new()?;
    renderer
        .register_font_file(ris_document_reports::documents::assets().join("BAHNSCHRIFT.TTF"))?;
    Ok(renderer)
}
