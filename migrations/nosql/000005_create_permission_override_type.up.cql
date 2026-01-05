-- Permission override
CREATE TYPE IF NOT EXISTS permission_override_type (
  "allow" bigint, 
  "deny" bigint
);
