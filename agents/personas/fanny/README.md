# Agent Fanny Portable Persona

Agent Fanny is a tenant-isolated, exportable persona package for the Pi agent runtime. Hermes may spawn one or many Fanny instances, but Hermes does not become Fanny and Fanny does not inherit Hermes owner information.

## Product model

- **Template:** this directory contains the sellable, non-customer-specific persona.
- **Instance:** each customer receives a new `tenant_id` and `agent_instance_id`.
- **Configuration:** customer categories, examples, avatar, voice, data sources, authority, and retention are supplied at spawn time.
- **Learning:** corrections become quarantined lesson proposals; approved lessons remain inside that instance.
- **Export:** the persona template can be exported without runtime memory, credentials, customer data, or owner context.

## Spawn contract

Hermes or another authorized factory submits:

```json
{
  "template_id": "fanny",
  "tenant_id": "customer-uuid",
  "agent_instance_id": "instance-uuid",
  "human_owner": "customer-admin-id",
  "approved_data_sources": ["sandbox-upload"],
  "approved_categories": ["customer service", "trust"],
  "authority_profile": "assisted",
  "customer_brand": {},
  "avatar": {},
  "voice": {},
  "language_preferences": ["es-MX", "en"]
}
```

The factory must reject missing tenant identity, namespace collisions, owner-memory inheritance, unrestricted tools, or unapproved production access.

## Deployment modes

1. **Self-trained:** customer configures examples and corrections through guided onboarding.
2. **Assisted setup:** Pauli Effect configures categories, examples, tests, and approval boundaries.
3. **Managed training:** MAXX Operations reviews outcomes and proposes approved improvements.

## Required proof before production

- tenant isolation test;
- memory leakage test;
- tool allowlist test;
- approval-gate test;
- customer acceptance examples;
- export test proving no private data is included;
- rollback and suspension test.

## Export boundary

An export contains only the reusable persona package. Customer-specific training data should be exported separately under the customer's control and never merged into the generic Fanny template.
