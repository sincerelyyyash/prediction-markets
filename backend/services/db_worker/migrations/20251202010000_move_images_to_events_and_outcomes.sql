-- Move image metadata from markets to events/outcomes

-- Add image URL to events (if not already present)
ALTER TABLE events
ADD COLUMN IF NOT EXISTS img_url TEXT;

-- Add image URL to outcomes (if not already present)
ALTER TABLE outcomes
ADD COLUMN IF NOT EXISTS img_url TEXT;

-- Drop obsolete image column from markets (images now live on events/outcomes)
ALTER TABLE markets
DROP COLUMN IF EXISTS img_url;


