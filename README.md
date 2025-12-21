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
- **PTO Tracking**: Manage paid time off with:
  - Annual PTO hour allocation
  - Time off planning with status tracking (Planned/Requested/Approved/Completed)
  - Holiday calendar management per year
  - Auto-calculation of workday hours (M-F, 8hrs/day) with holiday deductions
  - Planned vs. used hour tracking
  - Copy holidays from previous years
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
# Create .env file with your PostgreSQL connection
echo "MONEYBAE_DATABASE_URL=postgres://username@localhost/money_bae" > .env

# For system-wide installation, add to ~/.zshrc (or ~/.bashrc)
echo 'export MONEYBAE_DATABASE_URL="postgres://username@localhost/money_bae"' >> ~/.zshrc
source ~/.zshrc
```

3. Run migrations:
```bash
diesel migration run
```

4. Build and run:
```bash
cargo run
```

## Installation

### System-wide Installation

Install the binary to `~/.cargo/bin/` for system-wide access:

```bash
cargo install --path .
```

Then run from anywhere:
```bash
money-bae

# Check version
money-bae --version

# Show help
money-bae --help
```

### Development Workflow

```bash
# During development - test changes
cargo run

# When satisfied - update system-wide version
cargo install --path .

# Uninstall
cargo uninstall money-bae
```

### Version Management

When ready to release a new version:

1. Update version in `Cargo.toml`
2. Commit the version bump
3. Create a git tag
4. Install system-wide

```bash
# Update Cargo.toml version, then:
git add Cargo.toml
git commit -m "Bump version to 0.2.0"
git tag -a v0.2.0 -m "Release 0.2.0: Description of changes"
git push origin v0.2.0

# Install the new version
cargo install --path .
```

## Configuration

Application configuration is managed using `confy` and follows the XDG Base Directory specification.

### Configuration File Location

| Platform | Path |
|----------|------|
| **Linux** | `$XDG_CONFIG_HOME/money-bae/` or `$HOME/.config/money-bae/` |
| **macOS** | `$HOME/.config/money-bae/` |
| **Windows** | `{FOLDERID_RoamingAppData}/money-bae/config/` |

The configuration file will be automatically created on first run with default values.

### Editing Configuration

```bash
# On Linux/macOS
vi ~/.config/money-bae/money-bae.toml

# View current configuration
cat ~/.config/money-bae/money-bae.toml
```

## Logs

Application logs (including panics) are written to:
- **Primary:** `~/.money-bae.log` (hidden file in home directory)
- **Fallback:** `./money-bae.log` (current directory)
- **Last resort:** `/tmp/money-bae.log`

View logs:
```bash
tail -f ~/.money-bae.log
```

## Usage

### Navigation
- `q` - Quit application
- `c` - Back/Clear view
- `i` - Income table
- `b` - Bills table
- `l` - Ledger table
- `p` - PTO management

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

### PTO Management
- Create annual PTO records with available hours
- Plan time off entries with date ranges
- Track status transitions: Planned → Requested → Approved → Completed
- Automatic workday hour calculation (excludes weekends)
- Holiday hour tracking per year
- Hours auto-calculated when holidays defined, or manual override
- View planned, used, and remaining hours per year
- Copy holiday calendar from previous year

## Database Schema

- `bills` - Recurring bill templates
- `incomes` - Income entries (assignable to ledgers)
- `ledgers` - Monthly financial snapshots
- `ledger_bills` - Bill instances in specific ledgers
- `ptos` - Annual PTO records with hour allocations
- `pto_plan` - Planned time off entries with date ranges and status
- `holiday_hours` - Holiday calendar entries per PTO year

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
