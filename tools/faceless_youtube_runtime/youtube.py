from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


class YouTubeNotConfigured(RuntimeError):
    pass


def _service(scopes: list[str]):
    secret_file = os.getenv("HERMES_YOUTUBE_CLIENT_SECRET_FILE")
    token_file = os.getenv("HERMES_YOUTUBE_TOKEN_FILE")
    if not secret_file or not token_file:
        raise YouTubeNotConfigured("YouTube OAuth files are not configured")
    try:
        from google.auth.transport.requests import Request
        from google.oauth2.credentials import Credentials
        from google_auth_oauthlib.flow import InstalledAppFlow
        from googleapiclient.discovery import build
    except ImportError as exc:
        raise YouTubeNotConfigured("Install Hermes google extra before using YouTube OAuth") from exc

    creds = None
    token_path = Path(token_file).expanduser()
    if token_path.exists():
        creds = Credentials.from_authorized_user_file(str(token_path), scopes)
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            # OAuth setup is intentionally human-assisted. This opens the normal Google consent flow.
            flow = InstalledAppFlow.from_client_secrets_file(str(Path(secret_file).expanduser()), scopes)
            creds = flow.run_local_server(port=0)
        token_path.parent.mkdir(parents=True, exist_ok=True)
        token_path.write_text(creds.to_json(), encoding="utf-8")
        try:
            os.chmod(token_path, 0o600)
        except OSError:
            pass
    return build("youtube", "v3", credentials=creds)


def channel_snapshot(channel_id: str) -> dict[str, Any]:
    service = _service(["https://www.googleapis.com/auth/youtube.readonly"])
    response = service.channels().list(part="snippet,statistics,contentDetails", id=channel_id).execute()
    item = (response.get("items") or [None])[0]
    if not item:
        raise RuntimeError(f"YouTube channel not found: {channel_id}")
    return item


def upload_video(video_path: str, metadata: dict[str, Any], *, approved: bool = False) -> dict[str, Any]:
    if not approved:
        raise PermissionError("explicit publish approval is required")
    if os.getenv("HERMES_YOUTUBE_PUBLISH_ENABLED", "0") != "1":
        raise PermissionError("publishing is disabled; set HERMES_YOUTUBE_PUBLISH_ENABLED=1 after approval")
    path = Path(video_path).expanduser().resolve()
    if not path.is_file():
        raise FileNotFoundError(path)
    try:
        from googleapiclient.http import MediaFileUpload
    except ImportError as exc:
        raise YouTubeNotConfigured("Install Hermes google extra before publishing") from exc
    service = _service(["https://www.googleapis.com/auth/youtube.upload"])
    body = {
        "snippet": {
            "title": metadata["title"],
            "description": metadata.get("description", ""),
            "tags": metadata.get("tags", []),
            "categoryId": str(metadata.get("category_id", "22")),
        },
        "status": {
            "privacyStatus": metadata.get("privacy_status", "private"),
            "selfDeclaredMadeForKids": bool(metadata.get("made_for_kids", False)),
        },
    }
    request = service.videos().insert(
        part="snippet,status",
        body=body,
        media_body=MediaFileUpload(str(path), resumable=True),
    )
    response = None
    while response is None:
        _, response = request.next_chunk()
    return {"video_id": response.get("id"), "response": response}


def dump_snapshot(channel_id: str, output: Path) -> Path:
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(channel_snapshot(channel_id), indent=2) + "\n", encoding="utf-8")
    return output
