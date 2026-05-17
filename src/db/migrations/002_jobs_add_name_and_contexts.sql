ALTER TABLE jobs
    ADD COLUMN name           TEXT NOT NULL DEFAULT '',
    ADD COLUMN source_context TEXT NOT NULL DEFAULT '',
    ADD COLUMN target_context TEXT NOT NULL DEFAULT '';

-- Migrate old context into source_context
UPDATE jobs SET source_context = context, name = 'Untitled';

-- Drop the old column
ALTER TABLE jobs DROP COLUMN context;
