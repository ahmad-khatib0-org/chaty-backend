from psycopg2.extensions import connection

from src.models.config import Config
from src.sql.db import DatabasePool, parse_postgres_url
from src.sql.users import seed_users_table


def run_sql_seeders(cfg: Config, sql_connection: connection):
  """
    Run all SQL seeders.
  """

  seed_users_table(sql_connection, cfg)


def get_sql_connection(cfg: Config):
  db_params = parse_postgres_url(cfg.database.postgres)
  DatabasePool.initialize(minconn=1, maxconn=10, **db_params)
  conn = DatabasePool.get_conn()

  return conn
