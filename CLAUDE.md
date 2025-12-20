# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

money-bae: Terminal UI personal finance tracker built with Rust using the Cursive TUI library. Tracks income, bills, ledgers, and PTO through an interactive ncurses-based interface.

## Environment Setup

**Critical:** `MONEYBAE_DATABASE_URL` must be set for application to run.

### For Development (cargo run)
```bash
# .env file (gitignored, in project root)
MONEYBAE_DATABASE_URL=postgres://username@localhost/money_bae
```

### For System-wide Installation
```bash
# Add to ~/.zshrc or ~/.bashrc
export MONEYBAE_DATABASE_URL="postgres://username@localhost/money_bae"

# Reload shell
source ~/.zshrc
```

**Why both?**
- `.env`: Used by `cargo run` from project directory
- Shell export: Required for installed binary (`money-bae` command) to work from any directory

## Build & Run Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Build release version
cargo build --release

# Run tests
cargo test

# Check compilation without building
cargo check
```

## Release & Installation Workflow

### Development Cycle
```bash
# Test changes during development
cargo run

# Check version
cargo run -- --version

# When satisfied with changes, update system-wide binary
cargo install --path .

# Binary installed to ~/.cargo/bin/money-bae
# Run from anywhere: money-bae
```

### Version Release Workflow

When ready to release a new version:

1. **Update version** in `Cargo.toml` (follow semantic versioning)
2. **Commit version bump**:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to X.Y.Z"
   ```
3. **Create git tag**:
   ```bash
   git tag -a vX.Y.Z -m "Release X.Y.Z: Brief description of changes"
   git push origin vX.Y.Z
   ```
4. **Install system-wide**:
   ```bash
   cargo install --path .
   ```
5. **Verify**:
   ```bash
   money-bae --version
   ```

### Semantic Versioning Guide
- **Major (X.0.0)**: Breaking changes, incompatible API changes
- **Minor (0.X.0)**: New features, backwards-compatible
- **Patch (0.0.X)**: Bug fixes, backwards-compatible

### Notes
- `cargo run`: Use during development, no installation needed
- `cargo install --path .`: Rebuilds and replaces system-wide binary
- No need to uninstall between updates
- Uninstall: `cargo uninstall money-bae`
- Version displayed in TUI title bar and via `--version` flag

## Architecture

### Core Structure
- `src/main.rs`: Application entry point, sets up Cursive instance with custom retro theme, defines global key bindings
- `src/income_table.rs`: Income tracking module with table view implementation
- `src/bill_table.rs`: Bill management module
- `src/ledger_table.rs`: Ledger (monthly snapshot) management module
- `src/pto_table.rs`: PTO records table (annual allocations)
- `src/pto_detail.rs`: PTO detail view (planning + holiday management)
- `src/pto_logic.rs`: PTO hour calculation logic

### Key Components

**Main Application (main.rs)**
- Initializes Cursive TUI with custom theme (retro palette, terminal default background)
- Global keybindings:
  - `q`: Quit application
  - `c`: Clear current view, return to main menu
  - `i`: Income table view
  - `b`: Bills table view
  - `l`: Ledger table view
  - `p`: PTO table view
- Uses layer-based view system for navigation

**Income Table (income_table.rs)**
- Table view for income entries (date + amount)
- Add/Edit/Delete operations
- Income can be assigned to ledgers for planning

**Bill Table (bill_table.rs)**
- Recurring bill templates with due dates, amounts
- Auto-pay flag for bills paid automatically
- Bills instantiated into ledgers for monthly tracking

**Ledger Table (ledger_table.rs)**
- Monthly financial snapshots
- Tracks bank balance, income assignments, bill payments
- Shows planned vs. paid breakdown, net balance

**PTO Management (pto_table.rs + pto_detail.rs + pto_logic.rs)**
- **PTO Table**: Annual PTO records (year, available hours, planned, used, remaining)
- **PTO Detail**: Two-panel view:
  - **Left panel**: Planned PTO entries (date ranges, status, hours)
  - **Right panel**: Holiday hours calendar
