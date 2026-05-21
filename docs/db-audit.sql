/*
Purpose:
  Read-only PostgreSQL audit queries for inspecting index usage, sequential scans,
  relation sizes, JSONB TOAST compression, and duplicate/redundant indexes.

Usage:
  Run individual sections in psql or a PostgreSQL client against the target database.
  Results are diagnostic snapshots and may vary as pg_stat counters change.

Safety:
  All queries in this file are read-only and safe to run anytime. They only SELECT
  from PostgreSQL catalog/statistics views and do not modify schema or data.
*/

-- Query 1: Unused indexes
-- Helps identify non-unique, non-primary indexes in public schema that have never
-- been used according to pg_stat_user_indexes.idx_scan since statistics were last
-- reset. Interpret rows as cleanup candidates, not automatic drop instructions:
-- validate against recent workload coverage, infrequent maintenance/reporting
-- queries, and application release history before removing an index.
SELECT
  schemaname || '.' || relname AS table,
  indexrelname AS index,
  pg_size_pretty(pg_relation_size(indexrelid)) AS size,
  idx_scan
FROM pg_stat_user_indexes
JOIN pg_index USING (indexrelid)
WHERE NOT indisunique
  AND NOT indisprimary
  AND idx_scan = 0
  AND schemaname = 'public'
ORDER BY pg_relation_size(indexrelid) DESC;

-- Query 2: High seq-scan tables
-- Surfaces public tables with sequential scans, ranked by the number of tuples
-- read through those scans. High seq_tup_read with low idx_scan can indicate
-- missing indexes or queries that cannot use existing indexes; high seq_scan on
-- tiny tables may be harmless because scanning the whole table is cheaper.
-- Compare the rows estimate with workload expectations before acting.
SELECT
  relname AS table,
  seq_scan, seq_tup_read,
  idx_scan, idx_tup_fetch,
  n_live_tup AS rows
FROM pg_stat_user_tables
WHERE schemaname = 'public' AND seq_scan > 0
ORDER BY seq_tup_read DESC
LIMIT 20;

-- Query 3: Table + index bloat / size ranking
-- Ranks public heap tables by total on-disk footprint, split into heap, indexes,
-- and TOAST storage. Use it to find relations where data, index overhead, or
-- toasted values dominate disk usage. Large size alone is not proof of bloat;
-- treat this as a prioritization view for follow-up checks such as vacuum/analyze
-- history, index necessity, and data retention policy.
SELECT
  c.relname,
  pg_size_pretty(pg_total_relation_size(c.oid)) AS total,
  pg_size_pretty(pg_relation_size(c.oid)) AS heap,
  pg_size_pretty(pg_indexes_size(c.oid)) AS indexes,
  pg_size_pretty(pg_total_relation_size(c.reltoastrelid)) AS toast,
  c.reltuples::bigint AS approx_rows
FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
WHERE n.nspname='public' AND c.relkind='r'
ORDER BY pg_total_relation_size(c.oid) DESC
LIMIT 30;

-- Query 4: TOAST compression in use
-- Lists JSONB columns in public schema and the configured per-column TOAST
-- compression method. Interpret pglz/lz4 as explicit settings and default as
-- inheriting the database/server default. This helps confirm whether large JSONB
-- payloads are using the intended compression before comparing storage size or
-- considering column-level compression changes.
SELECT relname, attname,
  CASE attcompression WHEN 'p' THEN 'pglz' WHEN 'l' THEN 'lz4' WHEN '\0' THEN 'default' ELSE attcompression::text END AS compression
FROM pg_attribute a
JOIN pg_class c ON a.attrelid=c.oid
JOIN pg_namespace n ON c.relnamespace=n.oid
JOIN pg_type t ON a.atttypid=t.oid
WHERE n.nspname='public' AND t.typname='jsonb' AND a.attnum>0 AND NOT a.attisdropped
ORDER BY relname, attname;

-- Query 5: Duplicate / redundant indexes
-- Finds index pairs on the same table where one index's column key sequence is a
-- left-prefix of another index's key sequence. Such pairs can be redundant when
-- predicates, uniqueness, included columns, operator classes, sort order, and
-- partial-index conditions are equivalent or irrelevant to the workload. Use the
-- output as a review queue: idx_a is the shorter/prefix candidate and idx_b is
-- the wider index that may cover it.
SELECT a.indexrelid::regclass AS idx_a, b.indexrelid::regclass AS idx_b,
       a.indrelid::regclass AS table_name
FROM pg_index a JOIN pg_index b ON a.indrelid = b.indrelid AND a.indexrelid <> b.indexrelid
WHERE array_to_string(a.indkey, ' ') || ' ' = substr(array_to_string(b.indkey, ' ') || ' ', 1, length(array_to_string(a.indkey, ' ')) + 1)
  AND a.indexrelid < b.indexrelid;
