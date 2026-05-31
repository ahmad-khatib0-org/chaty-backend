import subprocess

from cassandra.cluster import Cluster
from psycopg2.extensions import connection

from src.nosql.groups import seed_messages_groups
from src.models.config import Config


def run_nosql_seeders(cfg: Config, sql_connection: connection):
  cluster = Cluster([get_scylla_ip()])
  session = cluster.connect("chaty")

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
