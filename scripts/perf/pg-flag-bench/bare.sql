\set a random(1, 64)
\set n random(1, 200)
UPDATE messages SET flags_json = CASE WHEN flags_json = '[]' THEN '["Seen"]' ELSE '[]' END WHERE stable_id = 'm' || :a || '-' || :n;
