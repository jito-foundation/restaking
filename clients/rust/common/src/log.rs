pub trait PrettyDisplay {
    fn pretty_display(&self) -> String;
}

pub fn account_header(title: &str) -> String {
    format!("\n{}\n", title)
}

pub fn section_header(title: &str) -> String {
    format!("\n━━━ {} ━━━\n", title)
}

pub fn field(name: &str, value: impl std::fmt::Display) -> String {
    format!("  {}: {}\n", name, value)
}
