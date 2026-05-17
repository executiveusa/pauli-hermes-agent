import os
import sys

# Add the current directory to sys.path so we can import plugins
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from dotenv import load_dotenv
from supabase import create_client
from plugins.memory.second_brain.ingestion_pipeline import SecondBrainIngester

# Load the project .env to get Supabase keys
load_dotenv()

url = os.environ.get("SUPABASE_URL")
key = os.environ.get("SUPABASE_SERVICE_KEY") or os.environ.get("SUPABASE_KEY")

if not url or not key:
    print("Error: SUPABASE_URL and SUPABASE_KEY/SUPABASE_SERVICE_KEY must be set in your .env file")
    sys.exit(1)

print("Connecting to Supabase...")
client = create_client(url, key)
ingester = SecondBrainIngester(client)

pdf_path = r"E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\personal-data\Building-a-Second-Brain-By-Tiago-Forte.pdf"
chatgpt_json = r"E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\personal-data\CHAT GPT EXPORT 11.27.2025\conversations.json"
other_json = r"E:\ACTIVE PROJECTS-PIPELINE\HERMES AGENT.remote-main\personal-data\data-ed4eef1f-f754-4d19-8d4d-58c34a921666-1778658925-2e1817e8-batch-0000\conversations.json"

if os.path.exists(pdf_path):
    ingester.ingest_pdf(pdf_path)
else:
    print(f"PDF not found: {pdf_path}")
    
if os.path.exists(chatgpt_json):
    ingester.ingest_chatgpt_export(chatgpt_json)
else:
    print(f"ChatGPT JSON not found: {chatgpt_json}")
    
if os.path.exists(other_json):
    ingester.ingest_chatgpt_export(other_json)
else:
    print(f"Other JSON not found: {other_json}")

print("\n--- All Done! ---")
