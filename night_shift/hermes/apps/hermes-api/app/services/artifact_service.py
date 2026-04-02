def artifact_record(run_id: str, artifact_type: str, path: str) -> dict:
    return {"run_id": run_id, "artifact_type": artifact_type, "path": path}
