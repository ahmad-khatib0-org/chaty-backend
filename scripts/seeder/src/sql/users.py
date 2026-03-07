import json
from typing import List

from faker import Faker
from psycopg2.extensions import connection
from service.v1.users_db_pb2 import (
    USER_STATUS_BUSY,
    USER_STATUS_FOCUS,
    USER_STATUS_IDLE,
    USER_STATUS_INVISIBLE,
    USER_STATUS_ONLINE,
    User,
    UserStatus,
)
from ulid import ULID

from src.models.config import Config
from src.models.settings import NUMBER_OF_USERS
from src.shared_utils.app import generate_argon2_hash, get_time_miliseconds


def seed_users_table(con: connection, cfg: Config):
  """
  Seed the users table with realistic test data using Faker.
  Creates NUMBER_OF_USERS users with generated usernames and emails.
  
  Args:
    con: PostgreSQL database connection
    cfg: Application configuration
  """
  cursor = con.cursor()
  fake = Faker()

  current_time = get_time_miliseconds()

  # Prepare the INSERT statement
  insert_stmt = """
    INSERT INTO users (
      id, username, email, password_hash, display_name, badges,
      status_text, status_presence, profile_content, profile_background_id,
      privileged, suspended_until, created_at, updated_at, verified
    ) VALUES (
      %s, %s, %s, %s, %s, %s,
      %s, %s, %s, %s,
      %s, %s, %s, %s, %s
    )
  """

  # Generate and insert users
  used_usernames = set()
  used_emails = set()

  for _ in range(NUMBER_OF_USERS):
    user_id = str(ULID())

    while True:
      username = fake.user_name()
      if username not in used_usernames:
        used_usernames.add(username)
        break

    while True:
      email = fake.email()
      if email not in used_emails:
        used_emails.add(email)
        break

    password_hash = generate_argon2_hash("password123")

    display_name = fake.name()
    badges = fake.random_int(min=0, max=5)
    status_text = fake.sentence()[:510]  # Limit to 510 chars as per schema
    status_presence = "online"
    profile_content = fake.paragraph()
    profile_background_id = None
    privileged = fake.boolean(chance_of_getting_true=10)  # 10% chance
    suspended_until = None
    verified = fake.boolean(chance_of_getting_true=80)  # 80% chance verified

    cursor.execute(insert_stmt,
                   (user_id, username, email, password_hash, display_name, badges, status_text,
                    status_presence, profile_content, profile_background_id, privileged,
                    suspended_until, current_time, current_time, verified))

  print(f"Seeded {NUMBER_OF_USERS} users")


def get_users(conn: connection, num_of_users: int) -> List[User]:
  query = """
        SELECT 
            id, username, email, password_hash as password,
            display_name, badges, status_text, status_presence::text,
            profile_content, profile_background_id, privileged,
            suspended_until, created_at, updated_at, verified,
            avatar, relations, bot
        FROM users 
        LIMIT %s
    """

  cursor = conn.cursor()
  cursor.execute(query, (num_of_users, ))
  rows = cursor.fetchall()

  users = []
  for row in rows:
    user = User(id=row[0],
                username=row[1],
                email=row[2],
                password=row[3],
                display_name=row[4],
                badges=row[5],
                status_text=row[6],
                status_presence=status_str_to_enum(row[7]) if row[7] else None,
                profile_content=row[8],
                profile_background_id=row[9],
                privileged=row[10],
                suspended_until=row[11],
                created_at=row[12],
                updated_at=row[13],
                verified=row[14],
                avatar=json.loads(row[15]) if row[15] else None,
                relations=json.loads(row[16]) if row[16] else [],
                bot=json.loads(row[17]) if row[17] else None)
    users.append(user)

  cursor.close()
  return users


def status_str_to_enum(status_str: str) -> UserStatus | None:
  """Convert status string to proto enum number"""
  status_map = {
      "online": USER_STATUS_ONLINE,
      "idle": USER_STATUS_IDLE,
      "focus": USER_STATUS_FOCUS,
      "busy": USER_STATUS_BUSY,
      "invisible": USER_STATUS_INVISIBLE,
  }
  status_map.get(status_str)
