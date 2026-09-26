# Five API opportunity test — 2026-08-13

These are research cards, not build authorization. The `proven-better-new` filter was used first; Hermes portfolio routing is applied separately at the end.

## 1. Contract Signal API
**Instinct:** Small government contractors want relevant opportunities surfaced early without spending hours in procurement portals.
**Primary analog:** GovTribe, serving federal contractors and GovCon teams.

**PROVEN**
- SAM.gov exposes searchable published contract opportunities; raw access already exists.
- GovTribe sells opportunity/award intelligence, saved searches, pursuit tools and integrations.
- Current GovTribe pricing demonstrates willingness to pay for workflow layered over public procurement data.
- Alerts, filtering and pursuit context are established mechanics that should not be reinvented blindly.

**BETTER**
- Test one capability profile -> ranked live opportunities -> clear match reasons.
- Deliver into email/CRM/webhook rather than requiring another workspace.
- Make time-to-first-useful-match the activation metric.
- Adjacent variant: white-label client monitoring for GovCon consultants instead of direct-to-contractor SaaS.

**NEW**
- AI bid-fit scoring is unproven; deterministic filters must still work when scoring fails.
- Generated next-actions require source grounding.
- API-only delivery for this audience is unproven.
- Consultant resale and upstream data-use terms require validation.

**Graveyard:** No directly comparable failed API-first GovCon product found in this bounded sweep; research gap remains.
**Atomic test:** Manually deliver a daily ranked feed to 5 target buyers for one week and ask for paid continuation.
**Sources:** https://open.gsa.gov/api/get-opportunities-public-api/ ; https://sam.gov/opportunities ; https://govtribe.com/plans

## 2. Grant Signal API
**Instinct:** Nonprofits and grant consultants want relevant funding opportunities delivered automatically instead of repeatedly searching fragmented sources.
**Primary analog:** Instrumentl, serving nonprofits and grant consultants.

**PROVEN**
- Grants.gov search and opportunity-fetch endpoints provide a public federal opportunity floor.
- Instrumentl uses organization/client profiles and project criteria for intelligent matching.
- Continuous matching and recurring notifications are proven retention mechanics.
- Tracking and integrations show that discovery must connect to a real workflow.

**BETTER**
- Push ranked matches into existing email/CRM/Notion/Slack workflows instead of another dashboard.
- Show deterministic eligibility reasons before semantic ranking.
- Support consultants managing multiple client profiles through one compact interface.
- Adjacent variant: embedded grant matching for nonprofit web/CRM agencies.

**NEW**
- Cross-source normalization is unproven until every source's terms and freshness are measured.
- Semantic matching can overstate eligibility; deterministic gates must come first.
- API-first delivery may fit consultants/software vendors better than end nonprofits.
- Grant-success claims are unsupported; test qualified opportunities and time saved instead.

**Graveyard:** No directly comparable failed API-first nonprofit grant-matching product found in this bounded sweep.
**Atomic test:** Give 3 grant consultants ranked matches for 5 client profiles each and charge per managed client if they continue.
**Sources:** https://www.grants.gov/api/api-guide ; https://grants.gov/api/common/search2 ; https://www.instrumentl.com/capability/discover ; https://help.instrumentl.com/en/articles/14794007-discover-plan

## 3. Corporate Filing Event API
**Instinct:** Professionals monitoring public companies want important filing events converted into structured, verifiable signals quickly.
**Primary analog:** sec-api.io, serving developers and professional teams that consume SEC filing data programmatically.

**PROVEN**
- SEC data APIs expose submissions and XBRL JSON without an API key.
- sec-api.io demonstrates a paid market for normalized SEC data and extraction APIs.
- Broad search, extraction, mapping and high-throughput access are mature incumbent mechanics.
- Redistribution rights are treated separately from internal use, so licensing cannot be assumed from public availability.

**BETTER**
- Focus on one event family rather than clone a broad filings platform.
- Include the filing, section/item and source evidence with every structured event.
- Use a simple webhook/API contract for one recurring monitoring job.
- Adjacent variant: specialized business-development or recruiting workflows rather than a general finance data product.

**NEW**
- Fine-grained AI extraction can fail silently; keep deterministic form/item filters as the floor.
- A narrow event taxonomy may not generate enough recurring value.
- Low-latency delivery creates an operations burden that must be measured.
- Buyer selection is load-bearing because each profession values different events.