- **Status lifecycle**: Planned → Requested → Approved → Completed
- **Auto-calculation** (pto_logic.rs):
  - Counts M-F as 8-hour workdays
  - Excludes weekends
  - Subtracts holiday hours from range
  - If no holidays defined, uses manual hours entry
- **Database triggers**: Auto-update `hours_planned`, `hours_used`, `hours_remaining` on `pto_plan` changes
- **Holiday copying**: Copy previous year's holidays to new year with date shift

### Data Flow
1. User presses keybinding → triggers global callback in main.rs
2. Callback creates/modifies view → adds as layer to Cursive
3. IncomeTableView owns data, consumes self when adding to Cursive layer stack
4. Clear operation pops current layer, re-adds main menu TextView

### Cursive Layer System
Application uses layer stack for navigation. Layers pushed/popped to show different views. Current layer pattern: pop old view before pushing new one to maintain single active view.

## Development Notes

### Edition
Uses Rust 2024 edition (Cargo.toml:4). Requires recent Rust toolchain.

### Dependencies
- `cursive`: TUI framework with ncurses backend
- `cursive_table_view`: Table view widget extension
- `chrono`: Date/time handling
- `diesel`: PostgreSQL ORM
- `bigdecimal`: Precise monetary/hour calculations

### Database Schema
- `bills`: Recurring bill templates
- `incomes`: Income entries
- `ledgers`: Monthly financial snapshots
- `ledger_bills`: Bill instances in ledgers (many-to-many)
- `ptos`: Annual PTO records (aggregated hours)
- `pto_plan`: Planned time off entries (date ranges, status)
- `holiday_hours`: Holiday calendar per PTO year

**PTO Database Triggers**: `pto_plan` insert/update/delete automatically recalculates `ptos` aggregates (hours_planned, hours_used, hours_remaining) via PostgreSQL triggers.

## Development Workflow

### Communication Style
- Extremely concise interactions and commit messages. Sacrifice grammar for concision.

### Plans
- End each plan with unresolved questions list, if any. Extremely concise. Sacrifice grammar for concision.

### Plan Mode Context Management

**Overview**: Claude manages plans/tasks in `.claude/` directory (gitignored, local-only).

**Location**:
- Plans: `.claude/plans/{branch-name}.yaml`
- Context notes: `.claude/context/{branch-name}.md`

**Structure**:
```
.claude/
├── plans/{branch-name}.yaml     # Hierarchical: phases → tasks → subtasks
└── context/{branch-name}.md     # Freeform planning notes/decisions
```

**Workflow - Plan Mode Start**:
1. Detect current branch: `git branch --show-current`
2. Check for plan file: `.claude/plans/{branch}.yaml`
3. If exists: Load and present summary
4. If not: Create new plan structure
5. Present: phases, in-progress tasks, next actions

**YAML Structure**:
```yaml
branch: feature-name
created: ISO8601
updated: ISO8601
title: Brief plan description
phases:
  - id: p1
    title: Phase name
    status: pending|in_progress|completed
    tasks:
      - id: p1.1
        title: Task name
        status: pending|in_progress|completed
        notes: Optional details
        subtasks:
          - id: p1.1.1
            title: Subtask name
            status: pending|in_progress|completed
```

**Management Rules**:
- Auto-load plan on plan mode entry
- Update `updated` timestamp on every change
- One status per hierarchy level: pending → in_progress → completed
- Mark parent completed only when all children completed
- Store context/decisions in `.claude/context/{branch}.md`
- Never commit `.claude/` (local only)

**Context Switching**:
- Branch switch detected → auto-load new branch plan
- Each branch has isolated plan/context
- No cross-branch pollution

**File Operations**:
- Read: Single `Read` tool call
- Write: Single `Write` or `Edit` call
- No Bash subprocess overhead
- Direct YAML manipulation

**Status Tracking**:
- `pending`: Not started
- `in_progress`: Actively working
- `completed`: Done, verified
- Progression: pending → in_progress → completed
- Parents completed only when all children completed
- Claude updates automatically during work

**Example**: User switches to branch `feature/auth` → Claude:
1. Runs `git branch --show-current`
2. Reads `.claude/plans/feature-auth.yaml`
3. Presents: "Plan for feature/auth: 3 phases, phase 2 in progress, task p2.3 active"
