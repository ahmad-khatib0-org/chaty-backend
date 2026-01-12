CREATE TYPE IF NOT EXISTS interactions_type (
  reactions frozen<set<text>>, -- Reactions which should always appear (optional)
  restrict_reactions boolean -- Whether reactions should be restricted
);

