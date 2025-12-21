extern crate cursive_table_view;

mod income_table;
mod schema;
mod models;
mod db;
mod repositories;
mod bill_table;
mod ledger_table;
mod ledger_detail;
mod common_layout;
mod ui_helpers;
mod pto_logic;
mod pto_table;
mod pto_detail;
mod configuration_manager;
mod dependency_container;

use cursive::Cursive;
use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use cursive::views::TextView;
use simplelog::*;
use std::fs::File;
use std::rc::Rc;
use crate::db::PgConnector;
use crate::dependency_container::DependencyContainer;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let dc = Rc::new(DependencyContainer::new());
    
    // Initialize logging to file (since TUI uses stdio)
    let log_path = std::env::var("HOME")
        .map(|h| format!("{}/.money-bae.log", h))
        .unwrap_or_else(|_| "money-bae.log".to_string());

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(&log_path).unwrap_or_else(|_| {
                eprintln!("Warning: Could not create log file at {}", log_path);
                File::create("/tmp/money-bae.log").expect("Failed to create fallback log file")
            }),
        ),
    ]).ok(); // Ignore if logging fails to initialize

    // Log panics to file
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic payload"
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        log::error!("PANIC at {}: {}", location, message);
    }));

    log::info!("money-bae v{} started", VERSION);

    // Validate configuration before starting UI
    let config_manager = dc.configuration_manager();
    if config_manager.get_database_connection_string().is_none() {
        let config_name = configuration_manager::ConfigurationManager::get_config_name();
        eprintln!("\n❌ Configuration Error: database_connection_string not set");
        eprintln!("\nPlease edit your {}.toml configuration file.", config_name);
        eprintln!("(Location depends on your OS - see README or confy documentation)");
        eprintln!("\nAdd the following line:");
        eprintln!("  database_connection_string = \"postgres://username@localhost/database_name\"");
        eprintln!("\nExample for development:");
        eprintln!("  database_connection_string = \"postgres://{}@localhost/money_bae_dev\"", 
            std::env::var("USER").unwrap_or_else(|_| "username".to_string()));
        std::process::exit(1);
    }

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
                use cursive::theme::Color::{TerminalDefault, Rgb};
                use cursive::theme::PaletteColor::*;

                palette[Background] = TerminalDefault;
                palette[View] = TerminalDefault;
                palette[Primary] = White.light();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
                palette[HighlightText] = Rgb(255, 255, 255);  // Pure white text on highlighted rows
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
    
    let dc_income = Rc::clone(&dc);
    siv.add_global_callback('i', move |s| show_income_table(s, &dc_income));
    
    let dc_bill = Rc::clone(&dc);
    siv.add_global_callback('b', move |s| show_bill_table(s, &dc_bill));
    
    let dc_ledger = Rc::clone(&dc);
    siv.add_global_callback('l', move |s| show_ledger_table(s, &dc_ledger));
    
    let dc_pto = Rc::clone(&dc);
    siv.add_global_callback('p', move |s| show_pto_view(s, &dc_pto));


    let main_menu = common_layout::create_screen(
        &format!("money-bae v{}", VERSION),
        TextView::new("Personal Finance Tracker\n\nPress a key to begin:").center(),
        &common_layout::standard_footer()
    );

    siv.add_layer(main_menu);

    siv.run();
}

fn show_income_table(siv: &mut Cursive, dc: &DependencyContainer) {
    let income_table = income_table::IncomeTableView::new(dc.income_repo());

    income_table.add_table(siv);
}

fn show_bill_table(siv: &mut Cursive, dc: &DependencyContainer) {
    let bill_table = bill_table::BillTableView::new(dc.bill_repo());

    bill_table.add_table(siv);
}

fn show_ledger_table(siv: &mut Cursive, dc: &DependencyContainer) {
    let ledger_table = ledger_table::LedgerTableView::new(dc.ledger_repo());

    ledger_table.add_table(siv);
}

fn show_pto_view(siv: &mut Cursive, dc: &DependencyContainer) {
    siv.pop_layer();
    pto_table::show_pto_table_view(siv, &dc.pto_repo(), &dc.pto_plan_repo(), &dc.holiday_hours_repo());
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
