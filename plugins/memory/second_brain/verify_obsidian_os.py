import os
from pathlib import Path
from ingestion_pipeline import SecondBrainIngester
from canvas_generator import JSONCanvasGenerator

# Mock Supabase Client for testing
class MockSupabase:
    def table(self, name):
        return self
    def insert(self, data):
        return self
    def execute(self):
        class MockData:
            data = [{"id": "mock_id_123"}]
        return MockData()

def main():
    vault_path = Path("Test_Obsidian_Vault")
    vault_path.mkdir(exist_ok=True)
    inbox_path = vault_path / "Inbox"
    inbox_path.mkdir(exist_ok=True)
    
    # 1. Simulate Obsidian Clipper sending an article
    test_article = inbox_path / "AI_Agents_Future.md"
    with open(test_article, "w", encoding="utf-8") as f:
        f.write("# The Future of AI Agents\n\nAI agents are becoming highly autonomous systems capable of executing complex workflows.")
        
    print(f"Simulated Clipper drop: {test_article}")
    
    # 2. Run ingestion pipeline
    supabase = MockSupabase()
    ingester = SecondBrainIngester(supabase, vault_path=str(vault_path))
    
    # We will just test the cluster & distill method directly to see if JSON Canvas gets generated
    # (Because the full ingest_obsidian_inbox tries to use sentence_transformers which might not be installed)
    mock_episodes = [
        {"id": "ep1", "title": "AI Agents Future", "text": "AI agents will automate coding."},
        {"id": "ep2", "title": "Obsidian Plugins", "text": "Obsidian plugins allow custom behavior."},
        {"id": "ep3", "title": "Headless Sync", "text": "Syncing headlessly is great for servers."}
    ]
    
    clusters = {
        "AI & Tech": [mock_episodes[0]],
        "Obsidian Mastery": [mock_episodes[1], mock_episodes[2]]
    }
    
    print("Testing Canvas Generator...")
    canvas_gen = JSONCanvasGenerator(str(vault_path))
    canvas_gen.generate_layout(clusters)
    saved_path = canvas_gen.save("Agent_Brain_Map")
    
    if Path(saved_path).exists():
        print(f"SUCCESS: JSON Canvas created at {saved_path}")
        with open(saved_path, "r") as f:
            print("Canvas Output Preview:")
            print(f.read()[:200] + "...")
    else:
        print("FAILED to create JSON Canvas.")

if __name__ == "__main__":
    main()
