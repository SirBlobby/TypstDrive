use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::Value;

use super::entries::check_rate;
use super::identifiers::strip_prefix_ignore_case;
use super::policy::load_policy;
use super::sources::{get_json, JsonRequest};
use super::ApiError;
use crate::accounts::signed_in::SignedInUser;
use crate::AppState;

#[derive(Serialize)]
pub struct Affiliation {
    pub organization: String,
    pub department: Option<String>,
    pub role: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub current: bool,
}

#[derive(Serialize)]
pub struct OrcidProfile {
    pub orcid: String,
    pub name: String,
    pub given_names: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub affiliations: Vec<Affiliation>,
}

fn invalid() -> ApiError {
    (StatusCode::BAD_REQUEST, "Enter an ORCID iD such as 0000-0002-1825-0097".to_string())
}

fn checksum_character(digits: &[u32]) -> char {
    let total = digits.iter().fold(0, |total, digit| (total + digit) * 2);
    let result = (12 - total % 11) % 11;
    if result == 10 {
        'X'
    } else {
        char::from_digit(result, 10).unwrap_or('0')
    }
}

pub fn parse_orcid(input: &str) -> Result<String, ApiError> {
    let mut text = input.trim();
    for prefix in ["https://", "http://", "www.", "orcid.org/", "orcid:"] {
        if let Some(rest) = strip_prefix_ignore_case(text, prefix) {
            text = rest.trim();
        }
    }
    let characters: Vec<char> = text.trim_end_matches('/').chars().filter(|c| *c != '-' && !c.is_whitespace()).collect();
    if characters.len() != 16 {
        return Err(invalid());
    }
    let digits: Vec<u32> = characters[..15].iter().map(|c| c.to_digit(10)).collect::<Option<_>>().ok_or_else(invalid)?;
    if characters[15].to_ascii_uppercase() != checksum_character(&digits) {
        return Err((StatusCode::BAD_REQUEST, "That ORCID iD has a typo. Check the last character.".to_string()));
    }
    let joined: String = characters.iter().map(|c| c.to_ascii_uppercase()).collect();
    Ok(format!("{}-{}-{}-{}", &joined[0..4], &joined[4..8], &joined[8..12], &joined[12..16]))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |current, key| current.get(key)).filter(|found| !found.is_null())
}

fn text_at(value: &Value, path: &[&str]) -> Option<String> {
    value_at(value, path).and_then(Value::as_str).map(str::trim).filter(|text| !text.is_empty()).map(str::to_string)
}

fn affiliation(summary: &Value) -> Option<Affiliation> {
    Some(Affiliation {
        organization: text_at(summary, &["organization", "name"])?,
        department: text_at(summary, &["department-name"]),
        role: text_at(summary, &["role-title"]),
        city: text_at(summary, &["organization", "address", "city"]),
        country: text_at(summary, &["organization", "address", "country"]),
        current: value_at(summary, &["end-date"]).is_none(),
    })
}

fn affiliations(record: &Value, section: &str, summary_key: &str) -> Vec<Affiliation> {
    let groups = value_at(record, &["activities-summary", section, "affiliation-group"]).and_then(Value::as_array);
    groups
        .into_iter()
        .flatten()
        .filter_map(|group| group.get("summaries").and_then(Value::as_array))
        .flatten()
        .filter_map(|summary| summary.get(summary_key).and_then(affiliation))
        .collect()
}

fn profile(orcid: String, record: &Value) -> Result<OrcidProfile, ApiError> {
    let given_names = text_at(record, &["person", "name", "given-names", "value"]);
    let family_name = text_at(record, &["person", "name", "family-name", "value"]);
    let credit_name = text_at(record, &["person", "name", "credit-name", "value"]);
    let full_name = [given_names.clone(), family_name.clone()].into_iter().flatten().collect::<Vec<_>>().join(" ");
    let name = credit_name
        .or_else(|| (!full_name.is_empty()).then_some(full_name))
        .ok_or((StatusCode::NOT_FOUND, "This ORCID record does not show a public name".to_string()))?;

    let email = value_at(record, &["person", "emails", "email"])
        .and_then(Value::as_array)
        .and_then(|emails| emails.iter().find_map(|entry| text_at(entry, &["email"])));

    let mut found = affiliations(record, "employments", "employment-summary");
    if found.is_empty() {
        found = affiliations(record, "educations", "education-summary");
    }
    found.sort_by_key(|entry| !entry.current);

    Ok(OrcidProfile { orcid, name, given_names, family_name, email, affiliations: found })
}

pub async fn lookup(
    State(state): State<AppState>,
    Path(input): Path<String>,
    SignedInUser(user_id): SignedInUser,
) -> Result<Json<OrcidProfile>, ApiError> {
    load_policy(&state).await.check_orcid()?;
    let orcid = parse_orcid(&input)?;
    check_rate(&user_id)?;
    let url = format!("https://pub.orcid.org/v3.0/{orcid}/record");
    let record = get_json(JsonRequest::new(url, "application/json", "ORCID"))
        .await?
        .ok_or((StatusCode::NOT_FOUND, "No ORCID record was found for that iD".to_string()))?;
    Ok(Json(profile(orcid, &record)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_orcid_checksums() {
        assert_eq!(parse_orcid("https://orcid.org/0000-0002-1825-0097").unwrap(), "0000-0002-1825-0097");
        assert_eq!(parse_orcid("0000-0002-1694-233x").unwrap(), "0000-0002-1694-233X");
        assert!(parse_orcid("0000-0002-1825-0098").is_err());
        assert!(parse_orcid("1234").is_err());
    }
}
