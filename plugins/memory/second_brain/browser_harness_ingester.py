import os
import json
import logging
from pathlib import Path
from datetime import datetime
from typing import Dict, Any

logger = logging.getLogger(__name__)

class BrowserHarnessIngester:
    """
    Ingests data captured by the browser-harness extension/tool into the Obsidian Second Brain.
    Saves raw DOM data, highlights, and browsing telemetry directly into the Vault.
    """
    def __init__(self, vault_path: str = None):
        self.vault_path = vault_path or os.environ.get("OBSIDIAN_VAULT_PATH", os.path.expanduser("~/Obsidian_Hermes_Vault"))
        self.browser_dir = Path(self.vault_path) / "Browser"
        
    def _ensure_browser_dir(self):
        self.browser_dir.mkdir(parents=True, exist_ok=True)
        
    def ingest_session(self, session_data: Dict[str, Any]) -> str:
        """
        Takes raw session data from browser-harness and converts it to a Markdown note.
        Expected keys in session_data: 'url', 'title', 'timestamp', 'content', 'highlights'
        """
        self._ensure_browser_dir()
        
        url = session_data.get("url", "Unknown URL")
        title = session_data.get("title", "Untitled Browsing Session")
        timestamp = session_data.get("timestamp", datetime.now().isoformat())
        content = session_data.get("content", "")
        highlights = session_data.get("highlights", [])
        
        # Sanitize title for filename
        import re
        safe_title = re.sub(r'[\\/*?:"<>|]', "", title).strip()
        filename = f"{safe_title[:50]}_{int(datetime.now().timestamp())}.md"
        
        filepath = self.browser_dir / filename
        
        md_content = f"---\n"
        md_content += f"url: {url}\n"
        md_content += f"timestamp: {timestamp}\n"
        md_content += f"category: Browser\n"
        md_content += f"---\n\n"
        md_content += f"# {title}\n\n"
        
        if highlights:
            md_content += "## User Highlights\n"
            for hl in highlights:
                md_content += f"> {hl}\n\n"
                
        md_content += "## Page Content / DOM Snapshot\n"
        md_content += f"{content}\n"
        
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(md_content)
            
        logger.info(f"Ingested browser session into {filepath}")
        return str(filepath)
