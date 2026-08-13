# Niche → Awwwards Category Mapping

Maps business niches to Awwwards scrape targets and styles.refero.design categories.
When a niche is missing, add it here before running the scraper.

| Niche | Awwwards Category | Refero Category | Kupuri Projects |
|-------|-------------------|-----------------|-----------------|
| hospitality-luxury | hospitality | hospitality | Verified Vallarta™ |
| ai-agency | agency | tech-startup | COSMOS Studio |
| anime-studio | entertainment | creative-studio | AFROMATIONS STUDIOS |
| smb-migration | e-commerce | small-business | LANE Intelligence |
| casino-gaming | entertainment | gaming | Foxies Casino |
| coffee-brand | food-beverage | cpg | COSMOS UGC demo |
| documentary-film | film | documentary | Puerto Rico doc |
| tech-faceless-youtube | media | content-creator | COSMOS YouTube |
| real-estate | real-estate | real-estate | General |
| healthcare | healthcare | healthcare | General |
| restaurant-food | food-beverage | restaurant | General |
| fashion-luxury | fashion | luxury | General |
| fintech | fintech | finance | General |
| saas-b2b | saas | b2b | General |
| portfolio-creative | portfolio | creative | General |

## Scrape Schedule (per niche rotation)

The cron job rotates through 3 niches per night to stay within scraping limits:
- Mon/Thu: hospitality-luxury, ai-agency, anime-studio
- Tue/Fri: smb-migration, casino-gaming, coffee-brand
- Wed/Sat: documentary-film, tech-faceless-youtube, real-estate
- Sun: healthcare, restaurant-food, fashion-luxury
