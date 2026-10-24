//! The RIS templates keep the source composition order, units and typography.
use crate::{models::*, util::*};
use questpdf::*;
use std::path::{Path, PathBuf};
pub fn assets() -> PathBuf {
    if let Some(path) = std::env::var_os("DOC_REPORTS_ASSETS_PATH") {
        return path.into();
    }
    if let Some(directory) = questpdf::runtime::library_directory() {
        let path = directory.join("Assets");
        if path.join("BAHNSCHRIFT.TTF").is_file() {
            return path;
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}
pub fn render_document(document: &Document) -> anyhow::Result<Vec<u8>> {
    questpdf::Settings::set_license(questpdf::LicenseType::Community);
    let mut r = Renderer::new()?;
    r.register_font_file(assets().join("BAHNSCHRIFT.TTF"))?;
    r.generate(document)
}
pub fn color(v: u32) -> Color {
    Color(0xff000000 | v)
}
pub const GREY1: Color = Color(0xffbdbdbd);
pub const GREY2: Color = Color(0xffe0e0e0);
pub const GREY3: Color = Color(0xffeeeeee);
pub const DARK1: Color = Color(0xff757575);
pub const DARK2: Color = Color(0xff616161);
pub const DARK3: Color = Color(0xff424242);
pub const DARK4: Color = Color(0xff212121);
pub const BLUE2: Color = Color(0xff1976d2);
pub const BLUE4: Color = Color(0xff0d47a1);
pub const GREEN2: Color = Color(0xff388e3c);
pub const RED3: Color = Color(0xffc62828);
pub const RED4: Color = Color(0xffb71c1c);
pub const ORANGE3: Color = Color(0xffef6c00);
pub const PURPLE4: Color = Color(0xff4a148c);
fn metadata(doc: &mut Document, title: &str) {
    doc.metadata.title = title.into();
    doc.metadata.author = "50 LAB RIS".into()
}
fn receipt_cell(table: &mut TableDescriptor, background: Color) -> Container {
    table
        .cell()
        .border(0.5)
        .border_color(GREY1)
        .background(background)
        .padding_horizontal(2.)
        .padding_vertical(1.5)
}
fn receipt_header(table: &mut TableDescriptor, label: &str, right: bool) {
    let c = table
        .cell()
        .border(0.5)
        .border_color(Color::BLACK)
        .background(Color::BLACK)
        .padding_horizontal(2.)
        .padding_vertical(1.5);
    let c = if right { c.align_right() } else { c };
    c.text(label).font_color(Color::WHITE).bold();
}
fn detail_line(col: &mut ColumnDescriptor, label: &str, value: &str) {
    col.item().row(|row| {
        row.constant_item_unit(55., Unit::Millimetre)
            .text(label)
            .bold();
        row.relative_item().align_right().text(value).semi_bold();
    })
}
fn total_line(col: &mut ColumnDescriptor, label: &str, value: f64) {
    col.item().row(|row| {
        row.relative_item()
            .align_right()
            .text(label)
            .bold()
            .font_size(22.);
        row.constant_item_unit(45., Unit::Millimetre)
            .align_right()
            .text(amount(value))
            .bold()
            .font_size(22.);
    })
}
fn qr_svg(payload: &str, width: i32, height: i32) -> String {
    use qrcodegen::{QrCode, QrCodeEcc, QrSegment};
    let payload = if payload.trim().is_empty() {
        "https://50lab-ris.local"
    } else {
        payload
    };
    let segments = QrSegment::make_segments(payload);
    let qr = QrCode::encode_segments_advanced(
        &segments,
        QrCodeEcc::Low,
        qrcodegen::Version::MIN,
        qrcodegen::Version::MAX,
        None,
        false,
    )
    .expect("QR payload within capacity");
    let size = qr.size();
    let input = size + 8;
    let w = width.max(input);
    let h = height.max(input);
    let scale = (w / input).min(h / input);
    let left = (w - size * scale) / 2;
    let top = (h - size * scale) / 2;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.2\" baseProfile=\"tiny\" shape-rendering=\"crispEdges\" viewBox=\"0 0 {w} {h}\" viewport-fill=\"rgb(255,255,255)\" viewport-fill-opacity=\"1\" fill=\"rgb(0,0,0)\" fill-opacity=\"1\">"
    );
    // ZXing's SVG renderer merges vertical runs into maximal rectangles. Matching
    // that decomposition avoids antialiasing seams between adjacent QR modules.
    let black = |x: i32, y: i32| {
        x >= left
            && y >= top
            && x < left + size * scale
            && y < top + size * scale
            && qr.get_module((x - left) / scale, (y - top) / scale)
    };
    let mut processed = vec![false; (w * h) as usize];
    for x in 0..w {
        let mut start = None;
        for y in 0..=h {
            if y < h && processed[(y * w + x) as usize] {
                continue;
            }
            if y < h {
                processed[(y * w + x) as usize] = true
            }
            if y < h && black(x, y) {
                if start.is_none() {
                    start = Some(y)
                }
            } else if let Some(first) = start.take() {
                let mut end = x;
                for next in x + 1..w {
                    if (first..y).any(|yy| !black(next, yy)) {
                        break;
                    }
                    end = next;
                    for yy in first..y {
                        processed[(yy * w + next) as usize] = true
                    }
                }
                svg += &format!(
                    "<rect x=\"{x}\" y=\"{first}\" width=\"{}\" height=\"{}\"/>",
                    end - x + 1,
                    y - first
                );
            }
        }
    }
    svg += "</svg>";
    svg
}
pub fn invoice_receipt(model: &InvoiceReceiptModel) -> Document {
    let mut doc = Document::create(|container| {
        container.page(|page| {
            page.continuous_size(177.3, Unit::Millimetre);
            page.margin_unit(1.5, Unit::Millimetre);
            page.default_text_style(|s| s.font_family("Bahnschrift").font_size(18.));
            page.content().column(|column| {
                column.spacing(3.);
                let path = assets().join("receipt_header.png");
                if path.exists() {
                    column
                        .item()
                        .image_file(path)
                        .expect("receipt header asset")
                        .fit_width();
                } else {
                    column
                        .item()
                        .align_center()
                        .text("50 LAB RIS")
                        .bold()
                        .font_size(28.);
                }
                column
                    .item()
                    .align_center()
                    .text(if model.is_debt_receipt {
                        "REÇU DE PAIEMENT DE DETTE"
                    } else {
                        "REÇU DE PAIEMENT"
                    })
                    .bold()
                    .font_size(24.);
                column.item().padding_top(1.).row(|row| {
                    let payload = model.qr_code.clone();
                    row.constant_item_unit(34., Unit::Millimetre)
                        .aspect_ratio(1., AspectRatioOption::FitWidth)
                        .padding_right(3.)
                        .svg_dynamic(move |size| {
                            qr_svg(&payload, size.width as i32, size.height as i32)
                        });
                    row.relative_item().column(|details| {
                        details.spacing(2.);
                        detail_line(
                            details,
                            "Payé le:",
                            &model.paid_at.to_local().format("%d/%m/%Y %H:%M"),
                        );
                        detail_line(details, "Client:", &model.client_name);
                        if !model.client_phone.trim().is_empty() {
                            detail_line(details, "Téléphone:", &model.client_phone)
                        }
                        if !model.is_debt_receipt {
                            detail_line(details, "Montant total:", &amount(model.total_price));
                            if model.discount != 0. {
                                detail_line(details, "Remise totale:", &amount(model.discount))
                            }
                            if model.convention_discount != 0. {
                                detail_line(
                                    details,
                                    "Remise convention:",
                                    &amount(model.convention_discount),
                                )
                            }
                        }
                    });
                });
                if !model.is_debt_receipt && !model.exams.is_empty() {
                    column.item().padding_top(2.).table(|table| {
                        table.columns_definition(|c| {
                            c.relative_column_weight(3.);
                            c.relative_column_weight(5.);
                            c.relative_column_weight(2.);
                        });
                        table.header(|h| {
                            receipt_header(h, "Date d'examen", false);
                            receipt_header(h, "Examen", false);
                            receipt_header(h, "Tarif", true);
                        });
                        for (i, exam) in model.exams.iter().enumerate() {
                            let bg = if i % 2 == 0 { GREY3 } else { Color::WHITE };
                            receipt_cell(table, bg)
                                .text(exam.created_at.to_local().format("%d/%m/%Y %H:%M"));
                            receipt_cell(table, bg).text(&exam.exam);
                            receipt_cell(table, bg)
                                .align_right()
                                .text(amount(exam.price))
                                .bold();
                        }
                    });
                }
                if !model.products.is_empty() {
                    column.item().padding_top(2.).table(|table| {
                        table.columns_definition(|c| {
                            for w in [5., 2., 1., 2.] {
                                c.relative_column_weight(w)
                            }
                        });
                        table.header(|h| {
                            for (label, right) in [
                                ("Produit", false),
                                ("Prix", false),
                                ("Qté", false),
                                ("Sous-total", true),
                            ] {
                                receipt_header(h, label, right)
                            }
                        });
                        for (i, product) in model.products.iter().enumerate() {
                            let bg = if i % 2 == 0 { GREY3 } else { Color::WHITE };
                            receipt_cell(table, bg).text(&product.name);
                            receipt_cell(table, bg).text(amount(product.price));
                            receipt_cell(table, bg)
                                .text(format!("x {}", number(product.quantity, 0)));
                            receipt_cell(table, bg)
                                .align_right()
                                .text(amount(product.price * product.quantity))
                                .bold();
                        }
                    });
                }
                if !model.transactions.is_empty() {
                    column
                        .item()
                        .padding_top(2.)
                        .align_center()
                        .text("HISTORIQUE DE DETTES")
                        .bold()
                        .font_size(20.);
                    column.item().table(|table| {
                        table.columns_definition(|c| {
                            for w in [3., 4., 3.] {
                                c.relative_column_weight(w)
                            }
                        });
                        table.header(|h| {
                            for t in ["Date", "Montant", "Type"] {
                                receipt_header(h, t, false)
                            }
                        });
                        for (i, t) in model.transactions.iter().enumerate() {
                            let bg = if i % 2 == 0 { GREY3 } else { Color::WHITE };
                            receipt_cell(table, bg)
                                .text(t.date.to_local().format("%d/%m/%Y %H:%M"));
                            receipt_cell(table, bg).text(amount(t.amount));
                            receipt_cell(table, bg).text(match t.status.to_lowercase().as_str() {
                                "in" => "DETTE PAYÉE",
                                "discount" => "REMISE DE DETTE",
                                _ => "DETTE SORTIE",
                            });
                        }
                    });
                }
                column
                    .item()
                    .padding_top(2.)
                    .align_right()
                    .column(|totals| {
                        total_line(
                            totals,
                            if model.is_debt_receipt {
                                "Paiement de dette:"
                            } else {
                                "Montant à payer:"
                            },
                            if model.is_debt_receipt {
                                model.paid_amount
                            } else {
                                model.total_price - model.discount - model.convention_discount
                            },
                        );
                        total_line(totals, "Payée:", model.paid_amount);
                        if model.remaining_debt != 0. {
                            total_line(totals, "Reste à payer:", model.remaining_debt)
                        }
                    });
                column
                    .item()
                    .padding_top(3.)
                    .line_horizontal(0.5)
                    .line_color(DARK1);
                column.item().row(|row| {
                    row.relative_item().text_spans(|text| {
                        text.span("Paiement émis par: ").bold();
                        text.span(&model.paid_by);
                    });
                    row.relative_item()
                        .align_right()
                        .text(now().format("%d/%m/%Y %H:%M:%S"));
                });
            });
        })
    });
    metadata(
        &mut doc,
        if model.is_debt_receipt {
            "Reçu de paiement de dette"
        } else {
            "Reçu de paiement"
        },
    );
    doc
}
const DASHED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 3" preserveAspectRatio="none"><line x1="0" y1="1.5" x2="100" y2="1.5" stroke="#000000" stroke-width="0.8" stroke-dasharray="3 2" /></svg>"##;
const PHONE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000000" d="M6.62 10.79c1.44 2.83 3.76 5.14 6.59 6.59l2.2-2.2c.27-.27.67-.36 1.02-.24 1.12.37 2.33.57 3.57.57.55 0 1 .45 1 1V20c0 .55-.45 1-1 1C10.61 21 3 13.39 3 4c0-.55.45-1 1-1h3.5c.55 0 1 .45 1 1 0 1.25.2 2.45.57 3.57.11.35.03.74-.25 1.02l-2.2 2.2z" /></svg>"##;
fn payment_icon(paid: bool) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="13.5" fill="none" stroke="#000000" stroke-width="2" />{}</svg>"##,
        if paid {
            r##"<path d="M9.5 16.5l4.2 4.2 8.9-9.2" fill="none" stroke="#000000" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" />"##
        } else {
            r##"<path d="M11 11l10 10M21 11L11 21" fill="none" stroke="#000000" stroke-width="2.6" stroke-linecap="round" />"##
        }
    )
}
pub fn large_receipt(model: &LargeReceiptModel) -> Document {
    let mut doc = Document::create(|container| {
        container.page(|page| {
            page.continuous_size(97.5, Unit::Millimetre);
            page.margin_horizontal(Unit::Millimetre.points(3.));
            page.margin_vertical(Unit::Millimetre.points(2.));
            page.default_text_style(|s| s.font_family("Bahnschrift").font_color(Color::BLACK));
            page.content().column(|column| {
                column.spacing(0.);
                column.item().height_unit(2., Unit::Millimetre).svg(DASHED);
                let name = model.client_name.trim().to_uppercase();
                let length = name.chars().count();
                let size = if length <= 18 {
                    31.
                } else if length <= 26 {
                    28.
                } else {
                    25.
                };
                let height = if length <= 26 { 12. } else { 20. };
                column
                    .item()
                    .padding_top(12.)
                    .padding_bottom(8.)
                    .height_unit(height, Unit::Millimetre)
                    .scale_to_fit()
                    .align_middle()
                    .align_center()
                    .text(name)
                    .bold()
                    .font_size(size)
                    .line_height(0.95);
                let parts: Vec<_> = model.age.trim().splitn(2, ' ').collect();
                let age = parts
                    .first()
                    .filter(|s| !s.is_empty())
                    .copied()
                    .unwrap_or("—");
                let unit = parts
                    .get(1)
                    .map(|s| s.trim().to_uppercase())
                    .unwrap_or_default();
                let unit = match unit.as_str() {
                    "A" | "AN" | "ANS" => "ANS",
                    "M" | "MOIS" => "MOIS",
                    "J" | "JOUR" | "JOURS" => "JOURS",
                    x => x,
                };
                column.item().padding_bottom(14.).row(|row| {
                    row.relative_item_weight(0.72)
                        .align_middle()
                        .text_spans(|t| {
                            t.span(age).bold().font_size(28.);
                            if !unit.is_empty() {
                                t.span(format!(" {unit}")).semi_bold().font_size(17.);
                            }
                        });
                    if !model.client_phone.trim().is_empty() {
                        row.relative_item_weight(1.48).align_middle().row(|phone| {
                            phone
                                .constant_item_unit(7., Unit::Millimetre)
                                .align_middle()
                                .svg(PHONE);
                            phone
                                .relative_item()
                                .padding_left(1.5)
                                .height_unit(9., Unit::Millimetre)
                                .scale_to_fit()
                                .align_middle()
                                .text(model.client_phone.trim())
                                .semi_bold()
                                .font_size(19.)
                                .line_height(1.);
                        });
                    }
                });
                for (index, study) in model.studies.iter().enumerate() {
                    let label = format!("{} {}", study.modality.trim(), study.exam_type.trim())
                        .trim()
                        .to_uppercase();
                    column
                        .item()
                        .padding_top(if index == 0 { 0. } else { 11. })
                        .align_center()
                        .text(label)
                        .bold()
                        .font_size(20.5)
                        .line_height(0.95);
                    let id = study.id.to_string();
                    let (prefix, suffix) = if id.len() > 3 {
                        id.split_at(id.len() - 3)
                    } else {
                        ("", id.as_str())
                    };
                    column
                        .item()
                        .padding_top(7.)
                        .align_center()
                        .text_spans(|t| {
                            t.span("EXAMEN N°  ").semi_bold().font_size(17.);
                            t.span(prefix).semi_bold().font_color(DARK1).font_size(29.);
                            t.span(suffix).bold().font_size(33.);
                        });
                    if !study.substitute_doctor.trim().is_empty() {
                        column
                            .item()
                            .padding_top(7.)
                            .align_center()
                            .text_spans(|t| {
                                t.span("MÉDECIN REMPLAÇANT : ")
                                    .semi_bold()
                                    .font_color(DARK1)
                                    .font_size(11.5);
                                t.span(study.substitute_doctor.trim().to_uppercase())
                                    .bold()
                                    .font_size(15.5);
                            });
                    }
                }
                let paid = model.payment_status.trim().eq_ignore_ascii_case("paid");
                let status = match model.payment_status.trim().to_lowercase().as_str() {
                    "paid" => "PAYÉ",
                    "debt" | "notpaid" => "NON PAYÉ",
                    _ => "INCONNU",
                };
                column
                    .item()
                    .padding_top(16.)
                    .padding_bottom(12.)
                    .row(|row| {
                        row.relative_item_weight(0.85)
                            .align_middle()
                            .row(|status_row| {
                                status_row
                                    .constant_item_unit(11., Unit::Millimetre)
                                    .align_middle()
                                    .svg(payment_icon(paid));
                                status_row
                                    .relative_item()
                                    .padding_left(4.)
                                    .align_middle()
                                    .text(status)
                                    .bold()
                                    .font_size(18.);
                            });
                        row.relative_item_weight(1.4)
                            .align_middle()
                            .align_right()
                            .text_spans(|t| {
                                t.span(model.printed_at.to_local().format("%Y-%m-%d"))
                                    .semi_bold()
                                    .font_color(DARK1)
                                    .font_size(13.);
                                t.span("  ");
                                t.span(model.printed_at.to_local().format("%H:%M:%S"))
                                    .bold()
                                    .font_size(16.5);
                            });
                    });
                column.item().height_unit(2., Unit::Millimetre).svg(DASHED);
            });
        })
    });
    metadata(&mut doc, "Ticket patient");
    doc
}

