# Indigo Azul — Data Schema

## Core Entities

### Person
```
id, name, role, email, phone
type: CHILD | DONOR | PARTNER | STAFF | VOLUNTEER
tags: []
created_at, updated_at
```

### Child
```
extends: Person
program_id, enrollment_date
outcomes: []        -- measured outcomes
story_ids: []       -- linked stories
status: ACTIVE | GRADUATED | PENDING
```

### Donor
```
extends: Person
total_donated, currency
donation_ids: []
preferred_channel: ZEFFY | BTCPAY | CHECK
last_contact_at
strength: ACTIVE | WARM | FADING   -- same decay model as Rolodex
```

### Donation
```
id, donor_id, amount, currency, platform
campaign_id         -- which campaign drove this
timestamp, confirmed
tx_hash             -- for crypto donations
```

### Campaign
```
id, title, type: FUNDRAISING | AWARENESS | GRATITUDE
goal_amount, raised_amount, currency
start_date, end_date
content_ids: []     -- linked stories / posts
status: DRAFT | ACTIVE | CLOSED
```

### Program
```
id, name, type: CONSTRUCTION | EDUCATION | COMMUNITY
phase, budget, spent
children_enrolled: int
outcomes_tracked: []
start_date, target_end_date
```

### Story
```
id, child_id, program_id
title, body_text
media_urls: []
campaign_ids: []    -- stories linked to campaigns
published_at, platform
```

### ConstructionMilestone
```
id, program_id
title, description
budget_allocated, budget_spent
status: PLANNED | IN_PROGRESS | COMPLETE | BLOCKED
completion_date
photos: []
```

### GratitudeLog
```
id, donor_id, trigger: DONATION | MILESTONE | BIRTHDAY | ANNIVERSARY
message_sent, channel, sent_at
outcome_referenced: bool
```

## Memory Tags Schema

Every Supabase record must include:
```yaml
_tags:
  project: indigo_azul
  org: new_world_kids
  domain: construction | education | fundraising | ops
  entity_type: <entity name>
```

## Graph Relationships

```
Donor ──donated_to──► Campaign ──funds──► Program
Program ──serves──► Child ──has──► Outcome ──generates──► Story
Story ──drives──► Campaign
Campaign ──triggers──► GratitudeLog ──reaches──► Donor
```
