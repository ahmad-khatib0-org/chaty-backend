import sys
from pathlib import Path

# Add src directory to Python path - only place this is needed
sys.path.insert(0, str(Path(__file__).parent / 'src'))

from sql.sql_seeder import run_sql_seeders
from shared.load import load_config


def main():
  print("chaty app data seeder")

  try:
    config = load_config()
    print(f"Loaded config for environment: {config.production and 'production' or 'development'}")

    run_sql_seeders(config)
    # run_nosql_seeders()  # in the future
  except Exception as e:
    print(f"Error: {e}")


if __name__ == "__main__":
  main()
