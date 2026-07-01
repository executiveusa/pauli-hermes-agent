# GitHub Webhook Contract

## Endpoint

```text
POST /webhooks/github
```

## Required headers

- `X-GitHub-Event`
- `X-GitHub-Delivery`
- `X-Hub-Signature-256` when `GITHUB_WEBHOOK_SECRET` is configured

## Supported events

| Event | Action |
|---|---|
| `push` | Deploy only when `ref` resolves to an approved production branch. |
| `workflow_run` | Read-only diagnosis unless action/config explicitly allows deploy after CI success. |
| `deployment_status` | Verify URL and record state. |

## Rejected events

- Forked pull requests
- Branches outside production policy
- Payloads with invalid signatures
- Events with missing repository metadata

## Minimal push fields consumed

```json
{
  "repository": {"full_name": "owner/repo", "clone_url": "https://github.com/owner/repo.git"},
  "ref": "refs/heads/main",
  "after": "commit_sha"
}
```
