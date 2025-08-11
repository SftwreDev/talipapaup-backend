use chrono::{DateTime, Utc};
use num_format::{Locale, ToFormattedString};

pub fn format_money(amount: f64) -> String {
    let is_negative = amount < 0.0;
    let abs_amount = amount.abs();
    let whole_part = abs_amount.trunc() as u64;
    let decimal_part = (abs_amount.fract() * 100.0).round() as u64;

    let formatted = format!(
        "{}.{:02}",
        whole_part.to_formatted_string(&Locale::en),
        decimal_part
    );

    if is_negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}
pub fn format_datetime<T: Into<DateTime<Utc>>>(datetime: T) -> String {
    datetime.into().format("%Y-%m-%d %I:%M:%S %p").to_string()
}
