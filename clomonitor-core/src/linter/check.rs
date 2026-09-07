use anyhow::{Context, Error, Result, format_err};
use serde::{Deserialize, Serialize};
use which::which;

use super::{
    CheckSet, LinterInput,
    checks::{
        CHECKS, authentication, content_discoverability, content_structure, markdown_availability,
        observability, page_size, signed_releases, url_stability, util::helpers::should_skip_check,
    },
    datasource::{
        afdocs::{self, AfdocsCheckResult, AfdocsCheckStatus, AfdocsReport},
        github,
        scorecard::{Scorecard, ScorecardCheck, scorecard},
        security_insights::SecurityInsights,
    },
    metadata::{Exemption, METADATA_FILE, Metadata},
};

/// Type alias to represent a check identifier.
pub type CheckId = &'static str;

/// Check configuration.
pub(crate) struct CheckConfig {
    pub weight: usize,
    pub check_sets: Vec<CheckSet>,
    pub scorecard_name: Option<String>,
}

/// Input used by checks to perform their operations.
#[derive(Debug)]
pub(crate) struct CheckInput<'a> {
    pub li: &'a LinterInput,
    pub cm_md: Option<Metadata>,
    pub gh_md: github::md::MdRepository,
    pub scorecard: Result<Scorecard>,
    pub security_insights: Result<Option<SecurityInsights>>,
    pub afdocs: Result<Option<AfdocsReport>>,
}

impl CheckInput<'_> {
    pub(crate) async fn new(li: &LinterInput) -> Result<CheckInput<'_>> {
        // Check if required external tools are available
        if which("scorecard").is_err() {
            return Err(format_err!(
                "scorecard not found in PATH (https://github.com/ossf/scorecard#installation)"
            ));
        }
        if afdocs_required(&li.check_sets) && which("afdocs").is_err() {
            return Err(format_err!(
                "afdocs not found in PATH (https://www.npmjs.com/package/afdocs)"
            ));
        }

        // Get CLOMonitor metadata
        let cm_md = Metadata::from(li.root.join(METADATA_FILE))?;

        // The next both actions (get GitHub metadata and get scorecard) make use
        // of the GitHub token, which when used concurrently, may trigger some
        // GitHub secondary rate limits. So they should not be run concurrently.

        // Get GitHub metadata
        let gh_md = github::metadata(&li.url, &li.github_token).await?;

        // Get OpenSSF scorecard and afdocs report. The afdocs command does not
        // use the GitHub token, so running both concurrently cannot trigger the
        // GitHub secondary rate limits mentioned above. The afdocs report is
        // only fetched when any of the checks that rely on it will be run.
        let (scorecard, afdocs) = tokio::join!(
            async {
                scorecard(&li.url, &li.github_token)
                    .await
                    .context("error running scorecard command")
            },
            async {
                match &gh_md.homepage_url {
                    Some(url) if !url.is_empty() && afdocs_required(&li.check_sets) => {
                        afdocs::afdocs(url)
                            .await
                            .map(Some)
                            .context("error running afdocs command")
                    }
                    _ => Ok(None),
                }
            }
        );

        // Get OpenSSF security insights.
        let security_insights = SecurityInsights::new(&li.root);

        // Prepare and return check input
        let ci = CheckInput {
            li,
            cm_md,
            gh_md,
            scorecard,
            security_insights,
            afdocs,
        };
        Ok(ci)
    }
}

/// Check ids of the checks that rely on the afdocs report.
const AFDOCS_CHECKS: [CheckId; 7] = [
    authentication::ID,
    content_discoverability::ID,
    content_structure::ID,
    markdown_availability::ID,
    observability::ID,
    page_size::ID,
    url_stability::ID,
];

/// Check if any of the checks that rely on the afdocs report will be run for
/// the check sets provided.
fn afdocs_required(check_sets: &[CheckSet]) -> bool {
    AFDOCS_CHECKS
        .iter()
        .any(|check_id| !should_skip_check(check_id, check_sets))
}

/// Check output information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckOutput<T = ()> {
    pub passed: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    pub exempt: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemption_reason: Option<String>,

    pub failed: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_reason: Option<String>,
}

impl<T> CheckOutput<T> {
    /// Create a new CheckOutput instance with the passed field set to true.
    #[must_use]
    pub fn passed() -> Self {
        Self {
            passed: true,
            ..Default::default()
        }
    }

    /// Create a new CheckOutput instance with the passed field set to false.
    #[must_use]
    pub fn not_passed() -> Self {
        Self {
            passed: false,
            ..Default::default()
        }
    }

