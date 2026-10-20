pub mod messages;
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

fn preset_host() -> Option<(&'static str, u16)> {
    match env_value("SMTP_PROVIDER")?.to_lowercase().as_str() {
        "google" | "gmail" => Some(("smtp.gmail.com", 587)),
        _ => None,
    }
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
        let preset = preset_host();
        let Some(host) = env_value("SMTP_HOST").or_else(|| preset.map(|(host, _)| host.to_string())) else {
            return Self { config: None, transport: None, problem: None };
        };

        let port = env_value("SMTP_PORT")
            .and_then(|value| value.parse().ok())
            .or(preset.map(|(_, port)| port))
            .unwrap_or(587);
        let security = Security::parse(&env_value("SMTP_SECURITY").unwrap_or_default(), port);
        let username = env_value("SMTP_USERNAME");
        let password = env_value("SMTP_PASSWORD").map(|value| value.replace(' ', ""));

        let from_text = env_value("SMTP_FROM").or_else(|| username.clone()).unwrap_or_default();
        let from = match from_text.parse::<Mailbox>() {
            Ok(mailbox) => mailbox,
            Err(_) => {
                let problem = format!("SMTP_FROM '{from_text}' is not a valid sender address");
                tracing::warn!("Email is disabled: {problem}");
                return Self { config: None, transport: None, problem: Some(problem) };
            }
        };

        let config = EmailConfig { host, port, security, from, has_credentials: username.is_some() && password.is_some() };
        match build_transport(&config, username, password) {
            Ok(transport) => {
                tracing::info!("Email enabled through {}:{}", config.host, config.port);
                Self { config: Some(config), transport: Some(transport), problem: None }
            }
            Err(problem) => {
                tracing::warn!("Email is disabled: {problem}");
                Self { config: None, transport: None, problem: Some(problem) }
            }
        }
    }

    pub fn is_configured(&self) -> bool {
        self.transport.is_some()
    }

    pub fn status(&self) -> EmailStatus {
        let config = self.config.as_ref();
        EmailStatus {
            configured: self.is_configured(),
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
