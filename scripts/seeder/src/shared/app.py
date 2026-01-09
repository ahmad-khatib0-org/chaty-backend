from datetime import datetime


def get_time_miliseconds():
  return int(datetime.now().timestamp() * 1000)
