use axum::http::StatusCode;

use super::ApiError;

pub const ARXIV_DOI_PREFIX: &str = "10.48550/arXiv.";

#[derive(Debug, PartialEq, Eq)]
pub enum Identifier {
    Doi(String),
    Arxiv(String),
    Pubmed(String),
    Pmc(String),
    SemanticScholar(String),
}

const INVALID_MESSAGE: &str = "Enter a DOI (10.1000/xyz123), arXiv ID (2301.00001), PubMed ID (PMID 31452104), PMC ID (PMC6611412) or Semantic Scholar link";

const DOI_PREFIXES: &[&str] = &["doi.org/", "dx.doi.org/", "doi:"];
const ARXIV_PREFIXES: &[&str] = &["arxiv.org/abs/", "arxiv.org/pdf/", "export.arxiv.org/abs/", "arxiv:"];
const PUBMED_PREFIXES: &[&str] = &["pubmed.ncbi.nlm.nih.gov/", "ncbi.nlm.nih.gov/pubmed/", "pmid:", "pmid"];
const PMC_PREFIXES: &[&str] = &["pmc.ncbi.nlm.nih.gov/articles/", "ncbi.nlm.nih.gov/pmc/articles/", "europepmc.org/article/pmc/", "pmcid:"];
const SEMANTIC_SCHOLAR_PREFIXES: &[&str] = &["semanticscholar.org/paper/", "api.semanticscholar.org/", "s2:"];

fn invalid() -> ApiError {
    (StatusCode::BAD_REQUEST, INVALID_MESSAGE.to_string())
}

pub fn strip_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then(|| &text[prefix.len()..])
}

fn strip_any_prefix<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes
        .iter()
        .find_map(|prefix| strip_prefix_ignore_case(text, prefix))
        .map(|rest| rest.trim_start_matches([':', ' ']).trim())
}

fn without_web_prefix(input: &str) -> &str {
    let mut text = input.trim();
    for prefix in ["https://", "http://"] {
        if let Some(rest) = strip_prefix_ignore_case(text, prefix) {
            text = rest;
        }
    }
    strip_prefix_ignore_case(text, "www.").unwrap_or(text)
}

fn first_path_segment(text: &str) -> &str {
    text.split(['/', '?', '#']).next().unwrap_or_default()
}

fn without_version(id: &str) -> &str {
    match id.rfind('v') {
        Some(position) if position > 0 && position + 1 < id.len() && id[position + 1..].chars().all(|c| c.is_ascii_digit()) => &id[..position],
        _ => id,
    }
}

fn is_new_arxiv_id(id: &str) -> bool {
    let Some((prefix, number)) = id.split_once('.') else {
        return false;
    };
    prefix.len() == 4 && prefix.chars().all(|c| c.is_ascii_digit()) && (4..=5).contains(&number.len()) && number.chars().all(|c| c.is_ascii_digit())
}

fn is_old_arxiv_id(id: &str) -> bool {
    let Some((archive, number)) = id.split_once('/') else {
        return false;
    };
    !archive.is_empty()
        && archive.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c == '.')
        && number.len() == 7
        && number.chars().all(|c| c.is_ascii_digit())
}

fn arxiv_id(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim_end_matches(".pdf").trim_end_matches('/');
    let id = without_version(trimmed);
    (is_new_arxiv_id(id) || is_old_arxiv_id(id)).then(|| id.to_string())
}

fn parse_doi(text: &str) -> Option<String> {
    let decoded = text.replace("%2F", "/").replace("%2f", "/");
    let doi = decoded.trim().trim_end_matches(['.', ',', ';']);
    let (prefix, suffix) = doi.split_once('/')?;
    let valid_prefix = prefix.starts_with("10.") && prefix[3..].split('.').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    let valid_suffix = !suffix.is_empty() && !suffix.chars().any(|c| c.is_whitespace() || c.is_control());
    (valid_prefix && valid_suffix).then(|| doi.to_string())
}

fn doi_identifier(doi: String) -> Identifier {
    match strip_prefix_ignore_case(&doi, ARXIV_DOI_PREFIX).and_then(arxiv_id) {
        Some(id) => Identifier::Arxiv(id),
        None => Identifier::Doi(doi),
    }
}

fn pubmed_id(text: &str) -> Option<String> {
    let id = first_path_segment(text);
    (!id.is_empty() && id.len() <= 9 && id.chars().all(|c| c.is_ascii_digit())).then(|| id.to_string())
}

