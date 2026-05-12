# Workflow: Donor Update

## Trigger
- Monthly cadence (scheduled)
- Post-milestone (event-triggered)
- Post-donation (immediate)

## Steps

1. **Retrieve donor list**
   - Query: all donors with `last_contact_at < 30 days ago` OR `donation in last 7 days`
   - Tag filter: `project=indigo_azul, domain=fundraising`

2. **Pull recent impact data**
   - Children served (count + names if consent given)
   - Construction milestone reached
   - Program outcomes logged this period

3. **Select story**
   - Call `content_engine`: retrieve most recent unpublished story
   - Prefer stories linked to this donor's campaign if possible

4. **Generate update**
   - Tone: warm, specific, non-generic
   - Must include: 1 real child outcome OR 1 construction milestone
   - Must include: total impact to date for this donor
   - Length: 150–300 words for email, 50–100 for SMS

5. **Log gratitude**
   - Insert `GratitudeLog` record
   - Tag: `trigger=MILESTONE` or `trigger=DONATION`

6. **Queue for approval**
   - Surface draft to human for review before send
   - Approval gate: `EXTERNAL_PUBLISH`

7. **Send** (post-approval)
   - Channel: donor's `preferred_channel`
   - Update `last_contact_at`

## Output
- Draft message (Markdown)
- Gratitude log entry
- Updated donor record
