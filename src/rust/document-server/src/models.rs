// Stored properties ported from the RIS C# models.
use crate::util::Date;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisDebtReport {
    pub report_hash: String,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub clients: Vec<RisDebtClient>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisDebtClient {
    pub first_name: String,
    pub family_name: String,
    pub gender: String,
    pub phone_number: String,
    pub last_debt_at: Date,
    pub created_at: Date,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub debt_payment_info: Vec<DebtModel>,
    #[serde(with = "crate::util::decimal_json")]
    pub debt_amount: Decimal,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DebtModel {
    pub amount: f64,
    pub status: String,
    pub date: Date,
    pub created_by: Option<RisUser>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisUser {
    pub name: String,
    pub email: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisClient {
    pub id: String,
    pub first_name: String,
    pub family_name: String,
    pub phone_number: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisIncomeDebtPayment {
    pub amount: f64,
    pub status: String,
    pub paid_at: Date,
    pub paid_by: RisUser,
    pub client: RisClient,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisDebtOutReport {
    #[serde(deserialize_with = "crate::util::null_default")]
    pub transactions: Vec<RisDebtOutTransaction>,
    pub start_date: Date,
    pub end_date: Date,
    pub all_time: bool,
    pub as_of_date: Date,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisDebtOutTransaction {
    pub amount: f64,
    pub debt_at: Date,
    pub created_by: RisUser,
    pub client: RisClient,
    pub current_debt: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct StudyProduct {
    pub name: String,
    pub price: f64,
    pub quantity_p: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisStudyEnriched {
    pub id: i32,
    pub status_payment: String,
    pub exam_type: String,
    pub modality: String,
    pub discount: Option<f64>,
    pub price: f64,
    pub conv: String,
    pub conv_price: Option<f64>,
    pub paid_at: Date,
    pub paid_by: RisUser,
    pub client: RisClient,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub products: Vec<StudyProduct>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RisIncomeReport {
    #[serde(deserialize_with = "crate::util::null_default")]
    pub studies: Vec<RisStudyEnriched>,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub debt_payments: Vec<RisIncomeDebtPayment>,
    pub start_date: Date,
    pub end_date: Date,
    pub all_time: bool,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InvoiceReceiptModel {
    pub is_debt_receipt: bool,
    pub client_name: String,
    pub client_phone: String,
    pub paid_by: String,
    pub paid_at: Date,
    pub qr_code: String,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub exams: Vec<InvoiceReceiptExam>,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub products: Vec<InvoiceReceiptProduct>,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub transactions: Vec<InvoiceReceiptDebtTransaction>,
    pub total_price: f64,
    pub discount: f64,
    pub convention_discount: f64,
    pub paid_amount: f64,
    pub remaining_debt: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InvoiceReceiptExam {
    pub id: i32,
    pub created_at: Date,
    pub exam: String,
    pub price: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InvoiceReceiptProduct {
    pub name: String,
    pub price: f64,
    pub quantity: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct InvoiceReceiptDebtTransaction {
    pub date: Date,
    pub amount: f64,
    pub status: String,
    pub created_by: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LargeReceiptModel {
    pub printed_at: Date,
    pub client_name: String,
    pub client_phone: String,
    pub age: String,
    pub payment_status: String,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub studies: Vec<LargeReceiptStudy>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LargeReceiptStudy {
    pub id: i32,
    pub modality: String,
    pub exam_type: String,
    pub substitute_doctor: String,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SubstituteDoctorReportModel {
    pub doctor_name: String,
    pub month: i32,
    pub year: i32,
    pub substitute_percent: f64,
    pub created_by: String,
    pub report_date: Date,
    #[serde(deserialize_with = "crate::util::null_default")]
    pub studies: Vec<SubstituteDoctorStudy>,
    pub total_amount: f64,
    pub calculated_amount: f64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SubstituteDoctorStudy {
    pub id: i32,
    pub created_at: Date,
    pub exam_type: String,
    pub modality: String,
    pub client_name: String,
    pub price: f64,
    pub discount: f64,
    pub created_by_name: String,
    pub status_payment: String,
    pub paid_by_name: String,
}
pub const JSON_FIELDS: &[&str] = &[
    "age",
    "all_time",
    "amount",
    "as_of_date",
    "calculated_amount",
    "client",
    "client_name",
    "client_phone",
    "clients",
    "conv",
    "conv_price",
    "convention_discount",
    "created_at",
    "created_by",
    "created_by_name",
    "current_debt",
    "date",
    "debt_amount",
    "debt_at",
    "debt_payment_info",
    "debt_payments",
    "discount",
    "doctor_name",
    "email",
    "end_date",
    "exam",
    "exam_type",
    "exams",
    "family_name",
    "first_name",
    "gender",
    "id",
    "is_debt_receipt",
    "last_debt_at",
    "modality",
    "month",
    "name",
    "paid_amount",
    "paid_at",
    "paid_by",
    "paid_by_name",
    "payment_status",
    "phone_number",
    "price",
    "printed_at",
    "products",
    "qr_code",
    "quantity",
    "quantity_p",
    "remaining_debt",
    "report_date",
    "report_hash",
    "start_date",
    "status",
    "status_payment",
    "studies",
    "substitute_doctor",
    "substitute_percent",
    "total_amount",
    "total_price",
    "transactions",
    "year",
];
