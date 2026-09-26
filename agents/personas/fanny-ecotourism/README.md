# Fanny Ecotourism

Portable bilingual digital-worker template for ecotourism, hospitality, tours, cultural experiences, and Hispanic-owned small businesses.

## Strategic markets

### Puerto Vallarta

- eco hotels and boutique stays;
- nature, food, cultural, wellness, and community experiences;
- bilingual visitor communication;
- content that respects local capacity, culture, wildlife, and seasonality.

### Mexico City

- cultural and food experiences;
- travel planners and destination businesses;
- businesses that can refer or package experiences with Jalisco and Washington State.

### Everett, Seattle, and Skagit County

- Mexican and Hispanic-owned businesses;
- visiting-friends-and-relatives travel;
- cross-border cultural, family, food, and tourism relationships;
- Spanish-first operators who need a nontechnical voice interface.

## Sellable outcome

Fanny listens to the owner’s idea, challenges unsupported assumptions, organizes the idea into a measurable campaign, prepares bilingual content, creates Postiz drafts, requests approval, and reports results.

She is not sold as a generic chatbot. She is sold as a voice-first growth partner with a narrow, measurable workflow.

## Default first product

A two-week assisted campaign pilot:

1. voice onboarding with Grill Me;
2. define one bookable offer;
3. identify one audience and one destination corridor;
4. establish proof and prohibited claims;
5. connect approved Postiz channels;
6. produce a seven-post bilingual draft calendar;
7. human review;
8. schedule approved posts;
9. measure inquiries, bookings, saves, shares, and qualified conversations;
10. recommend stop, revise, or expand.

## Voice architecture

### Recommended demo stack

Use ElevenLabs Agents for the first impressive browser demo because it provides integrated speech recognition, language model orchestration, text-to-speech, interruption handling, turn taking, knowledge, workflows, and web deployment.

Use the Pi/Hermes control API as the custom brain and tool layer when ready. Keep the voice provider replaceable through an adapter.

### Alternative

A Voicebox adapter may be used when the customer already has an approved Voicebox service with an accessible API or SDK. Do not bind the persona contract to an undocumented provider.

### Production direction

The long-term interface should expose:

```text
startSession
streamAudio
receiveTranscript
sendAgentText
receiveAgentAudio
interrupt
endSession
```

No persona logic belongs in the voice adapter.

## Postiz architecture

Use Postiz MCP for agent-native tool discovery when available. Use OAuth2 for customer-specific accounts. Use the CLI or public API for server workflows.

Default permissions:

- list integrations;
- inspect schemas;
- read analytics;
- create drafts;
- upload approved media.

Scheduling, publishing, deleting, account connection, or settings changes require human approval.

## Prompt enhancement

The owner may speak informally. Fanny creates an internal campaign brief but must preserve intent and show the resulting summary before external action.

```text
RAW IDEA
→ INTENDED OUTCOME
→ CUSTOMER
→ OFFER
→ EVIDENCE
→ ASSUMPTIONS
→ CHANNEL
→ CONTENT ANGLE
→ SMALLEST TEST
→ METRIC
→ HUMAN CONFIRMATION
```

## Training modes

- Self-trained: customer uses the voice onboarding and approved examples.
- Assisted: Pauli Effect configures persona, channels, metrics, and first campaign.
- Managed: recurring review, corrections, analytics, and new skills.
- MAXX Operations: infrastructure, integrations, monitoring, content operations, and ongoing optimization.

## Isolation

Each customer receives a separate tenant, agent instance, memory namespace, Postiz OAuth identity, content calendar, approval policy, analytics history, and export bundle.

Customer learning never enters another customer’s Fanny instance without explicit anonymization, review, and authorization.
