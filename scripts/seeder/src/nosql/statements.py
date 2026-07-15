from dataclasses import dataclass

from cassandra.cluster import PreparedStatement


@dataclass
class PreparedServers:
  """Prepared statements for the servers table"""
  insert: PreparedStatement


@dataclass
class PreparedChannels:
  """Prepared statements for the channels and channels_by_user tables"""
  insert_channel: PreparedStatement
  insert_channel_by_user: PreparedStatement


@dataclass
class PreparedServerMembers:
  """Prepared statements for server_members and server_members_by_user tables"""
  insert_member: PreparedStatement
  insert_member_by_user: PreparedStatement


@dataclass
class PreparedMessages:
  """Prepared statements for the messages table"""
  pass


@dataclass
class PreparedServerBans:
  """Prepared statements for the server_bans table"""
  pass


@dataclass
class Prepared:
  """Container for all prepared NoSQL statements used by seeders"""

  servers: PreparedServers
  channels: PreparedChannels
  server_members: PreparedServerMembers
  messages: PreparedMessages
  server_bans: PreparedServerBans
