use serde::Deserialize;

#[derive(Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}
