import random

from cassandra.cluster import Session
from psycopg2.extensions import connection
from ulid import ULID

from src.models.settings import NUMBER_OF_GROUPS
from src.nosql.statements import Prepared
from src.shared_utils.app import get_time_miliseconds
from src.sql.users import get_users


def seed_messages_groups(sess: Session, conn: connection, stmt: Prepared):
  """Seed multiple group channels with fake data"""

  users = get_users(conn, 10)
  user_ids = [user.id for user in users]

  for _ in range(NUMBER_OF_GROUPS):
    channel_id = str(ULID())
    created_at = get_time_miliseconds()
    updated_at = created_at

    owner_id = random.choice(user_ids)

    # Create recipients (owner + 2-5 other random users)
    num_recipients = random.randint(3, 6)
    recipients = frozenset(random.sample(user_ids, min(num_recipients, len(user_ids))))
    recipients = recipients | {owner_id}  # Ensure owner is included, use frozenset

    # Execute prepared statement with UDT fields as separate parameters
    params_channel = (
        channel_id,
        "group",
        owner_id,
        f"Group {random.choice(['Chat', 'Gaming', 'Study', 'Friends', 'Work', 'Project'])} #{random.randint(1, 999)}",  # name
        random.choice(
            [None, "General discussion", "Hang out", "Project collaboration", "Random chat"]),
        recipients,
        None,
        None,
        None,
        random.random() < 0.1,
        created_at,
        updated_at)

    sess.execute(stmt.channels.insert_channel, params_channel)

    for recipient_id in recipients:
      params_by_user = (
          recipient_id, channel_id, "group", owner_id,
          f"Group {random.choice(['Chat', 'Gaming', 'Study', 'Friends', 'Work', 'Project'])} #{random.randint(1, 999)}",
          random.choice([
              None, "General discussion", "Hang out", "Project collaboration", "Random chat"
          ]), recipients, None, None, None, random.random() < 0.1, created_at, updated_at)
      sess.execute(stmt.channels.insert_channel_by_user, params_by_user)

  print(f"Seeded {NUMBER_OF_GROUPS} group channels")