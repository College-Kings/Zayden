-- Add up migration script here
ALTER TABLE levels ADD COLUMN xp INT NOT NULL DEFAULT 0;
ALTER TABLE levels ADD COLUMN level INT NOT NULL DEFAULT 0;