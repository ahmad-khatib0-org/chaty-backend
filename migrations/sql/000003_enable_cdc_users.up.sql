-- Create a Changefeed for the users table
-- This captures changes (INSERT, UPDATE, DELETE) and publishes them to Kafka/Redpanda
-- The changefeed will emit to topic "search.users.changes"
--
-- Requirements:
-- 1. kv.rangefeed.enabled cluster setting must be true
-- 2. Kafka/Redpanda brokers must be accessible at the URI
-- 3. Kafka topic will be auto-created if auto.create.topics.enable=true on Redpanda
--
-- Notes:
-- - The 'resolved' option emits resolved timestamps (heartbeats) at intervals
-- - Messages include both before and after states for each change
-- - Only table changes are emitted, not schema changes
-- - When running in Docker, use redpanda:9092 (internal network) not localhost:9092
CREATE CHANGEFEED FOR TABLE users INTO 'kafka://redpanda:9092?topic_name=search.users.changes'
WITH resolved = '5s', format = 'json';