fn report_page(page: &mut Page, rtl: bool) {
    page.default_text_style(|s| s.font_family("Bahnschrift"));
    page.margin_horizontal(10.);
    page.margin_vertical(5.);
    page.margin_bottom(20.);
    if rtl {
        page.content_from_right_to_left()
    }
}
fn logo_row(column: &mut ColumnDescriptor) {
    column.item().padding_top(-30.).row(|row| {
        row.relative_item_weight(2.)
            .padding_top(35.)
            .svg_file(assets().join("logo.svg"))
            .expect("logo asset")
            .fit_area();
        row.relative_item_weight(8.)
            .svg_file(assets().join("header.svg"))
            .expect("header asset")
            .fit_area();
        row.relative_item_weight(2.)
            .padding_top(35.)
            .svg_file(assets().join("logo.svg"))
            .expect("logo asset")
            .fit_area();
    });
}
fn page_footer(page: &mut Page) {
    page.footer()
        .align_center()
        .padding_top(3.)
        .text_spans(|t| {
            t.span("Page ").font_family("Bahnschrift");
            t.current_page_number().bold().font_family("Bahnschrift");
            t.span(" de ").font_family("Bahnschrift");
            t.total_pages().bold().font_family("Bahnschrift");
        });
}
fn cell(table: &mut TableDescriptor, index: usize) -> Container {
    table
        .cell()
        .background(if index.is_multiple_of(2) {
            GREY2
        } else {
            Color::WHITE
        })
        .border_color(Color::BLACK)
        .border(1.)
        .padding_vertical(2.)
        .padding_horizontal(4.)
}
fn entire_cell(table: &mut TableDescriptor, index: usize) -> Container {
    table
        .cell()
        .show_entire()
        .background(if index.is_multiple_of(2) {
            GREY2
        } else {
            Color::WHITE
        })
        .border_color(Color::BLACK)
        .border(1.)
        .padding_vertical(2.)
        .padding_horizontal(4.)
}
fn header_cell(table: &mut TableDescriptor, label: &str, size: f32) {
    table
        .cell()
        .border_color(Color::BLACK)
        .border(1.)
        .background(Color::BLACK)
        .padding_vertical(2.)
        .padding_horizontal(4.)
        .text(label)
        .align_center()
        .font_size(size)
        .font_color(Color::WHITE);
}
fn white_cell(table: &mut TableDescriptor, label: &str, size: f32) {
    table
        .cell()
        .border_color(Color::BLACK)
        .border(1.)
        .padding_vertical(2.)
        .padding_horizontal(4.)
        .align_center()
        .text(label)
        .semi_bold()
        .font_size(size)
        .align_center();
}
fn name(family: &str, first: &str) -> String {
    format!("{family} {first}").trim().into()
}
const MONTHS: [&str; 12] = [
    "JANV.", "FÉVR.", "MARS", "AVR.", "MAI", "JUIN", "JUIL.", "AOÛT", "SEPT.", "OCT.", "NOV.",
    "DÉC.",
];
const WEEKDAYS: [&str; 7] = ["DIM.", "LUN.", "MAR.", "MER.", "JEU.", "VEN.", "SAM."];
fn period(start: Date, end: Date, all: bool, income: bool) -> String {
    use chrono::Datelike;
    if all {
        return "Toutes les périodes".into();
    }
    let f = |d: Date| {
        format!(
            "{} {} {}",
            d.format("%d"),
            MONTHS[d.0.month0() as usize],
            d.format("%Y")
        )
    };
    if start.0.date() == end.0.date() {
        if income {
            let days = [
                "DIMANCHE", "LUNDI", "MARDI", "MERCREDI", "JEUDI", "VENDREDI", "SAMEDI",
            ];
            format!(
                "{} {}",
                days[start.0.weekday().num_days_from_sunday() as usize],
                f(start)
            )
        } else {
            f(start)
        }
    } else {
        format!("{} - {}", f(start), f(end))
    }
}
fn french_timestamp(value: Date) -> String {
    use chrono::Datelike;
    format!(
        "{} {} {} {} | {}",
        WEEKDAYS[value.0.weekday().num_days_from_sunday() as usize],
        value.format("%d"),
        MONTHS[value.0.month0() as usize],
        value.format("%Y"),
        value.format("%H:%M")
    )
}
pub fn debt_report(info: &RisDebtReport) -> Document {
    Document::create(|container| {
        container.page(|page| {
            report_page(page, true);
            page.header()
                .show_once()
                .padding_vertical(10.)
                .column(|column| {
                    logo_row(column);
                    column
                        .item()
                        .padding_top(-20.)
                        .content_from_left_to_right()
                        .row(|row| {
                            row.spacing(20.);
                            row.relative_item()
                                .text("RAPPORT DES DETTES")
                                .font_size(14.)
                                .bold();
                            row.relative_item()
                                .align_right()
                                .text(now().format("%A, %d %B %Y | %H:%M"))
                                .font_size(14.)
                                .semi_bold();
                        });
                    column
                        .item()
                        .padding_top(5.)
                        .content_from_left_to_right()
                        .table(|table| {
                            table.columns_definition(|c| {
                                c.relative_column();
                                c.relative_column()
                            });
                            table.header(|h| {
                                header_cell(h, "Dette total", 12.);
                                header_cell(h, "Nombre de clients", 12.);
                            });
                            white_cell(
                                table,
                                &format!(
                                    "{} D.A",
                                    decimal_number(
                                        info.clients.iter().map(|c| c.debt_amount).sum(),
                                        2
                                    )
                                ),
                                14.,
                            );
                            white_cell(
                                table,
                                &format!("{} clients", number(info.clients.len() as f64, 0)),
                                14.,
                            );
                        });
                });
            page.content().column(|body| {
                body.item().content_from_left_to_right().table(|table| {
                    table.columns_definition(|c| {
                        for w in [1., 5., 3., 3., 4., 3.] {
                            c.relative_column_weight(w)
                        }
                    });
                    table.header(|h| {
                        for title in [
                            "N°",
                            "Client",
                            "N° de Tél",
                            "Dette total",
                            "Dernière dette le",
                            "Créé par",
                        ] {
                            header_cell(h, title, 8.)
                        }
                    });
                    for (index, client) in info.clients.iter().enumerate() {
                        cell(table, index)
                            .align_center()
                            .text(index.to_string())
                            .semi_bold()
                            .font_size(10.)
                            .align_center();
                        cell(table, index)
                            .align_left()
                            .text(name(&client.family_name, &client.first_name))
                            .semi_bold()
                            .font_size(10.)
                            .align_center();
                        cell(table, index)
                            .text(&client.phone_number)
                            .font_color(BLUE4)
                            .semi_bold()
                            .font_size(9.)
                            .align_center();
                        cell(table, index)
                            .align_right()
                            .text(format!("{} D.A", decimal_number(client.debt_amount, 2)))
                            .bold()
                            .font_size(10.)
                            .align_center();
                        cell(table, index)
                            .text(client.last_debt_at.format("%d %B %Y"))
                            .font_size(10.)
                            .align_center();
                        let creator = client
                            .debt_payment_info
                            .iter()
                            .max_by_key(|x| x.date)
                            .and_then(|x| x.created_by.as_ref())
                            .map(|u| u.name.as_str())
                            .unwrap_or("");
                        cell(table, index)
                            .text(creator)
                            .font_color(DARK2)
                            .font_size(10.)
                            .align_center();
                    }
                });
            });
            page_footer(page);
        })
    })
}
fn display_user(user: &RisUser) -> &str {
    if user.name.trim().is_empty() {
        "Utilisateur non renseigné"
    } else {
        &user.name
    }
}
pub fn debt_out_report(info: &RisDebtOutReport) -> Document {
    let mut rows: Vec<_> = info.transactions.iter().collect();
    rows.sort_by_key(|x| std::cmp::Reverse(x.debt_at));
    Document::create(|container| {
        container.page(|page|{
  report_page(page,false);
  page.header().show_once().padding_vertical(10.).column(|column|{
   logo_row(column);column.item().padding_top(-20.).row(|row|{row.relative_item().text("RAPPORT DES DETTES SORTIES").font_size(14.).bold();row.relative_item().align_right().text(format!("Généré le {}",info.as_of_date.format("%d/%m/%Y %H:%M"))).font_size(11.).semi_bold();});
   column.item().padding_top(5.).table(|table|{table.columns_definition(|c|{c.relative_column();c.relative_column()});header_cell(table,"Période",10.);header_cell(table,"Dette sortie totale",10.);white_cell(table,&period(info.start_date,info.end_date,info.all_time,false),10.);white_cell(table,&format!("{} D.A",number(rows.iter().map(|x|x.amount).sum(),2)),10.);});
  });
  page.content().column(|body|{
   body.item().show_once().padding_vertical(8.).background(color(0xffe0b2)).border(1.).border_color(color(0xf57c00)).padding(6.).text_spans(|t|{t.span("Note : ").semi_bold().font_size(8.).font_color(DARK4);t.span(format!("la colonne « Dette totale restante du client au {} » indique le solde actuel du client. Une dette sortie figurant dans ce rapport peut donc avoir été réglée ultérieurement.",info.as_of_date.format("%d/%m/%Y"))).font_size(8.).font_color(DARK4);});
   if rows.is_empty(){body.item().padding_top(20.).align_center().text("Aucune dette sortie ne correspond aux filtres sélectionnés.").font_size(12.).italic().font_color(DARK2);return}
   let mut days=std::collections::BTreeMap::new();for row in &rows{days.entry(row.debt_at.0.date()).or_insert_with(Vec::new).push(*row)}
   for (day,day_rows) in days.iter().rev(){
    body.item().padding_horizontal(200.).padding_vertical(3.).background(BLUE2).corner_radius(4.).padding_vertical(4.).text(day.format("%d/%m/%Y").to_string()).extra_black().font_size(14.).font_color(Color::WHITE).align_center();
    let mut users=std::collections::BTreeMap::new();for row in day_rows{users.entry(display_user(&row.created_by)).or_insert_with(Vec::new).push(*row)}
    for (user,transactions) in users{
     body.item().padding_vertical(4.).background(GREEN2).corner_radius(4.).padding_vertical(4.).text(user).extra_black().font_size(14.).font_color(Color::WHITE).align_center();
     body.item().table(|table|{
      table.columns_definition(|c|for w in [1.2,2.8,3.8,2.8,2.5,3.,4.]{c.relative_column_weight(w)});
      table.header(|h|{for title in ["N°".into(),"Date & Heure".into(),"Client".into(),"N° de téléphone".into(),"Dette sortie".into(),"Dette créée par".into(),format!("Dette totale restante du client au {}",info.as_of_date.format("%d/%m/%Y"))]{header_cell(h,&title,7.)}});
      for (i,t) in transactions.iter().enumerate(){let index=i+1;
       entire_cell(table,index).align_center().text(number(index as f64,0)).semi_bold().font_size(8.);
       entire_cell(table,index).align_center().text_spans(|text|{text.span(t.debt_at.format("%d/%m/%Y")).semi_bold().font_size(8.);text.span(format!(" {}",t.debt_at.format("%H:%M"))).font_size(8.);});
       entire_cell(table,index).text(name(&t.client.family_name,&t.client.first_name)).semi_bold().font_size(8.).align_left();
       entire_cell(table,index).text(&t.client.phone_number).font_color(BLUE4).font_size(8.).align_center();
       entire_cell(table,index).text(format!("{} D.A",number(t.amount,2))).bold().font_color(ORANGE3).font_size(8.).align_right();
       entire_cell(table,index).text(display_user(&t.created_by)).font_color(DARK2).font_size(8.).align_center();
       entire_cell(table,index).text(format!("{} D.A",number(t.current_debt,2))).bold().font_color(if t.current_debt>0.{RED3}else{GREEN2}).font_size(8.).align_right();
      }
     });
    }
   }
  });page_footer(page);
 })
    })
}
pub fn substitute_report(model: &SubstituteDoctorReportModel) -> Document {
    Document::create(|container| {
        container.page(|page| {
            report_page(page, false);
            page.size(PageSizes::A4);
            page.header()
                .show_once()
                .padding_vertical(5.)
                .column(|column| {
                    column
                        .item()
                        .text("RAPPORT MENSUEL - MÉDECIN REMPLAÇANT")
                        .font_size(14.)
                        .bold()
                        .align_center();
                    column.item().padding_top(5.).row(|row| {
                        row.spacing(10.);
                        row.relative_item().column(|col| {
                            col.item().row(|r| {
                                r.constant_item(150.)
                                    .text("Médecin Remplaçant: ")
                                    .bold()
                                    .font_size(9.);
                                r.auto_item().text(&model.doctor_name).font_size(9.);
                            });
                            col.item().row(|r| {
                                r.constant_item(150.)
                                    .text("Date du rapport: ")
                                    .bold()
                                    .font_size(9.);
                                r.auto_item()
                                    .text(model.report_date.format("%d/%m/%Y"))
                                    .font_size(9.);
                            });
                        });
                        row.relative_item().column(|col| {
                            col.item().row(|r| {
                                r.constant_item(150.)
                                    .text("Mois/Année: ")
                                    .bold()
                                    .font_size(9.);
                                r.auto_item()
                                    .text(format!("{:02}/{}", model.month, model.year))
                                    .font_size(9.);
                            });
                        });
                    });
                });
            page.content().padding_top(5.).column(|body| {
                body.item().table(|table| {
                    table.columns_definition(|c| {
                        for w in [3., 1.5, 4., 3., 2., 2.5, 2., 2.] {
                            c.relative_column_weight(w)
                        }
                    });
                    table.header(|h| {
                        for title in [
                            "Date de l'étude",
                            "ID",
                            "Type d'examen",
                            "Nom du client",
                            "Prix",
                            "Créé par",
                            "État de paiement",
                            "Payé par",
                        ] {
                            header_cell(h, title, 8.)
                        }
                    });
                    for (index, study) in model.studies.iter().enumerate() {
                        cell(table, index)
                            .text(study.created_at.format("%d/%m/%Y %H:%M"))
                            .font_size(7.)
                            .align_center();
                        cell(table, index)
                            .text(study.id.to_string())
                            .font_size(7.)
                            .align_center();
                        cell(table, index)
                            .text(format!("{} {}", study.modality, study.exam_type).trim())
                            .font_size(7.)
                            .align_start();
                        cell(table, index)
                            .text(&study.client_name)
                            .font_size(7.)
                            .align_start();
                        cell(table, index)
                            .align_right()
                            .text(format!("{} DZD", number(study.price - study.discount, 2)))
                            .font_size(7.)
                            .align_center();
                        cell(table, index)
                            .text(&study.created_by_name)
                            .font_size(7.)
                            .align_center();
                        let status = match study.status_payment.as_str() {
                            "paid" => "Payé",
                            "debt" => "Dette",
                            "notPaid" => "Non Payé",
                            s => s,
                        };
                        cell(table, index).text(status).font_size(7.).align_center();
                        cell(table, index)
                            .text(&study.paid_by_name)
                            .font_size(7.)
                            .align_center();
                    }
                });
                body.item().padding_top(20.).column(|column| {
                    column.item().padding_top(5.).align_right().column(|col| {
                        for (label, value) in [
                            ("Total:", format!("{} DZD", number(model.total_amount, 2))),
                            (
                                "Pourcentage:",
                                format!("{}%", number(model.substitute_percent, 2)),
                            ),
                            (
                                "Montant:",
                                format!("{} DZD", number(model.calculated_amount, 2)),
                            ),
                        ] {
                            col.item().row(|r| {
                                r.constant_item(70.).text(label).bold().font_size(9.);
                                r.constant_item(90.).align_right().text(value).font_size(9.);
                            });
                        }
                    });
                });
            });
            page_footer(page);
        })
    })
}

