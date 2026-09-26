# CapCut Programmatic Integration

## Goal

Give Hermes a deterministic path from an edit decision to an editable CapCut/Jianying draft. Browser/desktop automation is a fallback, not the primary architecture.

## Backend Priority

### A — CapCutAPI (preferred local backend)

Repository: `https://github.com/ashreo/CapCutAPI`

Use either:

1. **MCP / stdio** — preferred when Hermes can register the local MCP server directly.
2. **HTTP REST** — use the local API server when MCP registration is unavailable or when a workflow is easier to drive through an HTTP client.

The upstream project documents draft creation/saving plus video, audio, image, text, subtitle, effect and sticker operations. It also exposes keyframe and duration tools through MCP.

Default upstream HTTP service: `http://127.0.0.1:9001` unless the local config overrides it.

### B — CapCut Mate (alternate local/network backend)

Repository: `https://github.com/Hommy-master/capcut-mate`

Useful when its FastAPI deployment model, request validation, additional draft operations or render route fits the machine better. Its documented development server exposes API docs at `http://localhost:30000/docs`.

### C — VectCutAPI (optional cloud-assisted backend)

Repository: `https://github.com/sun-guannan/VectCutAPI`

Use when cloud preview/render or CapCut/Jianying draft export is valuable. Keep it optional so Hermes does not make a cloud dependency mandatory for local/private work.

## Hermes MCP Registration Pattern

Do not commit user-specific absolute paths or secrets. On the machine running CapCutAPI, register a local MCP process using the installed path, following this shape:

```json
{
  "mcpServers": {
    "capcut-api": {
      "command": "python3",
      "args": ["mcp_server.py"],
      "cwd": "/ABSOLUTE/PATH/TO/CapCutAPI",
      "env": {
        "PYTHONPATH": "/ABSOLUTE/PATH/TO/CapCutAPI",
        "DEBUG": "0"
      }
    }
  }
}
```

The exact Hermes MCP configuration location must follow the installed Hermes version's MCP configuration mechanism. Never overwrite an existing MCP configuration blindly; inspect and merge.

## HTTP Adapter

This skill includes `tools/capcut_api_client.py`.

Environment:

```bash
export CAPCUT_API_BASE_URL=http://127.0.0.1:9001
```

Examples:

```bash
python skills/studio/cinematic-master-editor/tools/capcut_api_client.py check
python skills/studio/cinematic-master-editor/tools/capcut_api_client.py call /create_draft payload.json --dry-run
python skills/studio/cinematic-master-editor/tools/capcut_api_client.py call /create_draft payload.json --execute
```

The client deliberately accepts JSON payload files rather than pretending the upstream schema is frozen. Inspect the installed backend's current API docs/schema before generating a production payload.

## Edit Contract

Before calling CapCut, Hermes should have:

- `production-spec.yaml`
- `shot-list.csv`
- material/source manifest
- local or reachable media paths/URLs
- timeline order and target durations
- picture/audio/caption track plan
- transition/effect decisions only where motivated
- output aspect ratio and canvas dimensions

Then execute:

```text
production spec
  -> create draft
  -> add picture media in timeline order
  -> add dialogue/VO
  -> add music / ambience / SFX layers
  -> add captions / text / graphics
  -> add only approved transitions/effects/keyframes
  -> save draft
  -> open/inspect draft in CapCut or backend preview
  -> rough-cut review
  -> corrections
  -> final export/render through an explicitly supported route
  -> playback + technical QC
```

## Safety and Truth Boundary

1. **Draft creation is not final export.** Do not report a final video merely because a JSON draft exists.
2. **Do not claim headless export unless the selected backend/version supports and successfully executes it.** CapCut desktop UI behavior changes over time.
3. **Do not write directly into an unknown CapCut cache/draft directory without a backup and confirmed target path.** Prefer backend APIs that manage draft persistence.
4. **Never commit local CapCut paths, credentials, cookies or private media paths into the repository.**
5. **Use browser/UI automation only after API/MCP routes are unavailable or insufficient.** Record why the fallback was needed.
6. **Preserve the original draft before destructive edits.** New revisions should be separate drafts or backed-up project states.
7. **Validate current backend schemas before a paid/large batch.** A stale skill description must not override the installed API.

## Browser/Desktop Fallback

If the API can create the draft but cannot perform the required final desktop-only operation:

1. Save and checksum the draft.
2. Open the correct project in CapCut.
3. Verify timeline/media linkage visually.
4. Perform the minimum UI-only operation.
5. Export to a new output path.
6. Probe the exported media and play representative sections.
7. Keep the pre-UI draft as rollback.

Do not use UI automation to conceal a missing API capability; document the boundary.

## Acceptance Test

A machine is **CapCut-connected** only after all applicable checks pass:

- backend process starts;
- Hermes/MCP or HTTP client reaches it;
- backend reports expected editing tools or API docs;
- a disposable test draft can be created;
- one disposable media asset can be inserted;
- draft can be saved and reopened/inspected;
- no user production project was overwritten.

Until the disposable draft test passes on the target machine, report the integration as **implemented but not machine-verified**.
