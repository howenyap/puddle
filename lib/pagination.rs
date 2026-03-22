use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub const MAX_PER_PAGE: u32 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PerPage(u32);

impl PerPage {
    pub const fn new_unchecked(per_page: u32) -> Self {
        Self(per_page)
    }

    pub fn new(per_page: u32) -> Result<Self, InvalidPerPage> {
        if per_page == 0 || per_page > MAX_PER_PAGE {
            return Err(InvalidPerPage(per_page));
        }

        Ok(Self(per_page))
    }

    pub const fn into_inner(self) -> u32 {
        self.0
    }
}

impl Display for PerPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PerPage {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let per_page = value
            .parse::<u32>()
            .map_err(|_| format!("invalid value '{value}' for '--per-page <PER_PAGE>'"))?;

        Self::new(per_page).map_err(|_| format!("per-page must be between 1 and {MAX_PER_PAGE}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPerPage(pub u32);

impl Display for InvalidPerPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "per-page must be between 1 and {MAX_PER_PAGE}")
    }
}

impl StdError for InvalidPerPage {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "perpage")]
    pub per_page: Option<PerPage>,
}

impl PageParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_page_accepts_values_within_documented_range() {
        assert_eq!(Ok(PerPage::new_unchecked(1)), PerPage::new(1));
        assert_eq!(Ok(PerPage::new_unchecked(50)), PerPage::new(50));
    }

    #[test]
    fn per_page_rejects_zero_and_values_above_documented_max() {
        assert_eq!(Err(InvalidPerPage(0)), PerPage::new(0));
        assert_eq!(Err(InvalidPerPage(51)), PerPage::new(51));
    }
}
