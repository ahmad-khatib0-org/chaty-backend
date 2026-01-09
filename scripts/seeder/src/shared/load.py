import os
from pathlib import Path

import yaml

from models.config import Config


def load_config() -> Config:
  env = os.getenv('ENV', 'local')

  if env not in ['dev', 'local', 'production']:
    raise ValueError(f"Invalid environment: {env}. Must be one of: 'dev', 'local', 'production'")

  # Build path to config file (relative to this seeder script location)
  # The files are at ../../../../chaty.dev.yaml and ../../../../chaty.local.yaml
  script_dir = Path(__file__).parent.parent.parent.parent.parent
  config_file = script_dir / f"chaty.{env}.yaml"

  if not config_file.exists():
    raise FileNotFoundError(f"Config file not found: {config_file}")

  with open(config_file, 'r') as f:
    config_data = yaml.safe_load(f)

  return Config(**config_data)
