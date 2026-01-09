from models.config import Config
from sql.db import DatabasePool, parse_postgres_url
from sql.users import seed_users_table


def run_sql_seeders(cfg: Config):
  """
  Initialize database connection pool and run all SQL seeders.
  Wraps seeders in transaction for automatic rollback on failure.
  """

  conn = None
  try:
    db_params = parse_postgres_url(cfg.database.postgres)
    DatabasePool.initialize(minconn=1, maxconn=10, **db_params)
    conn = DatabasePool.get_conn()

    # Run all seeders with the connection
    seed_users_table(conn, cfg)

    # Commit the transaction
    conn.commit()
    print("Database seeding completed successfully")

  except Exception as e:
    if conn:
      conn.rollback()
    raise RuntimeError(f"Failed to run SQL seeders: {e}")
  finally:
    if conn:
      DatabasePool.release_conn(conn)
    DatabasePool.close_all()
