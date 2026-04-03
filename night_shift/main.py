from __future__ import annotations

import uvicorn


if __name__ == "__main__":
    uvicorn.run("night_shift.app:app", host="0.0.0.0", port=8787, reload=False)
