# Secret Env Source

## Source Of Truth

Use `E:\THE PAULI FILES\master.env` as the local secret source of truth for this workstation.

## Sync Commands

PowerShell:

```powershell
.\scripts\pauli\sync-env-to-hermes.ps1 -EnvFile "E:\THE PAULI FILES\master.env"
```

WSL:

```bash
bash scripts/pauli/sync-env-to-hermes.sh "/mnt/e/THE PAULI FILES/master.env"
```

## Notes

- The sync scripts copy values into Hermes runtime env files without printing them.
- Do not commit the source env file or the generated Hermes `.env` files.
- Browser and Hostinger workflows should read secrets from the runtime env, not from chat.
