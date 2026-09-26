# Secret Handling

## Never print

- GitHub tokens
- Vercel tokens
- Environment variable values
- Webhook secrets
- Bypass tokens
- OAuth credentials
- API keys

## Redaction policy

Any value matching these patterns must be replaced with `[REDACTED]`:

- `ghp_`, `github_pat_`, `gho_`, `ghu_`
- `vercel_`
- `sk-`
- `xoxb-`, `xapp-`
- strings longer than 32 characters assigned to secret-like variable names

## Permitted secret operations

- Check if a variable exists.
- Pull `.env.local` for local diagnosis if the file is gitignored.
- Compare required variable names against Vercel project env names.

## Forbidden secret operations

- Echo values.
- Commit `.env*` files.
- Send values to another agent.
- Store values inside run reports.
