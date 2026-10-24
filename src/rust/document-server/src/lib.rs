//! Rust implementation of the RIS native document library.
pub mod documents;
pub mod ffi;
pub mod models;
pub mod printer;
mod resample;
pub mod util;
pub fn compose(
    report_type: &str,
    payload: serde_json::Value,
) -> anyhow::Result<questpdf::Document> {
    let payload = util::normalize_keys(payload);
    Ok(match report_type {
        "receipt" | "debt-receipt" => documents::invoice_receipt(&serde_json::from_value(payload)?),
        "large-receipt" => documents::large_receipt(&serde_json::from_value(payload)?),
        "debt" | "debt-report" => documents::debt_report(&serde_json::from_value(payload)?),
        "debt-out" | "debt-out-report" => {
            documents::debt_out_report(&serde_json::from_value(payload)?)
        }
        "substitute" | "substitute-report" => {
            documents::substitute_report(&serde_json::from_value(payload)?)
        }
        "income" | "income-report" => documents::income_report(&serde_json::from_value(payload)?),
        _ => anyhow::bail!("Unsupported report type '{report_type}'."),
    })
}
pub fn generate_pdf(report_type: &str, payload: serde_json::Value) -> anyhow::Result<Vec<u8>> {
    documents::render_document(&compose(report_type, payload)?)
}
