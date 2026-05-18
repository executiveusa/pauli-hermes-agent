import os
import subprocess
import logging
from typing import Dict, Any

logger = logging.getLogger(__name__)

class ObsidianHeadlessClient:
    """
    Python wrapper for the obsidian-headless sync engine.
    This allows the Hermes Agent to perform a two-way sync with an Obsidian Sync remote
    without needing the desktop application running.
    """
    def __init__(self, vault_path: str):
        self.vault_path = vault_path
        # Ensure credentials exist in env or fallback to empty strings
        self.email = os.environ.get("OBSIDIAN_EMAIL", "")
        self.password = os.environ.get("OBSIDIAN_PASSWORD", "")
        self.vault_password = os.environ.get("OBSIDIAN_VAULT_PASSWORD", "")
        self.binary_path = os.environ.get("OBSIDIAN_HEADLESS_BIN", "obsidian-headless")

    def sync(self) -> Dict[str, Any]:
        """
        Executes a two-way sync operation.
        Pulls remote changes and pushes local changes for the vault.
        """
        if not self.email or not self.password:
            logger.warning("Obsidian credentials not found in environment. Headless sync may fail.")
            return {"success": False, "error": "Missing OBSIDIAN_EMAIL or OBSIDIAN_PASSWORD"}

        env = os.environ.copy()
        env["OBSIDIAN_EMAIL"] = self.email
        env["OBSIDIAN_PASSWORD"] = self.password
        if self.vault_password:
            env["OBSIDIAN_VAULT_PASSWORD"] = self.vault_password

        # Example execution of the headless client
        cmd = [
            self.binary_path,
            "sync",
            "--vault", self.vault_path
        ]

        logger.info(f"Starting headless Obsidian sync for vault: {self.vault_path}")
        
        try:
            result = subprocess.run(
                cmd,
                env=env,
                capture_output=True,
                text=True,
                check=False
            )
            
            if result.returncode == 0:
                logger.info("Headless sync completed successfully.")
                return {
                    "success": True, 
                    "output": result.stdout
                }
            else:
                logger.error(f"Headless sync failed: {result.stderr}")
                return {
                    "success": False, 
                    "error": result.stderr
                }
                
        except FileNotFoundError:
            error_msg = f"Binary '{self.binary_path}' not found. Please ensure obsidian-headless is installed."
            logger.error(error_msg)
            return {"success": False, "error": error_msg}
        except Exception as e:
            logger.exception("Unexpected error during obsidian-headless execution.")
            return {"success": False, "error": str(e)}
