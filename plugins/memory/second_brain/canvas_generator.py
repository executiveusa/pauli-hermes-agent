import json
import logging
from pathlib import Path
from typing import List, Dict, Any

logger = logging.getLogger(__name__)

class JSONCanvasGenerator:
    """
    Generates JSON Canvas (.canvas) files for Obsidian.
    Allows visual representation of the agent's memory clusters.
    """
    def __init__(self, vault_path: str):
        self.vault_path = Path(vault_path)
        self.nodes = []
        self.edges = []
        
    def add_file_node(self, node_id: str, file_path: str, x: int, y: int, width: int = 400, height: int = 400):
        self.nodes.append({
            "id": str(node_id),
            "type": "file",
            "file": file_path,
            "x": x,
            "y": y,
            "width": width,
            "height": height
        })

    def add_text_node(self, node_id: str, text: str, x: int, y: int, width: int = 400, height: int = 400):
        self.nodes.append({
            "id": str(node_id),
            "type": "text",
            "text": text,
            "x": x,
            "y": y,
            "width": width,
            "height": height
        })

    def add_edge(self, edge_id: str, from_node: str, to_node: str, from_side: str = "right", to_side: str = "left"):
        self.edges.append({
            "id": str(edge_id),
            "fromNode": str(from_node),
            "fromSide": from_side,
            "toNode": str(to_node),
            "toSide": to_side
        })

    def generate_layout(self, clusters: Dict[str, List[Dict[str, Any]]]):
        """
        Takes clustered memory data and auto-generates a grid/circular layout.
        clusters: { "cluster_name": [ { "id": "...", "title": "..." }, ... ] }
        """
        # Very simple grid layout for demonstration
        x_offset = 0
        y_offset = 0
        spacing_x = 500
        spacing_y = 500
        
        for cluster_name, memories in clusters.items():
            # Add a text node for the cluster header
            cluster_node_id = f"cluster_{cluster_name}"
            self.add_text_node(cluster_node_id, f"# {cluster_name}", x_offset, y_offset, 300, 150)
            
            mem_x = x_offset + spacing_x
            mem_y = y_offset
            
            for i, mem in enumerate(memories):
                mem_id = mem.get("id", f"mem_{i}")
                title = mem.get("title", "Untitled")
                file_path = f"{title}.md"  # Assuming markdown file exists
                
                self.add_file_node(mem_id, file_path, mem_x, mem_y, 400, 400)
                self.add_edge(f"edge_{cluster_node_id}_{mem_id}", cluster_node_id, mem_id)
                
                mem_y += spacing_y
                
            x_offset += spacing_x * 2
            y_offset = 0 # reset y for next cluster column
            
    def save(self, filename: str) -> str:
        """Saves the canvas to the vault."""
        canvas_data = {
            "nodes": self.nodes,
            "edges": self.edges
        }
        
        if not filename.endswith(".canvas"):
            filename += ".canvas"
            
        out_path = self.vault_path / filename
        out_path.parent.mkdir(parents=True, exist_ok=True)
        
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(canvas_data, f, indent=2)
            
        logger.info(f"Canvas saved to {out_path}")
        return str(out_path)
