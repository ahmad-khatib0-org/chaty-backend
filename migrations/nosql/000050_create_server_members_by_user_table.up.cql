--
-- MINIMAL table for "get servers user X is in"  
--
CREATE TABLE server_members_by_user (
  user_id text,
  server_id text,
  PRIMARY KEY ((user_id), server_id)
);