fn pmc_id(text: &str) -> Option<String> {
    let segment = first_path_segment(text);
    let digits = strip_prefix_ignore_case(segment, "pmc")?;
    (!digits.is_empty() && digits.len() <= 9 && digits.chars().all(|c| c.is_ascii_digit())).then(|| digits.to_string())
}

fn is_semantic_scholar_hash(text: &str) -> bool {
    text.len() == 40 && text.chars().all(|c| c.is_ascii_hexdigit())
}

fn semantic_scholar_id(text: &str) -> Option<String> {
    if let Some(number) = strip_any_prefix(text, &["corpusid:", "corpus id"]) {
        let number = first_path_segment(number);
        return (!number.is_empty() && number.chars().all(|c| c.is_ascii_digit())).then(|| format!("CorpusId:{number}"));
    }
    text.split(['/', '?', '#'])
        .find(|segment| is_semantic_scholar_hash(segment))
        .map(|hash| hash.to_ascii_lowercase())
}

pub fn parse_identifier(input: &str) -> Result<Identifier, ApiError> {
    let text = without_web_prefix(input);

    if let Some(rest) = strip_any_prefix(text, DOI_PREFIXES) {
        return parse_doi(rest).map(doi_identifier).ok_or_else(invalid);
    }
    if let Some(rest) = strip_any_prefix(text, ARXIV_PREFIXES) {
        return arxiv_id(rest).map(Identifier::Arxiv).ok_or_else(invalid);
    }
    if let Some(rest) = strip_any_prefix(text, PMC_PREFIXES) {
        return pmc_id(rest).map(Identifier::Pmc).ok_or_else(invalid);
    }
    if let Some(rest) = strip_any_prefix(text, PUBMED_PREFIXES) {
        return pubmed_id(rest).map(Identifier::Pubmed).ok_or_else(invalid);
    }
    if let Some(rest) = strip_any_prefix(text, SEMANTIC_SCHOLAR_PREFIXES) {
        return semantic_scholar_id(rest).map(Identifier::SemanticScholar).ok_or_else(invalid);
    }

    if let Some(doi) = parse_doi(text) {
        return Ok(doi_identifier(doi));
    }
    if let Some(id) = pmc_id(text) {
        return Ok(Identifier::Pmc(id));
    }
    if let Some(id) = pubmed_id(text).filter(|_| !text.contains('/')) {
        return Ok(Identifier::Pubmed(id));
    }
    if let Some(id) = semantic_scholar_id(text).filter(|_| !text.contains('/')) {
        return Ok(Identifier::SemanticScholar(id));
    }
    arxiv_id(text).map(Identifier::Arxiv).ok_or_else(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_kind_of_identifier() {
        assert_eq!(parse_identifier("https://doi.org/10.1038/nature14539").unwrap(), Identifier::Doi("10.1038/nature14539".into()));
        assert_eq!(parse_identifier("arXiv:1706.03762v5").unwrap(), Identifier::Arxiv("1706.03762".into()));
        assert_eq!(parse_identifier("10.48550/arXiv.1706.03762").unwrap(), Identifier::Arxiv("1706.03762".into()));
        assert_eq!(parse_identifier("PMID: 31452104").unwrap(), Identifier::Pubmed("31452104".into()));
        assert_eq!(parse_identifier("https://pubmed.ncbi.nlm.nih.gov/31452104/").unwrap(), Identifier::Pubmed("31452104".into()));
        assert_eq!(parse_identifier("31452104").unwrap(), Identifier::Pubmed("31452104".into()));
        assert_eq!(parse_identifier("PMC6611412").unwrap(), Identifier::Pmc("6611412".into()));
        assert_eq!(parse_identifier("https://pmc.ncbi.nlm.nih.gov/articles/PMC6611412/").unwrap(), Identifier::Pmc("6611412".into()));
        assert_eq!(
            parse_identifier("https://www.semanticscholar.org/paper/Attention-is-All-you-Need-Vaswani/204e3073870fae3d05bcbc2f6a8e263d9b72e776").unwrap(),
            Identifier::SemanticScholar("204e3073870fae3d05bcbc2f6a8e263d9b72e776".into())
        );
        assert_eq!(parse_identifier("CorpusId:13756489").unwrap(), Identifier::SemanticScholar("CorpusId:13756489".into()));
        assert!(parse_identifier("not an identifier").is_err());
    }
}
