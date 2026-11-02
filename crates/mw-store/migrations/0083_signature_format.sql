-- Preserve authored signature editing mode. NULL is a legacy row; callers infer
-- HTML source for existing nonempty HTML, otherwise plain text.
ALTER TABLE signatures ADD COLUMN format TEXT CHECK (format IS NULL OR format IN ('plain', 'rich', 'html'));
ALTER TABLE personal_signature ADD COLUMN format TEXT CHECK (format IS NULL OR format IN ('plain', 'rich', 'html'));
