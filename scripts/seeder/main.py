import sys
from pathlib import Path

# Add src directory to Python path
sys.path.insert(0, str(Path(__file__).parent / 'src'))

from src.nosql.nosql_seeder import run_nosql_seeders
from src.sql.sql_seeder import run_sql_seeders
from src.shared_utils.load import load_config


def main():
  print("chaty app data seeder")

  try:
    config = load_config()
    print(f"Loaded config for environment: {config.production and 'production' or 'development'}")

    # run_sql_seeders(config)
    run_nosql_seeders(config)
  except Exception as e:
    print(f"Error: {e}")


if __name__ == "__main__":
  main()
