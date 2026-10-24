pub mod messages;
mod providers;
mod templates;

use lettre::message::header::ContentType;
use lettre::message::{Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Security {
    StartTls,
    Tls,
    Plain,
}

impl Security {
    fn parse(value: &str, port: u16) -> Self {
        match value.trim().to_lowercase().as_str() {
            "tls" | "ssl" | "implicit" => Self::Tls,
            "none" | "plain" | "off" => Self::Plain,
            "starttls" => Self::StartTls,
            _ if port == 465 => Self::Tls,
            _ => Self::StartTls,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::StartTls => "starttls",
            Self::Tls => "tls",
            Self::Plain => "none",
        }
    }
}

struct EmailConfig {
    provider: Option<&'static str>,
    host: String,
    port: u16,
    security: Security,
    from: Mailbox,
    has_credentials: bool,
}

pub struct OutgoingMessage {
    pub subject: String,
    pub text: String,
    pub html: String,
}

#[derive(Serialize)]
pub struct EmailStatus {
    pub configured: bool,
    pub provider: Option<&'static str>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub security: Option<&'static str>,
    pub from: Option<String>,
    pub has_credentials: bool,
    pub problem: Option<String>,
}

pub struct Mailer {
    config: Option<EmailConfig>,
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    problem: Option<String>,
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn disabled(problem: String) -> Mailer {
    tracing::warn!("Email is disabled: {problem}");
    Mailer { config: None, transport: None, problem: Some(problem) }
}

fn build_transport(config: &EmailConfig, username: Option<String>, password: Option<String>) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = match config.security {
        Security::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host).map_err(|error| error.to_string())?,
        Security::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host).map_err(|error| error.to_string())?,
        Security::Plain => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host),
    };
    let builder = builder.port(config.port);
    let builder = match (username, password) {
        (Some(username), Some(password)) => builder.credentials(Credentials::new(username, password)),
        _ => builder,
    };
    Ok(builder.build())
}

impl Mailer {
    pub fn from_environment() -> Self {
        let preset = match env_value("SMTP_PROVIDER") {
            Some(provider) => match providers::find_preset(&provider, env_value("SMTP_REGION").as_deref()) {
                Ok(preset) => Some(preset),
                Err(problem) => return disabled(problem),
            },
            None => None,
        };

        let Some(host) = env_value("SMTP_HOST").or_else(|| preset.as_ref().map(|preset| preset.host.clone())) else {
            return Self { config: None, transport: None, problem: None };
        };

        let port = env_value("SMTP_PORT")
            .and_then(|value| value.parse().ok())
            .or(preset.as_ref().map(|preset| preset.port))
            .unwrap_or(587);
        let security = Security::parse(&env_value("SMTP_SECURITY").unwrap_or_default(), port);
        let username = env_value("SMTP_USERNAME")
            .or_else(|| preset.as_ref().and_then(|preset| preset.default_username).map(str::to_string));
        let strips_password_spaces = preset.as_ref().is_some_and(|preset| preset.strips_password_spaces);
        let password = env_value("SMTP_PASSWORD")
            .map(|value| if strips_password_spaces { value.replace(' ', "") } else { value });

        let from_text = env_value("SMTP_FROM")
            .or_else(|| username.clone().filter(|name| name.contains('@')))
            .unwrap_or_default();
        let from = match from_text.parse::<Mailbox>() {
            Ok(mailbox) => mailbox,
            Err(_) if from_text.is_empty() => return disabled("SMTP_FROM is required when SMTP_USERNAME is not an email address".to_string()),
            Err(_) => return disabled(format!("SMTP_FROM '{from_text}' is not a valid sender address")),
        };

        let config = EmailConfig {
            provider: preset.as_ref().map(|preset| preset.label),
            host,
            port,
            security,
            from,
            has_credentials: username.is_some() && password.is_some(),
        };
        match build_transport(&config, username, password) {
            Ok(transport) => {
                tracing::info!("Email enabled through {}:{}", config.host, config.port);
                Self { config: Some(config), transport: Some(transport), problem: None }
            }
            Err(problem) => disabled(problem),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.transport.is_some()
    }

    pub fn status(&self) -> EmailStatus {
        let config = self.config.as_ref();
        EmailStatus {
            configured: self.is_configured(),
            provider: config.and_then(|config| config.provider),
            host: config.map(|config| config.host.clone()),
            port: config.map(|config| config.port),
            security: config.map(|config| config.security.label()),
            from: config.map(|config| config.from.to_string()),
            has_credentials: config.is_some_and(|config| config.has_credentials),
            problem: self.problem.clone(),
        }
    }

    pub async fn send(&self, recipient: &str, message: OutgoingMessage) -> Result<(), String> {
        let (Some(config), Some(transport)) = (&self.config, &self.transport) else {
            return Err("Email is not configured on this server".to_string());
        };

        let to = recipient.parse::<Mailbox>().map_err(|_| format!("'{recipient}' is not a valid email address"))?;
        let body = MultiPart::alternative()
            .singlepart(SinglePart::builder().header(ContentType::TEXT_PLAIN).body(message.text))
            .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(message.html));

        let email = Message::builder()
            .from(config.from.clone())
            .to(to)
            .subject(message.subject)
            .multipart(body)
            .map_err(|error| error.to_string())?;

        transport.send(email).await.map(|_| ()).map_err(|error| error.to_string())
    }
}
