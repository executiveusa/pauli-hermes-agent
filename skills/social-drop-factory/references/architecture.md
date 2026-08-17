# Social Drop Factory — Architecture Reference

## Purpose

Turn one reusable renderer into a multi-client campaign factory without locking production to a visual builder.

## System boundary

```text
INPUTS
  brand system
  weekly idea
  source copy
  media
  CTA
  campaign type
       ↓
CONTENT COMPILER
       ↓
MANIFEST
       ↓
SHARED RENDERER
       ↓
QUALITY / PARITY GATES
       ↓
STATIC OUTPUT
       ↓
DEPLOYMENT
       ↓
DISTRIBUTION
       ↓
ANALYTICS / CRM
       ↓
LEARNING FOR NEXT CAMPAIGN
```

## Canonical ownership

### GitHub
Owns:

- renderer
- reusable components
- client manifests
- campaign manifests
- tests
- deployment configuration
- architecture documentation
- version history

### Webflow
Use as an optional:

- visual R&D surface
- interaction laboratory
- builder-native editing surface
- client preview/review surface

Webflow must not become the only source of truth.

### Static hosting

The baseline should remain deployable to any normal static host, including Cloudflare Pages, GitHub Pages, Netlify, Vercel static hosting, conventional web servers, or compatible object/CDN hosting.

## Client structure

Preferred target:

```text
clients/<client>/
├── brand.json
├── assets/
└── campaigns/
    └── <campaign>.json
```

A campaign file should express content, not implementation details.

## Campaign types

Suggested `type` values:

- `weekly-social`
- `personalized-sales`
- `recruiting`
- `nonprofit-action`
- `local-service-lead`
- `product-launch`
- `creator-motivation`
- `event`
- `client-onboarding`

The renderer may select a composition from `type`, but shared primitives should remain reusable.

## Common manifest shape

```json
{
  "type": "weekly-social",
  "client": "asc3nd",
  "campaign": "2026-08-17-community-mentorship",
  "governingIdea": "Mentorship turns values into visible community action.",
  "brand": {
    "name": "ASC3ND",
    "tokens": {}
  },
  "week": {
    "monday": {"role": "belief"},
    "wednesday": {"role": "story"},
    "friday": {"role": "action"}
  },
  "drop": {
    "cards": [],
    "cta": {}
  },
  "tracking": {}
}
```

## Portability requirements

A campaign is portable when:

- business content is not hard-coded into renderer internals
- critical interactions do not require Webflow
- asset URLs can be replaced without rewriting layout code
- deployment does not require one vendor
- analytics can be swapped without replacing campaign content
- mobile behavior is defined and tested
- client-specific changes stay in manifest/assets/tokens when possible

## Asset pipeline

During parity work it can be useful to preserve the exact source URLs from Webflow to remove an uncontrolled variable.

After visual parity is proven, harden sovereignty:

```text
Drive / client upload
        ↓
asset ingest
        ↓
validate
        ↓
optimize WebP/AVIF where appropriate
        ↓
owned object storage or repo assets
        ↓
manifest references permanent asset location
```

Drive should be an intake source, not a permanent runtime requirement unless explicitly chosen.

## Quality gates

### Structural gate

- expected sections exist
- expected number of cards exists
- correct weekly roles exist
- correct CTA exists

### Responsive gate

Verify defined viewports, e.g.:

- desktop: 1440px
- tablet: 991px/1024px range
- mobile: 390px

### Visual gate

If parity is claimed, compare rendered screenshots against the reference. Structural parity is not pixel parity.

### Business gate

Before shipping, identify the one intended conversion event. If there is no measurable next action, the campaign is incomplete.

## Automation boundary

Safe to automate heavily:

- manifest generation
- content compilation
- image resizing/format conversion
- static builds
- linting
- parity assertions
- preview deployments
- analytics instrumentation
- variant generation

Require stronger review for:

- final visual approval
- sensitive claims
- client-specific facts
- publishing to live channels
- irreversible changes
- paid media spend
- outreach sends at scale

## Factory test

The architecture is proven when at least three materially different use cases render from the same engine without client-specific application forks, for example:

1. ASC3ND weekly social campaign
2. roofing personalized-sales Drop
3. recruiting Drop

If one of those requires a new codebase, identify why and extract the missing reusable primitive rather than accepting the fork by default.