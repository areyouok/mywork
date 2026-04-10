-- Database Schema for MyWork Scheduler
-- SQLite database for managing scheduled tasks and their executions

-- Tasks table: stores scheduled task configurations
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    prompt TEXT NOT NULL,
    cron_expression TEXT,
    simple_schedule TEXT, -- JSON: {"type": "interval", "value": 5, "unit": "minutes"}
    once_at TEXT,
    enabled INTEGER DEFAULT 1,
    timeout_seconds INTEGER DEFAULT 300,
    working_directory TEXT, -- Custom working directory for task execution
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Executions table: stores task execution history
CREATE TABLE IF NOT EXISTS executions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    session_id TEXT, -- opencode session ID
    status TEXT NOT NULL, -- pending, running, success, failed, timeout, skipped
    started_at TEXT NOT NULL,
    finished_at TEXT,
    output_file TEXT, -- Path to output file
    error_message TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_executions_task_id ON executions(task_id);
CREATE INDEX IF NOT EXISTS idx_executions_started_at ON executions(started_at);