**Graveyard:** No directly comparable failed narrow-event API found in this bounded sweep.
**Atomic test:** Pick one event type and one professional audience; manually deliver 20 source-grounded events and seek a paid pilot.
**Sources:** https://www.sec.gov/search-filings/edgar-application-programming-interfaces ; https://sec-api.io/pricing

## 4. Permit Trigger API
**Instinct:** Businesses selling into construction want to contact the right company or property when a real project starts, not months later from stale lists.
**Primary analog:** Shovels, serving property, climate, construction and GTM teams.

**PROVEN**
- Chicago exposes machine-readable building permit data through its public data API.
- Shovels normalizes permit/contractor data and explicitly supports sales, integrations and market research.
- PermitGrab and PermitDrop show paid monthly demand for permit-triggered lead products.
- Freshness, coverage, filtering and normalization are core mechanics.

**BETTER**
- Start with one metro + one trade instead of pretending to have national coverage.
- Deliver only high-intent triggers into the buyer's current workflow.
- Include source, freshness and coverage metadata with every result.
- Adjacent variant: territory feeds for agencies/software vendors serving contractors.

**NEW**
- Cross-jurisdiction normalization is load-bearing if expansion happens too early.
- Contact enrichment adds privacy/licensing/outreach complexity; prove value without it first.
- Generated outreach copy is secondary and unproven.
- Public access does not automatically grant unrestricted redistribution rights.

**Graveyard:** None identified; generic permit aggregation is already crowded.
**Atomic test:** One metro + one trade, 25 verified fresh triggers, then ask for paid monthly continuation.
**Sources:** https://dev.socrata.com/foundry/data.cityofchicago.org/iiwr-py22 ; https://docs.shovels.ai/docs/shovels-api-introduction ; https://www.shovels.ai/ ; https://permitgrab.com/pricing ; https://www.permitdrop.com/pricing

## 5. Website Performance Action API
**Instinct:** Agencies and site owners want performance problems translated into prioritized fixes without repeatedly interpreting raw audit output.
**Primary analog:** WebPageTest paid API, serving companies and developers running performance tests and integrations.

**PROVEN**
- Google's PageSpeed Insights API can analyze a URL and return scores and optimization information.
- Google explicitly supports integrating PageSpeed analysis into development workflows.
- WebPageTest sells API access and recurring performance testing, proving paid demand for automated web-performance data.
- Performance budgets, scheduled tests and integrations are established mechanics.

**BETTER**
- Normalize raw audit output into the 3 highest-impact actions for a specific stack such as WordPress/Shopify/Next.js.
- Track before/after evidence so agencies can prove improvement to clients.
- Deliver an agency-friendly client report plus machine-readable API rather than raw Lighthouse JSON.
- Adjacent variant: white-label QA endpoint for small web studios to run before every client handoff.

**NEW**
- Turning audit findings into stack-specific fixes is the unproven value layer.
- Automated fix recommendations may be wrong; source metrics and confidence must remain visible.
- Generic performance scoring is commoditized; the product must own a narrow workflow.
- Google notes changes to PageSpeed/CrUX behavior over time, so upstream contract drift must be monitored.

**Graveyard:** No directly comparable failed action-oriented micro-API found in this bounded sweep.
**Atomic test:** Run 10 agency sites, produce the top 3 prioritized fixes plus before/after proof format, and ask 5 agencies to pay for recurring client QA.
**Sources:** https://developers.google.com/speed/docs/insights/v5/reference/ ; https://developers.google.com/speed/docs/insights/v5/get-started ; https://product.webpagetest.org/

# Separate Hermes portfolio routing
- **SELL:** Contract Signal API — strong paid analog, public upstream, narrow consultant/contractor wedge.
- **SELL:** Grant Signal API — strong matching analog and direct consultant/nonprofit workflow fit.
- **SELL:** Corporate Filing Event API — strong paid API analog if kept to one event + one buyer.
- **SELL:** Permit Trigger API — multiple paid lead/data analogs; start geographically bounded.
- **SELL:** Website Performance Action API — cheap upstream and easy paid agency test, but differentiation must be stack-specific.

Only one becomes the active bounded experiment. The other four remain queued until the active experiment proves or fails.
