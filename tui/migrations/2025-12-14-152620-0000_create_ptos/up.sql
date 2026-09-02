CREATE TABLE ptos (
    id SERIAL PRIMARY KEY,
    year INTEGER NOT NULL UNIQUE,
    prev_year_hours NUMERIC(10, 2) NOT NULL DEFAULT 0,
    available_hours NUMERIC(10, 2) NOT NULL,
    hours_planned NUMERIC(10, 2) NOT NULL DEFAULT 0,
    hours_used NUMERIC(10, 2) NOT NULL DEFAULT 0,
    hours_remaining NUMERIC(10, 2) NOT NULL DEFAULT 0,
    rollover_hours BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
