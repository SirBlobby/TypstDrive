use super::templates::{self, Template};
use super::OutgoingMessage;

struct Email<'a> {
    template: &'a Template,
    subject: String,
    heading: &'a str,
    preheader: &'a str,
    button: Option<(&'a str, &'a str)>,
    values: Vec<(&'a str, String)>,
}

fn site_line() -> String {
    crate::site_url::public_url()
        .map(|url| format!(" from <a href=\"{0}\" style=\"color:#7b8794;\">{0}</a>", templates::escape_html(&url)))
        .unwrap_or_default()
}

fn build(email: Email) -> OutgoingMessage {
    let mut values = email.values;
    if let Some((label, link)) = email.button {
        values.push(("link", link.to_string()));
        let button = templates::render(templates::BUTTON, &[("label", label.to_string()), ("link", link.to_string())], true);
        values.push(("button", button));
    }

    let content = templates::render(email.template.html, &values, true);
    let html = templates::render(
        templates::LAYOUT,
        &[
            ("subject", email.subject.clone()),
            ("heading", email.heading.to_string()),
            ("preheader", email.preheader.to_string()),
            ("content", content),
            ("site_line", site_line()),
        ],
        true,
    );
    let text = templates::render(email.template.text, &values, false).trim().to_string() + "\n";

    OutgoingMessage { subject: email.subject, text, html }
}

pub fn email_verification(username: &str, link: &str, code: &str, code_minutes: i64) -> OutgoingMessage {
    build(Email {
        template: &templates::EMAIL_VERIFICATION,
        subject: "Confirm your email address".to_string(),
        heading: "Confirm your email",
        preheader: "Enter the code or open the link to confirm your email address.",
        button: Some(("Confirm email", link)),
        values: vec![
            ("username", username.to_string()),
            ("code", code.to_string()),
            ("minutes", code_minutes.to_string()),
        ],
    })
}

pub fn password_reset(username: &str, link: &str) -> OutgoingMessage {
    build(Email {
        template: &templates::PASSWORD_RESET,
        subject: "Reset your password".to_string(),
        heading: "Reset your password",
        preheader: "Use this link within 1 hour to choose a new password.",
        button: Some(("Choose a new password", link)),
        values: vec![("username", username.to_string())],
    })
}

pub fn sign_in_code(username: &str, code: &str, minutes: i64) -> OutgoingMessage {
    build(Email {
        template: &templates::SIGN_IN_CODE,
        subject: format!("Your TypstDrive sign-in code is {code}"),
        heading: "Your sign-in code",
        preheader: "Enter this code to sign in to TypstDrive.",
        button: None,
        values: vec![
            ("username", username.to_string()),
            ("code", code.to_string()),
            ("minutes", minutes.to_string()),
        ],
    })
}

pub fn test_message(admin_name: &str) -> OutgoingMessage {
    build(Email {
        template: &templates::TEST_MESSAGE,
        subject: "TypstDrive test email".to_string(),
        heading: "Email is working",
        preheader: "Outgoing email from TypstDrive is set up correctly.",
        button: None,
        values: vec![("admin", admin_name.to_string())],
    })
}

pub fn team_invitation(team_name: &str, inviter_name: &str, role: &str, link: Option<&str>) -> OutgoingMessage {
    build(Email {
        template: &templates::TEAM_INVITATION,
        subject: format!("You were added to {team_name}"),
        heading: "You joined a team",
        preheader: "You now have access to a team space on TypstDrive.",
        button: link.map(|link| ("Open TypstDrive", link)),
        values: vec![
            ("team", team_name.to_string()),
            ("inviter", inviter_name.to_string()),
            ("role", role.to_string()),
        ],
    })
}
