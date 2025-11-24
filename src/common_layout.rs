use cursive::views::{LinearLayout, Panel, TextView};
use cursive::View;

/// Creates a full-screen layout with header, content, and footer
pub fn create_screen<V: View>(title: &str, content: V, footer_hint: &str) -> impl View {
    LinearLayout::vertical()
        .child(Panel::new(TextView::new(title).center()))
        .child(Panel::new(content))
        .child(TextView::new(footer_hint).center())
}

/// Standard footer hints
pub fn standard_footer() -> String {
    "q:Quit | h:Home | i:Income | b:Bills | l:Ledger".to_string()
}

pub fn view_footer() -> String {
    standard_footer()
}