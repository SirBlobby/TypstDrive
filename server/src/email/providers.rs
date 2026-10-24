pub struct ProviderPreset {
    pub label: &'static str,
    pub host: String,
    pub port: u16,
    pub default_username: Option<&'static str>,
    pub strips_password_spaces: bool,
}

struct ProviderDefinition {
    names: &'static [&'static str],
    label: &'static str,
    port: u16,
    default_username: Option<&'static str>,
    strips_password_spaces: bool,
    host: fn(Option<&str>) -> Result<String, String>,
}

fn fixed(host: &'static str) -> Result<String, String> {
    Ok(host.to_string())
}

fn mailgun_host(region: Option<&str>) -> Result<String, String> {
    match region.unwrap_or("us") {
        "us" => fixed("smtp.mailgun.org"),
        "eu" => fixed("smtp.eu.mailgun.org"),
        other => Err(format!("SMTP_REGION '{other}' is not a Mailgun region. Use 'us' or 'eu'")),
    }
}

fn ses_host(region: Option<&str>) -> Result<String, String> {
    let region = region.unwrap_or("us-east-1");
    let valid = !region.is_empty() && region.chars().all(|character| character.is_ascii_alphanumeric() || character == '-');
    if valid {
        Ok(format!("email-smtp.{region}.amazonaws.com"))
    } else {
        Err(format!("SMTP_REGION '{region}' is not an AWS region such as 'eu-west-1'"))
    }
}

const PROVIDERS: &[ProviderDefinition] = &[
    ProviderDefinition {
        names: &["google", "gmail"],
        label: "Google",
        port: 587,
        default_username: None,
        strips_password_spaces: true,
        host: |_| fixed("smtp.gmail.com"),
    },
    ProviderDefinition {
        names: &["mailgun"],
        label: "Mailgun",
        port: 587,
        default_username: None,
        strips_password_spaces: false,
        host: mailgun_host,
    },
    ProviderDefinition {
        names: &["sendgrid"],
        label: "SendGrid",
        port: 587,
        default_username: Some("apikey"),
        strips_password_spaces: false,
        host: |_| fixed("smtp.sendgrid.net"),
    },
    ProviderDefinition {
        names: &["ses", "amazon-ses", "aws"],
        label: "Amazon SES",
        port: 587,
        default_username: None,
        strips_password_spaces: false,
        host: ses_host,
    },
    ProviderDefinition {
        names: &["resend"],
        label: "Resend",
        port: 587,
        default_username: Some("resend"),
        strips_password_spaces: false,
        host: |_| fixed("smtp.resend.com"),
    },
    ProviderDefinition {
        names: &["mailjet"],
        label: "Mailjet",
        port: 587,
        default_username: None,
        strips_password_spaces: false,
        host: |_| fixed("in-v3.mailjet.com"),
    },
    ProviderDefinition {
        names: &["microsoft", "outlook", "office365"],
        label: "Microsoft 365",
        port: 587,
        default_username: None,
        strips_password_spaces: false,
        host: |_| fixed("smtp.office365.com"),
    },
    ProviderDefinition {
        names: &["fastmail"],
        label: "Fastmail",
        port: 465,
        default_username: None,
        strips_password_spaces: false,
        host: |_| fixed("smtp.fastmail.com"),
    },
];

pub fn provider_names() -> Vec<&'static str> {
    PROVIDERS.iter().map(|provider| provider.names[0]).collect()
}

pub fn find_preset(provider: &str, region: Option<&str>) -> Result<ProviderPreset, String> {
    let requested = provider.trim().to_lowercase();
    let region = region.map(|value| value.trim().to_lowercase());
    let definition = PROVIDERS
        .iter()
        .find(|definition| definition.names.contains(&requested.as_str()))
        .ok_or_else(|| format!("SMTP_PROVIDER '{provider}' is not known. Use one of: {}", provider_names().join(", ")))?;

    Ok(ProviderPreset {
        label: definition.label,
        host: (definition.host)(region.as_deref())?,
        port: definition.port,
        default_username: definition.default_username,
        strips_password_spaces: definition.strips_password_spaces,
    })
}
