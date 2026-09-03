CREATE TABLE pto_plan (
    id SERIAL PRIMARY KEY,
    pto_id INTEGER NOT NULL REFERENCES ptos(id) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    name VARCHAR NOT NULL,
    description TEXT,
    hours NUMERIC(10, 2) NOT NULL,
    status VARCHAR NOT NULL DEFAULT 'Planned',
    custom_hours BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
