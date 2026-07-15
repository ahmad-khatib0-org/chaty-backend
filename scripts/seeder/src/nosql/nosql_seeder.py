import subprocess

from cassandra.cluster import Cluster
from psycopg2.extensions import connection

from src.nosql.groups import seed_messages_groups
from src.nosql.servers import seed_servers
from src.nosql.statements import (
    Prepared,
    PreparedChannels,
    PreparedMessages,
    PreparedServerBans,
    PreparedServerMembers,
    PreparedServers,
)
from src.models.config import Config


def run_nosql_seeders(cfg: Config, sql_connection: connection):
  cluster = Cluster([get_scylla_ip()])
  session = cluster.connect("chaty")

  # Prepare all statements upfront
  stmt = prepare_statements(session)

  seed_messages_groups(session, sql_connection, stmt)
  seed_servers(session, sql_connection, stmt)


def prepare_statements(sess) -> Prepared:
  """Prepare all CQL statements used by seeders, mirroring the Rust Prepared struct pattern"""

  insert_server = sess.prepare("""
    INSERT INTO servers (
        id, owner_id, name, description, default_permissions,
        icon, banner, flags, nsfw, analytics, discoverable,
        roles, categories, system_messages, stats,
        channels, created_at, updated_at
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  """)

  insert_channel = sess.prepare("""
    INSERT INTO channels (
        id, channel_type, "group", created_at, updated_at
    ) 
    VALUES (?, ?, {user_id: ?, name: ?, description: ?, recipients: ?, icon: ?, last_message_id: ?, permissions: ?, nsfw: ?}, ?, ?)
  """)

  insert_channel_by_user = sess.prepare("""
    INSERT INTO channels_by_user (
        user_id, channel_id, channel_type, "group", created_at, updated_at
    ) 
    VALUES (?, ?, ?, {user_id: ?, name: ?, description: ?, recipients: ?, icon: ?, last_message_id: ?, permissions: ?, nsfw: ?}, ?, ?)
  """)

  insert_server_member = sess.prepare("""
    INSERT INTO server_members (
        server_id, user_id, username, avatar, nickname,
        joined_at, roles, timeout, can_publish, can_receive
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  """)

  insert_server_member_by_user = sess.prepare("""
    INSERT INTO server_members_by_user (
        user_id, server_id
    )
    VALUES (?, ?)
  """)

  return Prepared(
      servers=PreparedServers(insert=insert_server),
      channels=PreparedChannels(
          insert_channel=insert_channel,
          insert_channel_by_user=insert_channel_by_user,
      ),
      server_members=PreparedServerMembers(
          insert_member=insert_server_member,
          insert_member_by_user=insert_server_member_by_user,
      ),
      messages=PreparedMessages(),
      server_bans=PreparedServerBans(),
  )


def get_scylla_ip():
  result = subprocess.run([
      "docker", "inspect", "-f", "'{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}'",
      "scylladb"
  ],
                          capture_output=True,
                          text=True)
  # Remove quotes from output
  return result.stdout.strip().strip("'")
