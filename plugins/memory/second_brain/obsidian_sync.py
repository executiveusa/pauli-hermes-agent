import os
import json
import logging
import re
from pathlib import Path
from typing import Dict, Any, List

logger = logging.getLogger(__name__)

def sanitize_filename(name: str) -> str:
    """Removes invalid characters for filenames."""
    # Remove characters that are invalid in Windows/Unix filenames
    safe_name = re.sub(r'[\\/*?:"<>|]', "", name)
    return safe_name.strip()

class ObsidianSyncer:
    def __init__(self, supabase_client):
        self._supabase = supabase_client
        self.vault_path = os.environ.get("OBSIDIAN_VAULT_PATH", os.path.expanduser("~/Obsidian_Hermes_Vault"))
        
    def _ensure_vault_exists(self):
        Path(self.vault_path).mkdir(parents=True, exist_ok=True)
        
    def _fetch_all_memories(self) -> List[Dict[str, Any]]:
        # Handle pagination if necessary. For now, fetch up to 1000.
        res = self._supabase.table("memories").select("*").limit(1000).execute()
        return res.data

    def _fetch_memory_tags(self) -> Dict[str, List[str]]:
        # Fetch tags mapped to memories. Assuming `memory_tags` maps memory_id to tag_id 
        # and `tags` has id, name. For simplicity, we use RPC if available or join query.
        try:
            res = self._supabase.table("memory_tags").select("memory_id, tags(name)").execute()
            tag_map = {}
            for row in res.data:
                m_id = row.get("memory_id")
                tag_name = row.get("tags", {}).get("name")
                if m_id and tag_name:
                    if m_id not in tag_map:
                        tag_map[m_id] = []
                    tag_map[m_id].append(tag_name)
            return tag_map
        except Exception as e:
            logger.debug(f"Failed to fetch memory tags: {e}")
            return {}

    def _fetch_memory_links(self) -> Dict[str, List[str]]:
        try:
            # Assuming `memory_links` has source_id, target_id
            # We want to map source memory IDs to a list of target memory IDs
            res = self._supabase.table("memory_links").select("source_id, target_id").execute()
            link_map = {}
            for row in res.data:
                s_id = row.get("source_id")
                t_id = row.get("target_id")
                if s_id and t_id:
                    if s_id not in link_map:
                        link_map[s_id] = []
                    link_map[s_id].append(t_id)
            return link_map
        except Exception as e:
            logger.debug(f"Failed to fetch memory links: {e}")
            return {}
            
    def _build_frontmatter(self, memory: Dict[str, Any], tags: List[str]) -> str:
        fm = "---\n"
        fm += f"id: {memory.get('id')}\n"
        fm += f"title: \"{memory.get('title', 'Untitled').replace('\"', '\\\"')}\"\n"
        if memory.get("category"):
            fm += f"category: \"{memory.get('category')}\"\n"
        
        if tags:
            fm += "tags:\n"
            for t in tags:
                fm += f"  - {t}\n"
                
        fm += f"created_at: {memory.get('created_at', '')}\n"
        fm += f"updated_at: {memory.get('updated_at', '')}\n"
        fm += "---\n\n"
        return fm

    def run_sync(self) -> Dict[str, Any]:
        """Synchronizes Supabase memories to local Obsidian Markdown files."""
        if not self._supabase:
            return {"success": False, "error": "Supabase client not initialized"}
            
        try:
            self._ensure_vault_exists()
            
            memories = self._fetch_all_memories()
            tag_map = self._fetch_memory_tags()
            link_map = self._fetch_memory_links()
            
            # Create a map of memory_id to filename/title so we can create wikilinks
            id_to_title = {
                m.get("id"): sanitize_filename(m.get("title", f"Untitled_{m.get('id')}"))
                for m in memories
            }
            
            files_written = 0
            for memory in memories:
                m_id = memory.get("id")
                safe_title = id_to_title.get(m_id)
                if not safe_title:
                    continue
                    
                tags = tag_map.get(m_id, [])
                linked_ids = link_map.get(m_id, [])
                
                content = self._build_frontmatter(memory, tags)
                
                content += f"# {memory.get('title', 'Untitled')}\n\n"
                if memory.get("summary"):
                    content += f"**Summary:** {memory.get('summary')}\n\n"
                
                content += f"{memory.get('content', '')}\n\n"
                
                # Append Wikilinks
                if linked_ids:
                    content += "## Related Links\n"
                    for t_id in linked_ids:
                        target_title = id_to_title.get(t_id)
                        if target_title:
                            content += f"- [[{target_title}]]\n"
                            
                filepath = Path(self.vault_path) / f"{safe_title}.md"
                with open(filepath, "w", encoding="utf-8") as f:
                    f.write(content)
                files_written += 1
                
            return {
                "success": True, 
                "message": f"Successfully synced {files_written} memories to {self.vault_path}",
                "vault_path": str(self.vault_path),
                "files_written": files_written
            }
            
        except Exception as e:
            logger.exception("Obsidian sync failed")
            return {"success": False, "error": str(e)}
