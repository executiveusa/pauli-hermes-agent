# Pricing and Cost Snapshot

Last verified: 2026-08-04

This file is a planning snapshot. Prices and terms can change. Hermes must verify the official source immediately before purchase.

## 21st.dev

Official pricing page: `https://21st.dev/pricing`

- Free access: browsing, previews, search, discovery, and two component copies per day for signed-in users.
- Builder: $8/month billed quarterly or $6/month billed yearly.
- Builder + AI: starts at $20/month billed quarterly or $15/month billed yearly, depending on included monthly AI credits.
- Additional AI credits: 100 credits for $5; purchased credits roll over.
- Team: $7.50 per seat/month billed yearly before optional AI credits.
- Payments are described as non-refundable.

Hermes should begin with the free allowance for verification. Do not purchase a plan until actual component-install volume or AI-generation demand justifies it.

## QVAC

QVAC is treated as a local runtime dependency. Repository installation may be free, but local use still has costs:

- model storage;
- electricity;
- SSD wear and capacity;
- setup and maintenance time;
- optional model or provider licensing;
- stronger worker hardware when the Surface cannot run the selected workload.

Do not equate open-source software with zero operating cost.

## Moondream

Moondream can be evaluated as a local specialist vision backend. Hermes must verify current runtime, model license, account/API-key requirements, and hosted-service pricing from official Moondream sources before installation or purchase.

The default Surface trial must prohibit silent cloud fallback.

## Local model cost framework

For each candidate, report:

| Cost | Required evidence |
| --- | --- |
| Model download | exact model, quantization, expected bytes |
| Runtime storage | weights + cache + temporary working space |
| Media derivatives | frame, proxy, transcript, embedding growth per hour of footage |
| Compute | measured time and power profile on target machine |
| Subscription | official current price and billing interval |
| Client maintenance | update, backup, monitoring, and support scope |

## Purchase gate

Before spending money, Hermes reports:

```text
NEED
FREE ALTERNATIVE TESTED
ONE-TIME COST
RECURRING COST
CANCELLATION/REFUND TERMS
OWNER OF ACCOUNT
EXPECTED COMMERCIAL RETURN
ROLLBACK
HUMAN APPROVAL
```
