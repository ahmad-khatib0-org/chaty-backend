import sys
from pathlib import Path

# Add src directory to Python path
sys.path.insert(0, str(Path(__file__).parent / 'src'))

from src.sql.db import DatabasePool
from src.nosql.nosql_seeder import run_nosql_seeders
from src.sql.sql_seeder import get_sql_connection, run_sql_seeders
from src.shared_utils.load import load_config


def main():
  print("chaty app data seeder")

  config = load_config()
  print(f"Loaded config for environment: {config.production and 'production' or 'development'}")

  sql_connection = get_sql_connection(config)

  try:
    run_sql_seeders(config, sql_connection)
    sql_connection.commit()

    print("Sql Database seeding completed successfully")

  except Exception as e:
    if sql_connection:
      sql_connection.rollback()
    raise RuntimeError(f"Failed to run SQL seeders: {e}")

  run_nosql_seeders(config, sql_connection)

  if sql_connection:
    DatabasePool.release_conn(sql_connection)
  DatabasePool.close_all()


if __name__ == "__main__":
  main()
