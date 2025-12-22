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

2. Set up environment files:
```bash
# Copy example files to create your environment files
cp .env.dev.example .env.dev
cp .env.prod.example .env.prod

# Edit with your database URLs
vi .env.dev
vi .env.prod
```

3. Set up databases:
```bash
# For local databases:
createdb money_bae_dev
createdb money_bae

# For Azure PostgreSQL, use connection strings from Terraform outputs
# Example: postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require

# Switch to development environment
./use-dev-env.sh

# Run migrations on dev database
diesel migration run
```

4. Configure application:
```bash
# Edit development config (auto-created on first run)
vi ~/.config/money-bae-dev/money-bae-dev.toml

# Add your database connection string:
# For local: database_connection_string = "postgres://username@localhost/money_bae_dev"
# For Azure: database_connection_string = "postgres://username:password@psql-mb-core-cus-dev.postgres.database.azure.com/money_bae?sslmode=require"
```

5. Build and run:
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

### Development vs Production

The application uses separate configs based on build profile:
- **Development** (`cargo run`): Uses `money-bae-dev` config
- **Production** (`cargo run --release` or installed binary): Uses `money-bae` config

### Configuration File Location

| Environment | Path |
|-------------|------|
| **Development** | `~/.config/money-bae-dev/money-bae-dev.toml` |
| **Production** | `~/.config/money-bae/money-bae.toml` |

Configuration files are automatically created on first run with default values.

### Editing Configuration

```bash
# Development config
vi ~/.config/money-bae-dev/money-bae-dev.toml

# Production config
vi ~/.config/money-bae/money-bae.toml
```

**Required setting:**
```toml
database_connection_string = "postgres://username@localhost/database_name"
```

### Database Environment Files

Diesel CLI uses `.env` files for migrations. **Never commit these files - they contain credentials.**

**Setup:**
```bash
# Copy example files
cp .env.dev.example .env.dev
cp .env.prod.example .env.prod

# Edit with your database URLs
vi .env.dev   # Development database URL
vi .env.prod  # Production database URL
```

**Files:**
- `.env.dev.example` - Template for dev database (tracked in git)
- `.env.prod.example` - Template for prod database (tracked in git)
- `.env.dev` - Actual dev database URL (gitignored, contains credentials)
- `.env.prod` - Actual prod database URL (gitignored, contains credentials)
- `.env` - Active env (gitignored, created by helper scripts)

**Helper scripts:**
```bash
# Switch to dev environment
./use-dev-env.sh

# Switch to prod environment
./use-prod-env.sh
```

### Production Database Workflow

**Always backup before migrations:**
```bash
# Backup production database
./backup-db.sh money_bae

# Switch to production environment
./use-prod-env.sh

# Run migrations
diesel migration run
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
