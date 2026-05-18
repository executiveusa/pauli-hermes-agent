import subprocess
import os
import sys

def register(ctx):
    def _cmd_llm_wiki(args):
        print("Starting LLM Wiki on http://127.0.0.1:8085")
        plugin_dir = os.path.dirname(os.path.abspath(__file__))
        server_script = os.path.join(plugin_dir, "server.py")
        subprocess.run([sys.executable, server_script])

    if hasattr(ctx, "register_cli_command"):
        ctx.register_cli_command(
            name="wiki",
            help="Start the LLM Wiki graph visualizer",
            handler=_cmd_llm_wiki
        )
