use std::fmt;

use anyhow::{Error, Result, format_err};
use serde::Deserialize;
use tokio::process::Command;

/// Agent-Friendly Documentation Spec url.
pub(crate) const SPEC_URL: &str = "https://agentdocsspec.com/spec/";

/// AFDocs report (list of check results).
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AfdocsReport {
    results: Vec<AfdocsCheckResult>,
}

/// AFDocs check result details.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct AfdocsCheckResult {
    pub id: String,
    pub category: String,
    pub status: AfdocsCheckStatus,
    pub message: String,
}

/// AFDocs check result status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AfdocsCheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
    Error,
    #[serde(other)]
    Other,
}

impl fmt::Display for AfdocsCheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
            Self::Other => "OTHER",
        };
        write!(f, "{status}")
    }
}

/// Run afdocs on the website provided and return its report. The afdocs
/// command exits with a non-zero code when any of its checks fail, so the
/// exit code is ignored as long as the output contains a valid report.
pub(crate) async fn afdocs(website_url: &str) -> Result<AfdocsReport> {
    let output = Command::new("afdocs")
        .arg("check")
        .arg(website_url)
        .arg("--format")
        .arg("json")
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: AfdocsReport = serde_json::from_str(stdout.as_ref()).map_err(|_| {
        format_err!(
            "unexpected afdocs output: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })?;
    Ok(report)
}

/// Get all check results in the afdocs report that belong to the category
/// the check provided represents.
pub(crate) fn get_category<'a>(
    afdocs: &'a Result<Option<AfdocsReport>>,
    check_id: &str,
) -> Result<Option<Vec<&'a AfdocsCheckResult>>, &'a Error> {
    let category = check_id.replace('_', "-");
    match afdocs {
        Ok(Some(report)) => {
            let results: Vec<&AfdocsCheckResult> = report
                .results
                .iter()
                .filter(|r| r.category == category)
                .collect();
            if results.is_empty() {
                return Ok(None);
            }
            Ok(Some(results))
        }
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn sample_results() -> Vec<AfdocsCheckResult> {
        vec![
            AfdocsCheckResult {
                id: "llms-txt-exists".to_string(),
                category: "content-discoverability".to_string(),
                status: AfdocsCheckStatus::Warn,
                message: "llms.txt found but only reachable via cross-host redirect".to_string(),
            },
            AfdocsCheckResult {
                id: "llms-txt-directive-html".to_string(),
                category: "content-discoverability".to_string(),
                status: AfdocsCheckStatus::Fail,
                message: "No llms.txt directive found in HTML of any of 1 pages".to_string(),
            },
            AfdocsCheckResult {
                id: "markdown-url-support".to_string(),
                category: "markdown-availability".to_string(),
                status: AfdocsCheckStatus::Pass,
                message: "Markdown URLs supported".to_string(),
            },
        ]
    }

    #[test]
    fn report_parsed_from_real_json_output() {
        let json = std::fs::read_to_string("src/testdata/afdocs/report.json").unwrap();
        let report: AfdocsReport = serde_json::from_str(&json).unwrap();

        assert_eq!(report.results.len(), 23);
        assert!(
            report
                .results
                .iter()
                .any(|r| r.category == "content-discoverability")
        );
    }

    #[test]
    fn get_category_found() {
        let afdocs = Ok(Some(AfdocsReport {
            results: sample_results(),
        }));

        let results = get_category(&afdocs, "content_discoverability")
            .unwrap()
            .unwrap();
        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .all(|r| r.category == "content-discoverability")
        );
    }

    #[test]
    fn get_category_not_found() {
        let afdocs = Ok(Some(AfdocsReport {
            results: sample_results(),
        }));

        assert!(get_category(&afdocs, "page_size").unwrap().is_none());
    }

    #[test]
    fn get_category_no_website() {
        let afdocs = Ok(None);

        assert!(
            get_category(&afdocs, "content_discoverability")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn get_category_datasource_error() {
        let afdocs = Err(format_err!("something failed"));

        assert!(get_category(&afdocs, "content_discoverability").is_err());
    }
}
