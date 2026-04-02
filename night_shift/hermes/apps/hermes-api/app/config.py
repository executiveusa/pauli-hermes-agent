from pydantic import BaseModel
import os
class Settings(BaseModel):
    env: str = os.getenv("HERMES_ENV", "development")
    log_level: str = os.getenv("HERMES_LOG_LEVEL", "INFO")
    database_url: str = (f"postgresql+psycopg://{os.getenv('POSTGRES_USER','postgres')}:{os.getenv('POSTGRES_PASSWORD','postgres')}@{os.getenv('POSTGRES_HOST','postgres')}:{os.getenv('POSTGRES_PORT','5432')}/{os.getenv('POSTGRES_DB','hermes')}")
    redis_url: str = os.getenv("REDIS_URL", "redis://redis:6379/0")
settings = Settings()
