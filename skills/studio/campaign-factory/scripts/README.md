# Campaign Factory QR generator

## Install

```bash
python -m pip install -r skills/studio/campaign-factory/requirements.txt
```

## Generate

```bash
python skills/studio/campaign-factory/scripts/generate_qr.py \
  --url https://asc3nd.org \
  --campaign "Community Cuts for Kids" \
  --output ./campaign-output/community-cuts/qr
```

The command creates an SVG master, PNG and PDF derivatives, SHA-256 checksums, and a JSON payload receipt.

Generation is not scan verification. The receipt deliberately remains `GENERATED_NOT_SCAN_VERIFIED` until a later software decode, physical-device scan, and final-composition scan are completed and recorded.
