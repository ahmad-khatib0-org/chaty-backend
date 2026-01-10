from faker import Faker
from psycopg2.extensions import connection
from ulid import ULID

from models.config import Config
from models.settings import NUMBER_OF_USERS
from shared.app import generate_argon2_hash, get_time_miliseconds


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
