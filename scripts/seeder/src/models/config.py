from typing import Dict, List, Optional

from pydantic import BaseModel


class Database(BaseModel):
  scylladb: str
  db_name: str
  postgres: str
  dragonfly: str


class Kafka(BaseModel):
  brokers: List[str]
  username: Optional[str] = None
  password: Optional[str] = None
  sasl_mechanism: Optional[str] = None
  security_protocol: Optional[str] = None


class Topics(BaseModel):
  password_reset: str
  password_reset_dlq: str
  user_created: str
  email_confirmation: str
  email_confirmation_dlq: str
  search_users_changes: str
  search_users_changes_dlq: str


class OAuth(BaseModel):
  public_url: str
  admin_url: str
  client_id: str
  client_secret: str
  redirect_uri: str
  scopes: List[str]
  token_endpoint: str
  auth_endpoint: str
  userinfo_endpoint: str
  confirmation_url: str
  reset_password_url: str


class Hosts(BaseModel):
  app: str
  api: str
  ws: str
  files: str
  gifs: str
  auth: str
  livekit: Dict[str, str]
  otel_collector: str
  api_metrics: str
  auth_metrics: str
  search_metrics: str


class ApiRegistration(BaseModel):
  invite_only: bool


class ApiSmtp(BaseModel):
  host: str
  username: str
  password: str
  from_address: str
  reply_to: Optional[str] = None
  port: Optional[int] = None
  use_tls: Optional[bool] = None
  use_starttls: Optional[bool] = None


class ApiEmailSendGrid(BaseModel):
  api_key: str
  from_address: str
  reply_to: Optional[str] = None


class ApiEmail(BaseModel):
  provider: str = "smtp"
  smtp: ApiSmtp
  sendgrid: ApiEmailSendGrid


class ApiSecurityCaptcha(BaseModel):
  hcaptcha_key: str
  hcaptcha_sitekey: str


class ApiSecurity(BaseModel):
  captcha: ApiSecurityCaptcha
  trust_cloudflare: bool
  easypwned: str
  tenor_key: str


class ApiWorkers(BaseModel):
  max_concurrent_connections: int


class LiveKitNode(BaseModel):
  url: str
  lat: float
  lon: float
  key: str
  secret: str
  private: bool = False


class ApiLiveKit(BaseModel):
  call_ring_duration: int
  nodes: Dict[str, LiveKitNode]


class ApiUsers(BaseModel):
  early_adopter_cutoff: Optional[int] = None


class Api(BaseModel):
  registration: ApiRegistration
  email: ApiEmail
  security: ApiSecurity
  workers: ApiWorkers
  livekit: ApiLiveKit
  users: ApiUsers


class PushVapid(BaseModel):
  queue: str
  private_key: str
  public_key: str


class PushFcm(BaseModel):
  queue: str
  key_type: str
  project_id: str
  private_key_id: str
  private_key: str
  client_email: str
  client_id: str
  auth_uri: str
  token_uri: str
  auth_provider_x509_cert_url: str
  client_x509_cert_url: str


class PushApn(BaseModel):
  queue: str
  sandbox: bool
  pkcs8: str
  key_id: str
  team_id: str


class Pushd(BaseModel):
  production: bool
  exchange: str
  mass_mention_chunk_size: int
  message_queue: str
  mass_mention_queue: str
  dm_call_queue: str
  fr_accepted_queue: str
  fr_received_queue: str
  generic_queue: str
  ack_queue: str
  vapid: PushVapid
  fcm: PushFcm
  apn: PushApn


class FilesLimit(BaseModel):
  min_file_size: int
  min_resolution: List[int]
  max_mega_pixels: int
  max_pixel_side: int


class FilesS3(BaseModel):
  endpoint: str
  path_style_buckets: bool
  region: str
  access_key_id: str
  secret_access_key: str
  default_bucket: str


class Files(BaseModel):
  encryption_key: str
  webp_quality: float
  blocked_mime_types: List[str]
  clamd_host: str
  scan_mime_types: List[str]
  limit: FilesLimit
  preview: Dict[str, List[int]]
  s3: FilesS3


class GlobalLimits(BaseModel):
  group_size: int
  message_embeds: int
  message_replies: int
  message_reactions: int
  server_emoji: int
  server_roles: int
  server_channels: int
  new_user_hours: int
  body_limit_size: int


class FeaturesLimits(BaseModel):
  outgoing_friend_requests: int
  bots: int
  message_length: int
  message_attachments: int
  servers: int
  voice_quality: int
  video: bool
  video_resolution: List[int]
  video_aspect_ratio: List[float]
  file_upload_size_limit: Dict[str, int]


class FeaturesAdvanced(BaseModel):
  process_message_delay_limit: int = 5


class FeaturesLimitsCollection(BaseModel):
  global_: Optional[GlobalLimits] = None
  new_user: Optional[FeaturesLimits] = None
  default: Optional[FeaturesLimits] = None

  def __init__(self, **data):
    # Handle the 'global' key from YAML
    if 'global' in data:
      data['global_'] = data.pop('global')
    super().__init__(**data)
  
  class Config:
    populate_by_name = True


class Features(BaseModel):
  limits: FeaturesLimitsCollection
  webhooks_enabled: bool
  mass_mentions_send_notifications: bool
  mass_mentions_enabled: bool
  advanced: Optional[FeaturesAdvanced] = None


class Search(BaseModel):
  host: str
  endpoints: List[str] = []
  api_key: str
  index_usernames: str
  index_usernames_dlq: str
  request_timeout_seconds: int


class Sentry(BaseModel):
  api: str
  ws: str
  voice_ingress: str
  files: str
  proxy: str
  pushd: str
  crond: str
  gifs: str


class Config(BaseModel):
  database: Database
  kafka: Kafka
  topics: Topics
  oauth: OAuth
  hosts: Hosts
  api: Api
  pushd: Pushd
  files: Files
  features: Features
  search: Search
  sentry: Sentry
  production: bool
  available_languages: List[str]
  default_language: str
