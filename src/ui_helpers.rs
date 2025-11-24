use cursive::Cursive;
use cursive::views::{Button, HideableView};

pub fn toggle_buttons_visible(siv: &mut Cursive, item_count: usize, button_names: &[&str]) {
    for name in button_names {
        siv.call_on_name(name, |v: &mut HideableView<Button>| {
            v.set_visible(item_count > 0);
        });
    }
}
