def generate_prd_batch(label: str, selection_mode: str, batch_size: int) -> dict:
    return {"label": label, "selection_mode": selection_mode, "batch_size": batch_size, "status": "pending_approval"}