    /// Create a new CheckOutput instance with the exempt field set to true.
    #[must_use]
    pub fn exempt() -> Self {
        Self {
            exempt: true,
            ..Default::default()
        }
    }

    /// Create a new CheckOutput instance with the failed field set to true.
    #[must_use]
    pub fn failed() -> Self {
        Self {
            failed: true,
            ..Default::default()
        }
    }

    /// Url field setter.
    #[must_use]
    pub fn url(mut self, url: Option<String>) -> CheckOutput<T> {
        self.url = url;
        self
    }

    /// Value field setter.
    #[must_use]
    pub fn value(mut self, value: Option<T>) -> CheckOutput<T> {
        self.value = value;
        self
    }

    /// Details field setter.
    #[must_use]
    pub fn details(mut self, details: Option<String>) -> CheckOutput<T> {
        self.details = details;
        self
    }

    /// Exemption reason field setter.
    #[must_use]
    pub fn exemption_reason(mut self, reason: Option<String>) -> CheckOutput<T> {
        self.exemption_reason = reason;
        self
    }

    /// Fail reason field setter.
    #[must_use]
    pub fn fail_reason(mut self, reason: Option<String>) -> CheckOutput<T> {
        self.fail_reason = reason;
        self
    }
}

impl<T> Default for CheckOutput<T> {
    fn default() -> Self {
        Self {
            passed: false,
            url: None,
            value: None,
            details: None,
            exempt: false,
            exemption_reason: None,
            failed: false,
            fail_reason: None,
        }
    }
}

impl<T> From<Exemption> for CheckOutput<T> {
    fn from(exemption: Exemption) -> Self {
        Self::exempt().exemption_reason(Some(exemption.reason))
    }
}

impl<T> From<Result<Option<&ScorecardCheck>, &Error>> for CheckOutput<T> {
    fn from(sc_check: Result<Option<&ScorecardCheck>, &Error>) -> Self {
        match sc_check {
            Ok(sc_check) => match sc_check {
                Some(sc_check) => {
                    let signed_releases =
                        CHECKS[signed_releases::ID].scorecard_name.as_ref().unwrap();
                    let mut output = CheckOutput::default();
                    let pass_threshold = match &sc_check.name {
                        n if n == signed_releases => 1.0,
                        _ => 5.0,
                    };
                    if sc_check.score >= pass_threshold {
                        output.passed = true;
                    }
                    output.details = Some(format!(
                        r"# {} OpenSSF Scorecard check

**Score**: {} (check passes with score >= {})

**Reason**: {}

**Details**: {}

**Please see the [check documentation]({}) in the ossf/scorecard repository for more details**",
                        sc_check.name,
                        sc_check.score,
                        pass_threshold,
                        sc_check.reason,
                        match &sc_check.details {
                            Some(details) => format!("\n\n>{}", details.join("\n")),
                            None => "-".to_string(),
                        },
                        sc_check.documentation.url,
                    ));
                    output
                }
                None => CheckOutput::not_passed(),
            },
            Err(err) => CheckOutput::failed().fail_reason(Some(format!("{err:#}"))),
        }
    }
}

impl<T> From<Result<Option<Vec<&AfdocsCheckResult>>, &Error>> for CheckOutput<T> {
    fn from(results: Result<Option<Vec<&AfdocsCheckResult>>, &Error>) -> Self {
        match results {
            Ok(Some(results)) if !results.is_empty() => {
                // Prepare category name from the category of the first result
                let category_name = match results[0].category.as_str() {
                    "url-stability" => "URL stability".to_string(),
                    slug => {
                        let category = slug.replace('-', " ");
                        let mut chars = category.chars();
                        match chars.next() {
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                            None => String::new(),
                        }
                    }
                };

                // Prepare check output based on the results statuses
                let all_skipped = results.iter().all(|r| r.status == AfdocsCheckStatus::Skip);
                let some_failed = results.iter().any(|r| {
                    matches!(r.status, AfdocsCheckStatus::Fail | AfdocsCheckStatus::Error)
                });
                let (mut output, result) = if all_skipped {
                    (
                        CheckOutput::exempt().exemption_reason(Some(
                            "afdocs skipped all checks in this category for this website"
                                .to_string(),
                        )),
                        "skipped (afdocs skipped all checks in this category for this website)",
                    )
                } else if some_failed {
                    (
                        CheckOutput::not_passed(),
                        "not passed (a category passes when none of its checks fail)",
                    )
                } else {
                    (
                        CheckOutput::passed(),
                        "passed (a category passes when none of its checks fail)",
                    )
                };
                output.details = Some(format!(
                    r"# {} AFDocs checks

**Result**: {}

{}

**Please see the [Agent-Friendly Documentation Spec]({}) for more details**",
                    category_name,
                    result,
                    results
                        .iter()
                        .map(|r| format!("- **{}** `{}`: {}", r.status, r.id, r.message))
                        .collect::<Vec<String>>()
                        .join("\n"),
                    afdocs::SPEC_URL,
                ));
                output
            }
            Ok(_) => CheckOutput::failed().fail_reason(Some(
                "category not found in the afdocs report (possible afdocs version mismatch)"
                    .to_string(),
            )),
            Err(err) => CheckOutput::failed().fail_reason(Some(format!("{err:#}"))),
        }
    }
}

