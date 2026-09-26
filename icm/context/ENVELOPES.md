# Worker and Judge Envelopes

## Worker request
```yaml
profile_id: plan|score|implement|write_short|write_long|judge|test|docs
system_prompt_path: string
user_prompt_path: string
expected_output: string
correlation_id: string
```

## Worker result
```yaml
correlation_id: string
status: complete|blocked|failed
artifact_refs: []
evidence_refs: []
cost: null
risks: []
rollback: null
```

## Judge result
```yaml
verdict: accept|reject|halt
evidence_refs: []
reason: string
largest_gap: null
fixes: []
human_decision: null
```

Builders never self-approve. Missing evidence prevents acceptance.
