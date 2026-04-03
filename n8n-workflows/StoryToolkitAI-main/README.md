# n8n Workflow Templates

Import these workflows into your n8n instance on Hostinger.

## Workflows

### 1. Documentary Processing
**File:** `documentary-processing.json`
- **Trigger:** Webhook
- **Function:** Process documentary videos via orchestrator
- **Path:** `/webhook/documentary-webhook`

### 2. GitHub → Notion Sync
**File:** `github-notion-sync.json`
- **Trigger:** Schedule (Daily at 2 AM)
- **Function:** Sync all GitHub repos to Notion database

### 3. MCP Health Check
**File:** `mcp-health-check.json`
- **Trigger:** Schedule (Every 15 minutes)
- **Function:** Monitor HTTP MCP servers and update health status

### 4. Skills Filesystem Sync
**File:** `skills-filesystem-sync.json`
- **Trigger:** Schedule (Every 6 hours)
- **Function:** Scan .claude/skills directory and sync to Supabase

## Setup Instructions

### 1. Import Workflows

1. Open n8n at your Hostinger URL
2. Click "Import from File"
3. Upload each JSON file
4. Click "Save"

### 2. Configure Credentials

Create these credentials in n8n:

**GitHub PAT** (httpHeaderAuth)
- Header Name: `Authorization`
- Value: `Bearer YOUR_GITHUB_PAT`

**Notion API** (notionApi)
- API Key: `YOUR_NOTION_API_TOKEN`

**Supabase Postgres** (postgres)
- Host: `db.sbbuxnyvflczfzvsglpe.supabase.co`
- Database: `postgres`
- User: `postgres`
- Password: `YOUR_SUPABASE_PASSWORD`
- Port: `5432`
- SSL: `true`

### 3. Set Environment Variables

In n8n settings, add these environment variables:

```
BFF_API_URL=https://pauli-effect-bff.your-subdomain.workers.dev
NOTION_DATABASE_ID=your-notion-database-id
```

### 4. Update File Paths

In each workflow, update file paths to match your Hostinger setup:
- `/path/to/pauli-effect/` → Your actual project path
- `/path/to/.claude/skills/` → Your actual skills directory

### 5. Activate Workflows

1. Open each workflow
2. Click "Active" toggle at top right
3. Test each workflow manually first

## Testing

### Documentary Processing
```bash
curl -X POST https://your-n8n.com/webhook/documentary-webhook \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "test-job-id",
    "video_source": "gs://bucket/video.mp4",
    "config": {}
  }'
```

### GitHub Sync
Click "Execute Workflow" in n8n editor

### MCP Health Check
Will run automatically every 15 minutes

### Skills Sync
Will run automatically every 6 hours

## Monitoring

- Check execution logs in n8n UI
- Monitor Supabase tables for updates
- Check BFF API logs for errors

## Troubleshooting

**"Command not found"**
- Install required tools on Hostinger server (python, gsutil, ffmpeg)

**"Connection refused"**
- Check BFF_API_URL is correct
- Verify Cloudflare Workers is deployed

**"Database error"**
- Verify Supabase credentials
- Check table schemas match supabase_schema.sql
