import random

from cassandra.cluster import Session
from psycopg2.extensions import connection
from ulid import ULID

from src.models.settings import NUMBER_OF_SERVERS
from src.nosql.statements import Prepared
from src.shared_utils.app import get_time_miliseconds
from src.sql.users import get_users

# Default permission value corresponding to DEFAULT_PERMISSION_SERVER constant
DEFAULT_PERMISSION_SERVER = 104960065


def seed_servers(sess: Session, conn: connection, stmt: Prepared):
  """Seed multiple servers with fake data"""

  users = get_users(conn, 10)
  user_ids = [user.id for user in users]

  server_names = [
      "Gaming Hub",
      "Tech Talk",
      "Art Studio",
      "Music Lounge",
      "Book Club",
      "Movie Night",
      "Study Group",
      "Foodies",
      "Travel Buddies",
      "Sports Central",
      "Anime World",
      "Photography",
      "Programming",
      "Design Corner",
      "Fitness Freaks",
  ]

  for _ in range(NUMBER_OF_SERVERS):
    server_id = str(ULID())
    created_at = get_time_miliseconds()
    updated_at = created_at

    owner_id = random.choice(user_ids)

    # Server name
    name = random.choice(server_names)
    if random.random() < 0.3:
      name = f"{name} #{random.randint(1, 999)}"

    description = random.choice([
        None,
        "A place for the community",
        "Welcome everyone!",
        "Hang out and have fun",
        "Discuss and share ideas",
        "Our little corner of the internet",
    ])

    # Default channel (General text channel, matching servers_create_presave)
    general_channel_id = str(ULID())
    channel_ids = [general_channel_id]

    nsfw = random.random() < 0.05
    analytics = False
    discoverable = False
    flags = None
    default_permissions = DEFAULT_PERMISSION_SERVER

    params_server = (
        server_id,
        owner_id,
        name,
        description,
        default_permissions,
        None,  # icon
        None,  # banner
        flags,
        nsfw,
        analytics,
        discoverable,
        {},  # roles (empty, matching presave)
        [],  # categories (empty, matching presave)
        None,  # system_messages (None, matching presave)
        None,  # stats (None, matching presave)
        channel_ids,
        created_at,
        updated_at,
    )
    sess.execute(stmt.servers.insert, params_server)

    # Insert server member (owner only, matching servers_create behavior)
    member_id = owner_id
    member_user = next((u for u in users if u.id == member_id), None)
    username = member_user.username if member_user else f"user_{member_id[:8]}"

    params_member = (
        server_id,
        member_id,
        username,
        None,  # avatar
        None,  # nickname
        created_at,
        set(),  # roles (empty set)
        None,  # timeout
        True,  # can_publish
        True,  # can_receive
    )
    sess.execute(stmt.server_members.insert_member, params_member)

    # Insert server_members_by_user
    params_by_user = (member_id, server_id)
    sess.execute(stmt.server_members.insert_member_by_user, params_by_user)

  print(f"Seeded {NUMBER_OF_SERVERS} servers")
