use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateKind {
    Unspecified,
    Utc,
    Local,
}
#[derive(Clone, Copy, Debug)]
pub struct Date(pub NaiveDateTime, pub DateKind);
// DateTime comparison uses its clock ticks, without comparing Kind.
impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Date {}
impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl Default for Date {
    fn default() -> Self {
        Self(
            NaiveDate::from_ymd_opt(1, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            DateKind::Unspecified,
        )
    }
}
impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = self.0.format("%Y-%m-%dT%H:%M:%S").to_string();
        let fraction = format!("{:07}", self.0.nanosecond() / 100);
        let fraction = fraction.trim_end_matches('0');
        if !fraction.is_empty() {
            value.push('.');
            value.push_str(fraction);
        }
        match self.1 {
            DateKind::Utc => value.push('Z'),
            DateKind::Local => {
                if let Some(date) = Local.from_local_datetime(&self.0).latest() {
                    value.push_str(&date.format("%:z").to_string());
                }
            }
            DateKind::Unspecified => (),
        }
        serializer.serialize_str(&value)
    }
}
impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let valid = |date: NaiveDateTime| -> Result<NaiveDateTime, D::Error> {
            if !(1..=9999).contains(&date.year()) || date.nanosecond() >= 1_000_000_000 {
                return Err(serde::de::Error::custom(
                    "DateTime is outside the .NET supported range",
                ));
            }
            Ok(date)
        };
        let round = |d: NaiveDateTime| d.with_nanosecond(d.nanosecond() / 100 * 100).unwrap();
        if let Ok(d) = DateTime::parse_from_rfc3339(&s) {
            // System.Text.Json retains UTC for Z, converts numeric offsets to Local,
            // and leaves values without an offset Unspecified.
            if s.ends_with('Z') {
                return Ok(Self(round(valid(d.naive_utc())?), DateKind::Utc));
            }
            return Ok(Self(
                round(valid(d.with_timezone(&Local).naive_local())?),
                DateKind::Local,
            ));
        }
        for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"] {
            if let Ok(d) = NaiveDateTime::parse_from_str(&s, fmt) {
                return Ok(Self(round(valid(d)?), DateKind::Unspecified));
            }
        }
        if let Ok(d) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            return Ok(Self(d.and_hms_opt(0, 0, 0).unwrap(), DateKind::Unspecified));
        }
        Err(serde::de::Error::custom(format!("Invalid DateTime '{s}'")))
    }
}
impl Date {
    pub fn to_local(self) -> Self {
        if self.1 == DateKind::Local {
            return self;
        }
        Self(
            self.0.and_utc().with_timezone(&Local).naive_local(),
            DateKind::Local,
        )
    }
    pub fn format(self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }
}
pub fn now() -> Date {
    if let Ok(s) = std::env::var("QUESTPDF_TEST_NOW") {
        let mut date: Date =
            serde_json::from_value(serde_json::Value::String(s)).expect("valid QUESTPDF_TEST_NOW");
        date.1 = DateKind::Local;
        date
    } else {
        Date(Local::now().naive_local(), DateKind::Local)
    }
}
pub fn null_default<'de, D: Deserializer<'de>, T: Deserialize<'de> + Default>(
    d: D,
) -> Result<T, D::Error> {
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}
pub fn number(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    group_number(&s)
}
pub fn decimal_number(v: rust_decimal::Decimal, places: usize) -> String {
    // Decimal.ToString("N") rounds ties away from zero, unlike binary floating-point N formatting.
    let rounded = v.round_dp_with_strategy(
        places as u32,
        rust_decimal::RoundingStrategy::MidpointAwayFromZero,
    );
    group_number(&format!("{rounded:.places$}"))
}
pub mod decimal_json {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        rust_decimal::serde::arbitrary_precision::serialize(value, serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        // System.Text.Json's default decimal contract accepts JSON numbers only.
        let number = serde_json::Number::deserialize(deserializer)?;
        rust_decimal::serde::arbitrary_precision::deserialize(serde_json::Value::Number(number))
            .map_err(serde::de::Error::custom)
    }
}
fn group_number(s: &str) -> String {
    let mut parts = s.split('.');
    let integer = parts.next().unwrap();
    let (sign, digits) = if let Some(s) = integer.strip_prefix('-') {
        ("-", s)
    } else {
        ("", integer)
    };
    let mut out = String::from(sign);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',')
        }
        out.push(ch)
    }
    if let Some(f) = parts.next() {
        out.push('.');
        out.push_str(f)
    }
    out
}
pub fn amount(v: f64) -> String {
    format!("{} DA", number(v, 2))
}
pub fn normalize_keys(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Array(v) => Value::Array(v.into_iter().map(normalize_keys).collect()),
        Value::Object(m) => Value::Object(
            m.into_iter()
                .map(|(k, v)| {
                    let compact = k.replace('_', "").to_ascii_lowercase();
                    let key = crate::models::JSON_FIELDS
                        .iter()
                        .find(|f| f.replace('_', "") == compact)
                        .map(|f| (*f).to_string())
                        .unwrap_or(k);
                    (key, normalize_keys(v))
                })
                .collect(),
        ),
        x => x,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn debt_decimal_retains_precision_and_dotnet_formatting() {
        let client: crate::models::RisDebtClient =
            serde_json::from_str(r#"{"debt_amount":9007199254740993.125}"#).unwrap();
        assert_eq!(
            decimal_number(client.debt_amount, 2),
            "9,007,199,254,740,993.13"
        );
        assert_eq!(decimal_number("-2.345".parse().unwrap(), 2), "-2.35");
        let sum: rust_decimal::Decimal = ["0.1", "0.2"]
            .into_iter()
            .map(|v| v.parse::<rust_decimal::Decimal>().unwrap())
            .sum();
        assert_eq!(sum.to_string(), "0.3");
        assert!(
            serde_json::from_str::<crate::models::RisDebtClient>(r#"{"debt_amount":"1.23"}"#)
                .is_err()
        );
    }
}