#[derive(Clone, Copy)]
enum IncomeRow<'a> {
    Study(&'a RisStudyEnriched),
    Payment(&'a RisIncomeDebtPayment),
}
impl<'a> IncomeRow<'a> {
    fn paid_at(self) -> Date {
        match self {
            Self::Study(s) => s.paid_at,
            Self::Payment(p) => p.paid_at,
        }
    }
    fn paid_by(self) -> &'a RisUser {
        match self {
            Self::Study(s) => &s.paid_by,
            Self::Payment(p) => &p.paid_by,
        }
    }
    fn client(self) -> &'a RisClient {
        match self {
            Self::Study(s) => &s.client,
            Self::Payment(p) => &p.client,
        }
    }
    fn is_debt(self) -> bool {
        matches!(self,Self::Study(s) if s.status_payment.eq_ignore_ascii_case("debt"))
    }
    fn is_paid(self) -> bool {
        matches!(self, Self::Study(_)) && !self.is_debt()
    }
    fn cash(self) -> f64 {
        match self {
            Self::Study(s) if self.is_paid() => s.price,
            Self::Payment(p) if p.status.eq_ignore_ascii_case("in") => p.amount,
            _ => 0.,
        }
    }
}
fn order_income<'a>(rows: &[IncomeRow<'a>]) -> Vec<IncomeRow<'a>> {
    let mut studies: Vec<_> = rows
        .iter()
        .copied()
        .filter(|r| matches!(r, IncomeRow::Study(_)))
        .collect();
    studies.sort_by_key(|r| std::cmp::Reverse(r.paid_at()));
    let mut payments = std::collections::BTreeMap::<&str, Vec<_>>::new();
    for row in rows
        .iter()
        .copied()
        .filter(|r| matches!(r, IncomeRow::Payment(_)))
    {
        payments.entry(&row.client().id).or_default().push(row)
    }
    for p in payments.values_mut() {
        p.sort_by_key(|r| std::cmp::Reverse(r.paid_at()))
    }
    let mut ordered = Vec::new();
    for study in studies {
        ordered.push(study);
        if study.is_debt()
            && !study.client().id.trim().is_empty()
            && let Some(p) = payments.remove(study.client().id.as_str())
        {
            ordered.extend(p)
        }
    }
    let mut remaining: Vec<_> = payments.into_values().flatten().collect();
    remaining.sort_by_key(|r| std::cmp::Reverse(r.paid_at()));
    ordered.extend(remaining);
    ordered
}
fn income_paid_at(table: &mut TableDescriptor, date: Date, index: usize, color: Color) {
    entire_cell(table, index).align_right().column(|col| {
        col.item()
            .text(date.format("%d/%m/%Y"))
            .semi_bold()
            .font_size(9.)
            .font_color(color)
            .align_end();
        col.item()
            .text(date.format("%H:%M"))
            .font_size(9.)
            .font_color(color)
            .align_end();
    });
}
fn income_study(table: &mut TableDescriptor, study: &RisStudyEnriched, index: usize) {
    entire_cell(table, index)
        .align_center()
        .text(number(study.id as f64, 0).replace(',', " "))
        .semi_bold()
        .font_size(10.);
    entire_cell(table, index)
        .align_left()
        .text(name(&study.client.family_name, &study.client.first_name))
        .semi_bold()
        .font_size(10.);
    entire_cell(table, index).column(|col| {
        col.item()
            .text(&study.modality)
            .semi_bold()
            .font_size(9.)
            .align_left();
        col.item().text(&study.exam_type).font_size(9.).align_left();
        for p in &study.products {
            col.item()
                .text(format!("{}x {}", p.quantity_p, p.name))
                .font_size(9.)
                .align_left()
                .font_color(DARK2);
        }
    });
    entire_cell(table, index).align_right().column(|col| {
        if study.status_payment.eq_ignore_ascii_case("debt") {
            col.item()
                .text("DETTE")
                .bold()
                .font_size(10.)
                .font_color(RED3)
                .align_right();
            return;
        }
        let pad =
            study.discount.unwrap_or(0.) > 0. || (!study.conv.is_empty() && study.conv != "normal");
        col.item()
            .padding_bottom(if pad { 2. } else { 0. })
            .text(format!("{} D.A", number(study.price, 2)))
            .bold()
            .font_size(10.)
            .align_right();
        if study.discount.unwrap_or(0.) > 0. {
            col.item()
                .text("Remise:")
                .bold()
                .font_size(9.)
                .font_color(RED4)
                .align_right();
            col.item()
                .text(format!("{} D.A", number(study.discount.unwrap(), 2)))
                .bold()
                .font_size(10.)
                .font_color(RED4)
                .align_right();
        }
        if study.conv != "normal" && !study.conv.is_empty() {
            col.item()
                .text(format!("{}:", study.conv.to_uppercase()))
                .bold()
                .font_size(9.)
                .font_color(PURPLE4)
                .align_right();
            col.item()
                .text(format!(
                    "{} D.A",
                    study.conv_price.map(|v| number(v, 2)).unwrap_or_default()
                ))
                .bold()
                .font_size(10.)
                .font_color(PURPLE4)
                .align_right();
        }
    });
    income_paid_at(table, study.paid_at, index, Color::BLACK);
    entire_cell(table, index)
        .text(&study.paid_by.name)
        .font_color(DARK2)
        .font_size(10.)
        .align_center();
}
fn income_payment(table: &mut TableDescriptor, p: &RisIncomeDebtPayment, index: usize) {
    let (label, color) = match p.status.to_lowercase().as_str() {
        "discount" => ("Remise de dette", color(0x5b2c83)),
        "out" => ("Dette sortie", color(0xa64500)),
        _ => ("Paiement de dette", color(0x1b7d31)),
    };
    entire_cell(table, index)
        .text("")
        .font_color(color)
        .font_size(10.)
        .align_center();
    entire_cell(table, index)
        .text(name(&p.client.family_name, &p.client.first_name))
        .semi_bold()
        .font_color(color)
        .font_size(10.)
        .align_left();
    entire_cell(table, index)
        .text(label)
        .semi_bold()
        .font_color(color)
        .font_size(10.)
        .align_left();
    entire_cell(table, index)
        .text(format!("{} D.A", number(p.amount, 2)))
        .bold()
        .font_color(color)
        .font_size(10.)
        .align_right();
    income_paid_at(table, p.paid_at, index, color);
    entire_cell(table, index)
        .text(&p.paid_by.name)
        .semi_bold()
        .font_color(color)
        .font_size(10.)
        .align_center();
}
pub fn income_report(info: &RisIncomeReport) -> Document {
    use chrono::Datelike;
    let all_rows: Vec<_> = info
        .studies
        .iter()
        .map(IncomeRow::Study)
        .chain(info.debt_payments.iter().map(IncomeRow::Payment))
        .collect();
    let paid = all_rows.iter().filter(|r| r.is_paid()).count();
    let payments = info
        .debt_payments
        .iter()
        .filter(|p| p.status.eq_ignore_ascii_case("in"))
        .count();
    let mut details = format!(
        "{} {}",
        number(paid as f64, 0),
        if paid == 1 { "examen" } else { "examens" }
    );
    if payments > 0 {
        details += &format!(
            " + {} {}",
            number(payments as f64, 0),
            if payments == 1 {
                "paiement de dette"
            } else {
                "paiements de dette"
            }
        )
    }
    let total_details = format!(
        "{} D.A ({details})",
        number(all_rows.iter().map(|r| r.cash()).sum(), 2)
    );
    Document::create(|container| {
        container.page(|page|{
  report_page(page,true);
  page.header().show_once().padding_vertical(10.).column(|column|{
   logo_row(column);column.item().padding_top(-20.).content_from_left_to_right().row(|row|{row.spacing(20.);row.relative_item_weight(2.).text("RAPPORT DES ENCAISSEMENTS PAR UTILISATEUR ET PAR JOUR").font_size(11.).bold();row.relative_item().align_right().text(french_timestamp(now())).font_size(11.).semi_bold();});
   column.item().padding_top(5.).content_from_left_to_right().corner_radius_top_left(4.).corner_radius_top_right(4.).table(|table|{
    table.columns_definition(|c|{c.relative_column_weight(0.85);c.relative_column_weight(1.15)});table.header(|h|{header_cell(h,"Période du rapport",12.);header_cell(h,"Montant total encaissé",12.);});white_cell(table,&period(info.start_date,info.end_date,info.all_time,true),14.);white_cell(table,&total_details,12.);
   });
  });
  page.content().content_from_left_to_right().column(|body|{
   let mut days=std::collections::BTreeMap::new();for row in &all_rows{days.entry(row.paid_at().0.date()).or_insert_with(Vec::new).push(*row)}
   for (date,day_rows) in days.iter().rev(){let day=format!("{} {} {}",date.day(),MONTHS[date.month0() as usize],date.year());
    body.item().padding_horizontal(200.).row(|r|{r.relative_item_weight(0.5).align_middle().padding_vertical(2.).background(BLUE2).corner_radius(4.).padding_vertical(4.).text(&day).extra_black().font_size(16.).font_color(Color::WHITE).align_center();});
    let mut users=std::collections::BTreeMap::new();for row in day_rows{users.entry(row.paid_by().name.as_str()).or_insert_with(Vec::new).push(*row)}
    for(user,user_rows)in users{let rows=order_income(&user_rows);
     body.item().row(|r|{r.relative_item_weight(0.5).align_middle().padding_vertical(4.).background(GREEN2).corner_radius(4.).padding_vertical(4.).text(user).extra_black().font_size(16.).font_color(Color::WHITE).align_center();});
     body.item().content_from_left_to_right().corner_radius_top_left(4.).corner_radius_top_right(4.).table(|table|{table.columns_definition(|c|for w in [2.,4.,4.,3.,2.,3.]{c.relative_column_weight(w)});table.header(|h|for title in ["N°","Client","Examen avec Produits","Montant Payé","Payé le","Payé par"]{header_cell(h,title,8.)});
      for (index,row) in rows.iter().enumerate(){match row{IncomeRow::Study(s)=>income_study(table,s,index),IncomeRow::Payment(p)=>income_payment(table,p,index)}}
     });
     let summary=[("Date",day.clone()),("Utilisateur",user.to_string()),("Examens payés",number(rows.iter().filter(|r|r.is_paid()).count() as f64,0)),("Montant total encaissé",format!("{} D.A",number(rows.iter().map(|r|r.cash()).sum(),0)))];
     body.item().show_entire().padding_vertical(6.).column(|col|{
      for(label,value)in summary{col.item().align_right().row(|r|{
       r.constant_item(130.).border(1.).border_color(Color::BLACK).padding_vertical(2.).padding_horizontal(4.).text(label).font_size(9.).align_right();
       r.constant_item(110.).border(1.).border_color(Color::BLACK).background(DARK3).padding_vertical(2.).padding_horizontal(4.).text(value).font_color(Color::WHITE).semi_bold().font_size(9.).align_right();
      });}
      col.item().padding_top(3.).align_right().width(240.).text("Note : les dettes sorties, remises de dette et examens à dette ne sont pas inclus dans le montant total. Seuls les paiements de dette sont inclus.").font_size(7.).font_color(DARK2).italic().align_right();
     });
    }
   }
  });page_footer(page);
 })
    })
}

macro_rules! document_type {
    ($name:ident,$model:ty,$compose:ident) => {
        pub struct $name(pub $model);
        impl IDocument for $name {
            fn compose(&self, container: &mut DocumentContainer) {
                container.pages.extend($compose(&self.0).container.pages)
            }
            fn metadata(&self) -> DocumentMetadata {
                $compose(&self.0).metadata
            }
        }
    };
}
document_type!(InvoiceReceiptDocument, InvoiceReceiptModel, invoice_receipt);
document_type!(LargeReceiptDocument, LargeReceiptModel, large_receipt);
document_type!(DebtDocument, RisDebtReport, debt_report);
document_type!(DebtOutReportByUsers, RisDebtOutReport, debt_out_report);
document_type!(IncomeReportByUsers, RisIncomeReport, income_report);
document_type!(
    SubstituteDoctorMonthlyReport,
    SubstituteDoctorReportModel,
    substitute_report
);
