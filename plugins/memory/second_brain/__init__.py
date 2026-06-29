from .provider import SecondBrainProvider

def register(ctx):
    provider = SecondBrainProvider()
    ctx.register_memory_provider(provider)

    def _cmd_obsidian_sync(args):
        from .obsidian_sync import ObsidianSyncer
        from supabase import create_client
        import os
        url = os.environ.get("SUPABASE_URL")
        key = os.environ.get("SUPABASE_SERVICE_KEY") or os.environ.get("SUPABASE_KEY")
        if not url or not key:
            print("Error: SUPABASE_URL and SUPABASE_KEY/SUPABASE_SERVICE_KEY must be set in .env")
            return
        client = create_client(url, key)
        syncer = ObsidianSyncer(client)
        print(f"Syncing Second Brain to Obsidian vault: {syncer.vault_path}...")
        res = syncer.run_sync()
        if res.get("success"):
            print(f"Success! {res.get('message')}")
        else:
            print(f"Failed: {res.get('error')}")

    if hasattr(ctx, "register_cli_command"):
        ctx.register_cli_command(
            name="obsidian-sync",
            help="Sync Second Brain to Obsidian Vault",
            handler=_cmd_obsidian_sync
        )

        def _cmd_ingest_exports(args):
            from .ingestion_pipeline import SecondBrainIngester
            from supabase import create_client
            import os
            url = os.environ.get("SUPABASE_URL")
            key = os.environ.get("SUPABASE_SERVICE_KEY") or os.environ.get("SUPABASE_KEY")
            if not url or not key:
                print("Error: SUPABASE_URL and SUPABASE_KEY/SUPABASE_SERVICE_KEY must be set in .env")
                return
            client = create_client(url, key)
            ingester = SecondBrainIngester(client)
            
            # Paths requested by user
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

        ctx.register_cli_command(
            name="ingest-exports",
            help="Ingest PDF and JSON exports into the Second Brain",
            handler=_cmd_ingest_exports
        )
