use super::brand::EmailBrand;
use super::templates::{self, Template};
use super::OutgoingMessage;

struct Email<'a> {
    template: &'a Template,
    subject: String,
    heading: &'a str,
    preheader: String,
    button: Option<(&'a str, &'a str)>,
    values: Vec<(&'a str, String)>,
}

pub struct SharedItem<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub link: Option<&'a str>,
}

fn site_line() -> String {
    crate::site_url::public_url()
        .map(|url| format!(" from <a href=\"{0}\" style=\"color:#7b8794;\">{0}</a>", templates::escape_html(&url)))
        .unwrap_or_default()
}

fn logo(brand: &EmailBrand) -> String {
    brand
        .logo_url
        .as_ref()
        .map(|url| templates::render(templates::LOGO, &[("logo_url", url.clone())], true))
        .unwrap_or_default()
}

fn build(brand: &EmailBrand, email: Email) -> OutgoingMessage {
    let mut values = email.values;
    values.push(("site_name", brand.site_name.clone()));
    values.push(("accent", brand.accent.clone()));
    if let Some((label, link)) = email.button {
        values.push(("link", link.to_string()));
        let button = templates::render(
            templates::BUTTON,
            &[("label", label.to_string()), ("link", link.to_string()), ("accent", brand.accent.clone())],
            true,
        );
        values.push(("button", button));
    }

    let content = templates::render(email.template.html, &values, true);
    let html = templates::render(
        templates::LAYOUT,
        &[
            ("subject", email.subject.clone()),
            ("heading", email.heading.to_string()),
            ("preheader", email.preheader),
            ("content", content),
            ("site_name", brand.site_name.clone()),
            ("accent", brand.accent.clone()),
            ("logo", logo(brand)),
            ("site_line", site_line()),
        ],
        true,
    );
    let text = templates::render(email.template.text, &values, false).trim().to_string() + "\n";

    OutgoingMessage { subject: email.subject, text, html }
}

pub fn email_verification(brand: &EmailBrand, username: &str, link: &str, code: &str, code_minutes: i64) -> OutgoingMessage {
    build(brand, Email {
        template: &templates::EMAIL_VERIFICATION,
        subject: "Confirm your email address".to_string(),
        heading: "Confirm your email",
        preheader: "Enter the code or open the link to confirm your email address.".to_string(),
        button: Some(("Confirm email", link)),
        values: vec![
            ("username", username.to_string()),
            ("code", code.to_string()),
            ("minutes", code_minutes.to_string()),
        ],
    })
}

pub fn password_reset(brand: &EmailBrand, username: &str, link: &str) -> OutgoingMessage {
    build(brand, Email {
        template: &templates::PASSWORD_RESET,
        subject: "Reset your password".to_string(),
        heading: "Reset your password",
        preheader: "Use this link within 1 hour to choose a new password.".to_string(),
        button: Some(("Choose a new password", link)),
        values: vec![("username", username.to_string())],
    })
}

pub fn sign_in_code(brand: &EmailBrand, username: &str, code: &str, minutes: i64) -> OutgoingMessage {
    build(brand, Email {
        template: &templates::SIGN_IN_CODE,
        subject: format!("Your {} sign-in code is {code}", brand.site_name),
        heading: "Your sign-in code",
        preheader: format!("Enter this code to sign in to {}.", brand.site_name),
        button: None,
        values: vec![
            ("username", username.to_string()),
            ("code", code.to_string()),
            ("minutes", minutes.to_string()),
        ],
    })
}

pub fn test_message(brand: &EmailBrand, admin_name: &str) -> OutgoingMessage {
    build(brand, Email {
        template: &templates::TEST_MESSAGE,
        subject: format!("{} test email", brand.site_name),
        heading: "Email is working",
        preheader: format!("Outgoing email from {} is set up correctly.", brand.site_name),
        button: None,
        values: vec![("admin", admin_name.to_string())],
    })
}

pub fn team_invitation(brand: &EmailBrand, team_name: &str, inviter_name: &str, role: &str, link: Option<&str>) -> OutgoingMessage {
    let label = format!("Open {}", brand.site_name);
    build(brand, Email {
        template: &templates::TEAM_INVITATION,
        subject: format!("You were added to {team_name}"),
        heading: "You joined a team",
        preheader: format!("You now have access to a team space on {}.", brand.site_name),
        button: link.map(|link| (label.as_str(), link)),
        values: vec![
            ("team", team_name.to_string()),
            ("inviter", inviter_name.to_string()),
            ("role", role.to_string()),
        ],
    })
}

fn role_action(role: &str) -> &'static str {
    match role {
        "editor" => "view and edit it",
        "suggester" => "view it and suggest changes",
        _ => "view it",
    }
}

pub fn item_shared(brand: &EmailBrand, username: &str, inviter_name: &str, item: SharedItem, role: &str) -> OutgoingMessage {
    let label = format!("Open {}", item.kind);
    build(brand, Email {
        template: &templates::ITEM_SHARED,
        subject: format!("{inviter_name} shared \"{}\" with you", item.name),
        heading: "Shared with you",
        preheader: format!("{inviter_name} gave you access to a {} on {}.", item.kind, brand.site_name),
        button: item.link.map(|link| (label.as_str(), link)),
        values: vec![
            ("username", username.to_string()),
            ("inviter", inviter_name.to_string()),
            ("kind", item.kind.to_string()),
            ("item", item.name.to_string()),
            ("role_action", role_action(role).to_string()),
        ],
    })
}

pub fn ownership_offered(brand: &EmailBrand, username: &str, owner_name: &str, item: SharedItem) -> OutgoingMessage {
    build(brand, Email {
        template: &templates::OWNERSHIP_OFFERED,
        subject: format!("{owner_name} wants to give you \"{}\"", item.name),
        heading: "Ownership offered to you",
        preheader: format!("Accept or decline in the drive on {}.", brand.site_name),
        button: item.link.map(|link| ("Review in the drive", link)),
        values: vec![
            ("username", username.to_string()),
            ("owner", owner_name.to_string()),
            ("kind", item.kind.to_string()),
            ("item", item.name.to_string()),
        ],
    })
}
