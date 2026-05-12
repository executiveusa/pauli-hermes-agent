# Indigo Azul — Retrieval Rules

## Memory System: Supabase + pgvector

All queries must filter on required tags before semantic search.

## Required Tag Filters

```python
base_filter = {
    "project": "indigo_azul",
    "org": "new_world_kids"
}
```

Always apply `base_filter` before any semantic or keyword search.

## Domain-Scoped Retrieval

| Task | domain filter | top_k |
|------|--------------|-------|
| Construction status | construction | 10 |
| Donor lookup | fundraising | 5 |
| Child story search | education | 5 |
| Campaign research | fundraising | 8 |
| Compliance/reporting | ops | 10 |

## Retrieval Priority Order

1. **Exact match** — ID lookup, always preferred
2. **Semantic search** — pgvector cosine similarity (threshold: 0.75)
3. **Keyword fallback** — ILIKE on title/body fields
4. **Graph traversal** — follow entity relationships (donor → campaign → child)

## Freshness Rules

| Entity | Max Age Before Refresh |
|--------|----------------------|
| Donation totals | 1 hour |
| Child outcomes | 24 hours |
| Campaign stats | 6 hours |
| Construction milestones | 24 hours |
| Donor strength scores | 7 days |

## Embedding Model

- Model: `text-embedding-3-small` (OpenAI) or equivalent
- Dimensions: 1536
- Fields to embed: title + body_text + tags joined as string

## Chunk Strategy

- Stories: full document (≤2000 tokens)
- Donor profiles: summary block only
- Construction reports: chunked by milestone
- Campaigns: full document

## Never Retrieve Without Tags

Any raw `SELECT *` without tag filters is prohibited.
Always scope to `project=indigo_azul` minimum.
