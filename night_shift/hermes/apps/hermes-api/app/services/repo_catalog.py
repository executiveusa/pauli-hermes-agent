from datetime import datetime
def catalog_repos(source: str, batch_size: int, include_archived: bool) -> dict:
    return {"source": source, "batch_size": batch_size, "include_archived": include_archived, "status": "queued", "queued_at": datetime.utcnow().isoformat()}