/// Wrapper macro that takes care of running some common pre-check operations
/// and the synchronous check function.
macro_rules! run {
    ($check:ident, $input:expr) => {
        (|| {
            // Check if this check should be skipped
            if should_skip_check($check::ID, &$input.li.check_sets) {
                return None;
            }

            // Check if an exemption has been declared for this check
            if let Some(exemption) = find_exemption($check::ID, $input.cm_md.as_ref()) {
                return Some(CheckOutput::from(exemption));
            }

            // Call sync check function and wrap returned check output in an option
            let output = match $check::check($input) {
                Ok(output) => output,
                Err(err) => CheckOutput::failed().fail_reason(Some(format!("{:#}", err))),
            };
            Some(output)
        })()
    };
}
pub(crate) use run;

/// Wrapper macro that takes care of running some common pre-check operations
/// and the asynchronous check function.
macro_rules! run_async {
    ($check:ident, $input:expr) => {
        async {
            // Check if this check should be skipped
            if should_skip_check($check::ID, &$input.li.check_sets) {
                return None;
            }

            // Check if an exemption has been declared for this check
            if let Some(exemption) = find_exemption($check::ID, $input.cm_md.as_ref()) {
                return Some(CheckOutput::from(exemption));
            }

            // Call async check function and wrap returned check output in an option
            let output = match $check::check($input).await {
                Ok(output) => output,
                Err(err) => CheckOutput::failed().fail_reason(Some(format!("{:#}", err))),
            };
            Some(output)
        }
    };
}
pub(crate) use run_async;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::datasource::scorecard::ScorecardCheckDocs;
    use anyhow::{Result, format_err};

    #[test]
    fn check_output_from_exemption() {
        let exemption = Exemption {
            check: "test".to_string(),
            reason: "test".to_string(),
        };

        assert_eq!(
            CheckOutput::<()>::from(exemption),
            CheckOutput {
                exempt: true,
                exemption_reason: Some("test".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_scorecard_check_passed() {
        let sc_check = ScorecardCheck {
            name: "Code-Review".to_string(),
            reason: "reason".to_string(),
            details: Some(vec!["details".to_string()]),
            score: 8.0,
            documentation: ScorecardCheckDocs {
                url: "https://test.url".to_string(),
            },
        };

        assert_eq!(
            CheckOutput::<()>::from(Ok(Some(&sc_check))),
            CheckOutput {
                passed: true,
                details: Some("# Code-Review OpenSSF Scorecard check\n\n**Score**: 8 (check passes with score >= 5)\n\n**Reason**: reason\n\n**Details**: \n\n>details\n\n**Please see the [check documentation](https://test.url) in the ossf/scorecard repository for more details**".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_scorecard_check_not_passed() {
        let sc_check = ScorecardCheck {
            name: "Code-Review".to_string(),
            reason: "reason".to_string(),
            details: Some(vec!["details".to_string()]),
            score: 4.0,
            documentation: ScorecardCheckDocs {
                url: "https://test.url".to_string(),
            },
        };

        assert_eq!(
            CheckOutput::<()>::from(Ok(Some(&sc_check))),
            CheckOutput {
                passed: false,
                details: Some("# Code-Review OpenSSF Scorecard check\n\n**Score**: 4 (check passes with score >= 5)\n\n**Reason**: reason\n\n**Details**: \n\n>details\n\n**Please see the [check documentation](https://test.url) in the ossf/scorecard repository for more details**".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_scorecard_check_not_available() {
        let sc_check: Result<Option<&ScorecardCheck>, &Error> = Ok(None);

        assert_eq!(
            CheckOutput::<()>::from(sc_check),
            CheckOutput {
                passed: false,
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_scorecard_check_failed() {
        let err = format_err!("fake error");
        let sc_check: Result<Option<&ScorecardCheck>, &Error> = Err(&err);

        assert_eq!(
            CheckOutput::<()>::from(sc_check),
            CheckOutput {
                failed: true,
                fail_reason: Some("fake error".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_passed() {
        let results = [
            AfdocsCheckResult {
                id: "markdown-url-support".to_string(),
                category: "markdown-availability".to_string(),
                status: AfdocsCheckStatus::Pass,
                message: "Markdown URLs supported".to_string(),
            },
            AfdocsCheckResult {
                id: "content-negotiation".to_string(),
                category: "markdown-availability".to_string(),
                status: AfdocsCheckStatus::Warn,
                message: "partial support".to_string(),
            },
        ];
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> =
            Ok(Some(results.iter().collect()));

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                passed: true,
                details: Some("# Markdown availability AFDocs checks\n\n**Result**: passed (a category passes when none of its checks fail)\n\n- **PASS** `markdown-url-support`: Markdown URLs supported\n- **WARN** `content-negotiation`: partial support\n\n**Please see the [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/) for more details**".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_not_passed() {
        let results = [
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
        ];
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> =
            Ok(Some(results.iter().collect()));

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                passed: false,
                details: Some("# Content discoverability AFDocs checks\n\n**Result**: not passed (a category passes when none of its checks fail)\n\n- **WARN** `llms-txt-exists`: llms.txt found but only reachable via cross-host redirect\n- **FAIL** `llms-txt-directive-html`: No llms.txt directive found in HTML of any of 1 pages\n\n**Please see the [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/) for more details**".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_error_not_passed() {
        let results = [AfdocsCheckResult {
            id: "http-status-codes".to_string(),
            category: "url-stability".to_string(),
            status: AfdocsCheckStatus::Error,
            message: "check errored".to_string(),
        }];
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> =
            Ok(Some(results.iter().collect()));

        let output = CheckOutput::<()>::from(results);
        assert!(!output.passed);
        assert!(!output.exempt);
        assert!(
            output
                .details
                .unwrap()
                .starts_with("# URL stability AFDocs checks")
        );
    }

    #[test]
    fn check_output_from_afdocs_category_all_skipped() {
        let results = [AfdocsCheckResult {
            id: "llms-txt-coverage".to_string(),
            category: "observability".to_string(),
            status: AfdocsCheckStatus::Skip,
            message: "llms.txt not found".to_string(),
        }];
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> =
            Ok(Some(results.iter().collect()));

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                exempt: true,
                exemption_reason: Some(
                    "afdocs skipped all checks in this category for this website".to_string()
                ),
                details: Some("# Observability AFDocs checks\n\n**Result**: skipped (afdocs skipped all checks in this category for this website)\n\n- **SKIP** `llms-txt-coverage`: llms.txt not found\n\n**Please see the [Agent-Friendly Documentation Spec](https://agentdocsspec.com/spec/) for more details**".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_not_available() {
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> = Ok(None);

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                failed: true,
                fail_reason: Some(
                    "category not found in the afdocs report (possible afdocs version mismatch)"
                        .to_string()
                ),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_empty_results() {
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> = Ok(Some(vec![]));

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                failed: true,
                fail_reason: Some(
                    "category not found in the afdocs report (possible afdocs version mismatch)"
                        .to_string()
                ),
                ..Default::default()
            }
        );
    }

    #[test]
    fn check_output_from_afdocs_category_failed() {
        let err = format_err!("fake error");
        let results: Result<Option<Vec<&AfdocsCheckResult>>, &Error> = Err(&err);

        assert_eq!(
            CheckOutput::<()>::from(results),
            CheckOutput {
                failed: true,
                fail_reason: Some("fake error".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn afdocs_required_matching_check_sets() {
        assert!(afdocs_required(&[CheckSet::Community]));
        assert!(afdocs_required(&[CheckSet::Docs]));
        assert!(afdocs_required(&[CheckSet::Code, CheckSet::Community]));
    }

    #[test]
    fn afdocs_required_no_matching_check_sets() {
        assert!(!afdocs_required(&[CheckSet::Code]));
        assert!(!afdocs_required(&[CheckSet::CodeLite]));
        assert!(!afdocs_required(&[CheckSet::Code, CheckSet::CodeLite]));
        assert!(!afdocs_required(&[]));
    }
}
