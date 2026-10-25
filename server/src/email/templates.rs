pub const LAYOUT: &str = include_str!("templates/layout.html");
pub const BUTTON: &str = include_str!("templates/button.html");
pub const LOGO: &str = include_str!("templates/logo.html");

pub struct Template {
    pub html: &'static str,
    pub text: &'static str,
}

pub const EMAIL_VERIFICATION: Template = Template {
    html: include_str!("templates/email_verification.html"),
    text: include_str!("templates/email_verification.txt"),
};

pub const PASSWORD_RESET: Template = Template {
    html: include_str!("templates/password_reset.html"),
    text: include_str!("templates/password_reset.txt"),
};

pub const TEAM_INVITATION: Template = Template {
    html: include_str!("templates/team_invitation.html"),
    text: include_str!("templates/team_invitation.txt"),
};

pub const SIGN_IN_CODE: Template = Template {
    html: include_str!("templates/sign_in_code.html"),
    text: include_str!("templates/sign_in_code.txt"),
};

pub const TEST_MESSAGE: Template = Template {
    html: include_str!("templates/test_message.html"),
    text: include_str!("templates/test_message.txt"),
};

pub const ITEM_SHARED: Template = Template {
    html: include_str!("templates/item_shared.html"),
    text: include_str!("templates/item_shared.txt"),
};

pub const OWNERSHIP_OFFERED: Template = Template {
    html: include_str!("templates/ownership_offered.html"),
    text: include_str!("templates/ownership_offered.txt"),
};

pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn lookup<'a>(values: &'a [(&str, String)], name: &str) -> &'a str {
    values
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.as_str())
        .unwrap_or("")
}

pub fn render(template: &str, values: &[(&str, String)], escape: bool) -> String {
    let mut output = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let after_open = &remaining[start..];
        let is_raw = after_open.starts_with("{{{");
        let (open, close) = if is_raw { ("{{{", "}}}") } else { ("{{", "}}") };

        let Some(end) = after_open[open.len()..].find(close) else {
            output.push_str(after_open);
            return output;
        };
        let name = after_open[open.len()..open.len() + end].trim();
        let value = lookup(values, name);
        if escape && !is_raw {
            output.push_str(&escape_html(value));
        } else {
            output.push_str(value);
        }
        remaining = &after_open[open.len() + end + close.len()..];
    }

    output.push_str(remaining);
    output
}
