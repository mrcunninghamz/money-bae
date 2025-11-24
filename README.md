# money-bae

Terminal UI personal finance tracker built with Rust.

## Features

- **Bill Management**: Track recurring bills with due dates, amounts, and payment status
- **Income Tracking**: Record income entries with dates and amounts
- **Ledger System**: Create monthly financial snapshots with:
  - Bank balance tracking
  - Income vs. expenses analysis
  - Bill payment planning
  - Net balance calculations
- **Interactive TUI**: Clean terminal interface with table views and forms

## Tech Stack

- **Rust** - Core language
- **Cursive** - Terminal UI framework
- **Diesel** - ORM for PostgreSQL
- **PostgreSQL** - Database
- **BigDecimal** - Precise monetary calculations

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

## Setup

1. Clone the repository:
```bash
git clone git@github.com:mrcunninghamz/money-bae.git
cd money-bae
```

2. Configure database:
```bash
# Update .env with your PostgreSQL connection
DATABASE_URL=postgres://username@localhost/money_bae
```

3. Run migrations:
```bash
diesel migration run
```

4. Build and run:
```bash
cargo run
```

## Usage

### Navigation
- `q` - Quit application
- `c` - Back/Clear view
- `i` - Income table
- `b` - Bills table
- `l` - Ledger table

### Bills
- Add/Edit/Delete bills
- Set due dates and amounts
- Mark as auto-pay
- Toggle payment status in ledgers

### Income
- Add/Edit/Delete income entries
- Assign income to ledgers for planning

### Ledgers
- Create monthly financial snapshots
- Add bills with customizable amounts
- Assign income entries
- View planned vs. paid bill breakdown
- Calculate net balance

## Database Schema

- `bills` - Recurring bill templates
- `incomes` - Income entries (assignable to ledgers)
- `ledgers` - Monthly financial snapshots
- `ledger_bills` - Bill instances in specific ledgers

## Development

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT
