ALTER TABLE pages
  ADD COLUMN metadata text[] DEFAULT array[]::text[] NOT NULL;
