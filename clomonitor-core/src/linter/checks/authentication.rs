use anyhow::Result;

use crate::linter::{
    CheckSet,
    check::{CheckId, CheckInput, CheckOutput},
};

use super::datasource::afdocs;

/// Check identifier.
pub(crate) const ID: CheckId = "authentication";

/// Check score weight.
pub(crate) const WEIGHT: usize = 1;

/// Check sets this check belongs to.
pub(crate) const CHECK_SETS: [CheckSet; 2] = [CheckSet::Community, CheckSet::Docs];

/// Check main function.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn check(input: &CheckInput) -> Result<CheckOutput> {
    Ok(afdocs::get_category(&input.afdocs, ID).into())
}
