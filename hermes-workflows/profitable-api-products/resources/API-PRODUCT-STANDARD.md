# API Product Standard

This is Hermes' stable specification/verification standard. Builders implement it; Hermes uses it to brief, inspect, test, and judge.

## 1. Choose the smallest interface
Prefer, in order:
1. existing owned native connector;
2. Composio toolkit/session;
3. Composio extension tool or authenticated proxy call for a missing provider action;
4. existing OpenAPI, GraphQL, or MCP exposed to Hermes through `mcp2cli`;
5. a narrow custom wrapper API;
6. a new underlying service only when the customer value cannot be delivered by the earlier options.

Expose additional surfaces only when a customer/job requires them:
- REST/OpenAPI for programmatic customers;
- MCP for agent-native customers;
- CLI for deterministic human/agent operations and debugging;
- webhook for event-driven delivery.
Do not create API + MCP + CLI merely for completeness.

## 2. API key and auth baseline
For long-lived customer API keys:
- generate secrets server-side;
- display the raw secret once;
- store only a one-way hash plus a safe short prefix/identifier;
- support immediate revocation and rotation;
- enforce tenant/account ownership in the database, not only in UI code;
- apply quotas/rate limits at the customer/account level, not merely per key;
- keep privileged service credentials server-side only;
- transmit secrets only over TLS;
- never put raw secrets in source control, logs, prompts, analytics, URLs, screenshots, or command history;
- prefer scoped permissions and separate dev/staging/prod credentials.

For OAuth/SaaS auth, prefer Composio's managed auth when it already covers the provider and preserves Pauli's Place tenant/approval model.

## 3. HTTP/product design baseline
- resource nouns and consistent naming;
- version from day one (`/v1/...` or equivalent contract version);
- accurate status codes, including auth, not-found, rate-limit, validation, and server failures;
- bounded pagination; server-side filters/sorts for high-cardinality data;
- schema validation and parameterized database access;
- least privilege, strict tenant isolation, and explicit CORS policy;
- idempotency where retries could duplicate side effects;
- request IDs and useful structured logs without secrets;
- declared freshness/source timestamps for data products.

## 4. Source and resale diligence
Before a worker builds:
- identify authoritative upstream source(s);
- record API/data license, terms, quotas, attribution, redistribution/resale restrictions, and freshness;
- distinguish public access from permission to resell/redistribute;
- define graceful behavior when upstream is unavailable or changes shape;
- verify the value layer is more than a transparent pass-through when the market already has the raw source.

## 5. Commercial contract
Every build brief must name:
- target buyer and recurring job;
- proven audience-matched analog and current pricing/mechanics;
- smallest paid outcome;
- proposed price and usage limit;
- variable cost per unit/request/customer;
- support burden and gross-margin target;
- acquisition channel and activation event;
- stop condition if buyer evidence does not appear.

Start with the thinnest key-vending/onboarding method appropriate to the test. A manual key can be acceptable for the first few paid pilots; automate only after demand warrants it.

## 6. Deployment/VPS acceptance
The worker must provide evidence for:
- reproducible deploy from source;
- environment separation and secret injection;
- HTTPS/domain routing;
- process/container supervision and health check;
- logs and basic metrics;
- backup/restore where state exists;
- database migrations with rollback or forward-repair path;
- resource/cost ceiling and rate limiting;
- rollback to prior known-good release.
Hermes may inspect and test these controls, but implementation belongs to the assigned worker.

## 7. Docs are onboarding
A customer must be able to reach first success with one copy/paste example. Minimum docs:
- one working curl example;
- auth header/key placement;
- real response shape;
- errors and limits;
- pagination/filter examples where relevant;
- OpenAPI or equivalent machine-readable contract for public REST products;
- changelog/version policy;
- support/contact path.

Activation proof is a real request against the deployed endpoint returning the expected result and a deliberately invalid credential returning the expected rejection.

## 8. Independent proof checklist
A verifier other than the builder checks:
1. paid-problem evidence exists;
2. upstream rights/terms are recorded;
3. one happy-path request works in the deployed environment;
4. invalid/missing auth fails correctly;
5. cross-tenant access is denied;
6. customer-level rate/usage limits cannot be bypassed by minting another key;
7. secrets are absent from client bundle, repository, logs, and receipts;
8. malformed input fails safely;
9. upstream outage/degradation has defined behavior;
10. docs produce first success from a cold client;
11. variable-cost and quota assumptions are measured;
12. rollback is documented and testable.

No score or CI pass substitutes for these receipts.

## 9. Composio boundary
Composio is an integration fabric, not the Pauli's Place control plane. Use its sessions/toolkits for broad SaaS connectivity; use extension tools or authenticated proxy execution when a provider endpoint is missing. Keep tenant identity, mission authority, approvals, budgets, auditing, and outcome evidence in Pauli's Place. For production applications, use Composio's SDK/API patterns rather than treating a developer CLI as the product runtime.
