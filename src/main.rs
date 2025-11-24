extern crate cursive_table_view;

mod income_table;
mod schema;
mod models;
mod db;
mod bill_table;
mod ledger_table;
mod ledger_detail;
mod common_layout;
mod ui_helpers;

use cursive::Cursive;
use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use cursive::views::TextView;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("money-bae {}", VERSION);
                return;
            }
            "--help" | "-h" => {
                println!("money-bae {} - Personal Finance Tracker", VERSION);
                println!("\nUsage: money-bae [OPTIONS]\n");
                println!("Options:");
                println!("  -v, --version    Show version information");
                println!("  -h, --help       Show this help message");
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Try 'money-bae --help' for more information");
                std::process::exit(1);
            }
        }
    }

    let mut siv = cursive::default();
    // Start with a nicer theme than default
    siv.set_theme(cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Simple,
        palette: Palette::retro().with(|palette| {
            use cursive::theme::BaseColor::*;

            {
                // First, override some colors from the base palette.
                use cursive::theme::Color::TerminalDefault;
                use cursive::theme::PaletteColor::*;

                palette[Background] = TerminalDefault;
                palette[View] = TerminalDefault;
                palette[Primary] = White.dark();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
            }

            {
                // Then override some styles.
                use cursive::theme::Effect::*;
                use cursive::theme::PaletteStyle::*;
                use cursive::theme::Style;
                palette[Highlight] = Style::from(Yellow.light()).combine(Bold);
                palette[HighlightInactive] = Style::from(Cyan.dark());
            }
        }),
    });

    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('h', |s| clear(s));
    siv.add_global_callback('i', |s| show_income_table(s));
    siv.add_global_callback('b', |s| show_bill_table(s));
    siv.add_global_callback('l', |s| show_ledger_table(s));


    let main_menu = common_layout::create_screen(
        &format!("money-bae v{}", VERSION),
        TextView::new("Personal Finance Tracker\n\nPress a key to begin:").center(),
        &common_layout::standard_footer()
    );

    siv.add_layer(main_menu);

    siv.run();
}

fn show_income_table(siv: &mut Cursive) {
    let income_table = income_table::IncomeTableView::new();

    income_table.add_table(siv);
}

fn show_bill_table(siv: &mut Cursive) {
    let bill_table = bill_table::BillTableView::new();

    bill_table.add_table(siv);
}

fn show_ledger_table(siv: &mut Cursive) {
    let ledger_table = ledger_table::LedgerTableView::new();

    ledger_table.add_table(siv);
}

fn clear(siv: &mut Cursive){
    siv.pop_layer();

    let main_menu = common_layout::create_screen(
        &format!("money-bae v{}", VERSION),
        TextView::new("Personal Finance Tracker\n\nPress a key to begin:").center(),
        &common_layout::standard_footer()
    );

    siv.add_layer(main_menu);
}
