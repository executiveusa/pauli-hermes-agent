# Deploy to Hostinger + Coolify

1. Build image from repository `Dockerfile`.
2. Configure Coolify service with persistent volume for `HERMES_HOME`.
3. Inject runtime env from managed secret provider (Infisical target state).
4. Expose gateway/dashboard ports behind TLS reverse proxy.
5. Validate `/health` and messaging/webhook reachability.
