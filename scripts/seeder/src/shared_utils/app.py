from datetime import datetime

import argon2


def get_time_miliseconds():
  return int(datetime.now().timestamp() * 1000)


def generate_argon2_hash(password: str) -> str:
  hasher = argon2.PasswordHasher(time_cost=2,
                                 memory_cost=19456,
                                 parallelism=1,
                                 hash_len=32,
                                 salt_len=16)
  return hasher.hash(password)
