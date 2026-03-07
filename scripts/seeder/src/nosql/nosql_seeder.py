import subprocess

from cassandra.cluster import Cluster

from src.nosql.groups import seed_messages_groups
from src.models.config import Config
from src.sql.sql_seeder import get_sql_connection


def run_nosql_seeders(cfg: Config, ):
  cluster = Cluster([get_scylla_ip()])
  session = cluster.connect("chaty")

  sql_connection = get_sql_connection(cfg)
  seed_messages_groups(session, sql_connection)


def get_scylla_ip():
  result = subprocess.run([
      "docker", "inspect", "-f", "'{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}'",
      "scylladb"
  ],
                          capture_output=True,
                          text=True)
  # Remove quotes from output
  return result.stdout.strip().strip("'")
