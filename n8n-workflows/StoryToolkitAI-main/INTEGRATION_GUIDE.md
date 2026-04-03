# POLLY CHARACTER SYSTEM — INTEGRATION GUIDE

## 🎨 WHAT YOU HAVE

A **production-ready character system** for Polly (anthropomorphic mischievous sheep in black & white gritty ink style) that generates consistent character art for your Pauli Effect delivery funnel.

### Files Provided:
1. **prompt.master.txt** - Master character definition (locked)
2. **prompt-api-schema.json** - Complete JSON schema for React integration
3. **PollyAsset.jsx** - React component for rendering Polly
4. **api-polly-generate.js** - Backend API endpoint for generation
5. **This file** - Integration guide

---

## ⚡ QUICK START (5 MINUTES)

### Step 1: Copy Files to Your Project

```bash
# Copy Polly system to your project
cp -r polly/ your-pauli-effect/brand/polly/

# Or in your Next.js project:
# - Copy PollyAsset.jsx → pages/components/PollyAsset.jsx
# - Copy api-polly-generate.js → pages/api/polly/generate.js
# - Copy prompt.master.txt → public/brand/polly/prompt.master.txt
# - Copy prompt-api-schema.json → public/brand/polly/schema.json
```

### Step 2: Set Up Image Generation Provider

Choose one (Hugging Face, Replicate, or custom):

**Option A: Hugging Face (Recommended)**
```bash
# Install Hugging Face inference SDK
npm install @huggingface/inference

# Add API key to .env.local
echo "HF_API_KEY=your_key_here" >> .env.local
```

**Option B: Replicate**
```bash
npm install replicate

echo "REPLICATE_API_TOKEN=your_token" >> .env.local
```

### Step 3: Update API Endpoint

Edit `pages/api/polly/generate.js` and uncomment your chosen provider:

```javascript
// Example for Hugging Face:
import { HfInference } from "@huggingface/inference";

async function generatePollyImage(prompt, width, height) {
  const hf = new HfInference(process.env.HF_API_KEY);
  const image = await hf.textToImage({
    inputs: prompt,
    model: "stabilityai/stable-diffusion-2-1"
  });
  // Upload to blob storage
  const url = await uploadToVercelBlob(image);
  return { url, format: 'png' };
}
```

### Step 4: Use in React Components

```jsx
import PollyAsset from '@/components/PollyAsset';

// In your delivery funnel:
export default function DeliveryPresentation() {
  return (
    <div>
      {/* Slide 1: Hero with Polly */}
      <PollyAsset 
        pack="delivery" 
        sceneId="dispatch"
        width={512}
        height={512}
      />
      
      {/* Slide 2: Polly celebrating */}
      <PollyAsset 
        pack="delivery" 
        sceneId="success"
        showPrompt={false}
      />
      
      {/* Or use quick components */}
      <PollyDeliverySuccess />
    </div>
  );
}
```

### Step 5: Test

```bash
npm run dev

# Visit http://localhost:3000/delivery
# Check console for Polly asset loading
# Verify images render correctly
```

---

## 🎬 SCENE PACKS AVAILABLE

### **Delivery Pack** (5 scenes)
- `delivery_dispatch` - Polly at dispatch desk
- `delivery_receipt` - Polly holding pizza box
- `delivery_night_run` - Polly delivering at night
- `delivery_customer_meeting` - Polly at customer's door
- `delivery_success` - Polly celebrating

### **Pauli Effect Pack** (3 scenes)
- `pauli_effect_presentation` - Polly presenting funnel
- `pauli_effect_referral` - Polly with golden coin
- `pauli_effect_wally_vs` - Polly defeating WordPress

### **Lifestyle Pack** (5 scenes)
- `lifestyle_alley` - Polly scheming in alley
- `lifestyle_cool` - Polly leaning cool
- `lifestyle_shocked` - Polly surprised
- `lifestyle_thinking` - Polly contemplating
- `lifestyle_playful` - Polly being playful

### **Promotional Pack** (4 scenes)
- `promo_hero` - Polly heroic on rooftop
- `promo_banner` - Polly peeking from banner
- `promo_action` - Polly in dynamic action
- `promo_mystery` - Polly mysterious in shadows

---

## 💡 USAGE EXAMPLES

### Example 1: Add Polly to Slide 1 (Opportunity)

**Before:**
```jsx
export function OpportunitySlide({ onView }) {
  return (
    <div className="flex flex-col items-center justify-center h-full">
      <h2>This is Exclusive</h2>
      <p>You were selected...</p>
    </div>
  );
}
```

**After:**
```jsx
import PollyAsset from '@/components/PollyAsset';

export function OpportunitySlide({ onView }) {
  return (
    <div className="flex flex-col md:flex-row items-center justify-center h-full gap-8">
      {/* Left: Polly */}
      <div className="w-full md:w-1/2">
        <PollyAsset 
          pack="delivery"
          sceneId="dispatch"
          width={400}
          height={400}
          className="rounded-lg shadow-lg"
        />
      </div>
      
      {/* Right: Content */}
      <div className="w-full md:w-1/2">
        <h2>This is Exclusive</h2>
        <p>You were selected...</p>
      </div>
    </div>
  );
}
```

### Example 2: Custom Scene (Dynamic Personalization)

```jsx
<PollyAsset
  pack="lifestyle"
  sceneId="cool"
  customScene={`Polly presenting a personalized offer for ${clientName}`}
  customMood="looking confident and sly"
  width={512}
  height={512}
/>
```

### Example 3: Banner Image for Email Campaign

```jsx
<PollyAsset
  pack="promotional"
  sceneId="hero"
  width={1200}
  height={400}
  altText="Pauli Effect delivery hero image"
  cacheKey="email-banner-jan-2025"
/>
```

### Example 4: Lazy Loading with Cache

```jsx
<PollyAsset
  pack="delivery"
  sceneId="success"
  onLoad={(data) => {
    console.log(`Polly loaded. Cached: ${data.cached}`);
    // Track in analytics
    trackEvent('polly_loaded', { cached: data.cached });
  }}
  showPrompt={process.env.NODE_ENV === 'development'}
/>
```

---

## 🔧 ADVANCED USAGE

### Custom Character Prompt

If you want to override the master prompt entirely:

```jsx
import { buildPollyPrompt } from '@/components/PollyAsset';

const customPrompt = buildPollyPrompt(
  {
    scene: "Polly riding a motorcycle through neon city",
    position: "dominating the entire scene",
    mood: "intense and focused"
  }
);

<PollyAsset
  customScene="Polly riding a motorcycle through neon city"
  customPosition="dominating the entire scene"
  customMood="intense and focused"
/>
```

### Batch Generation (Multiple Polly Assets)

```jsx
const scenes = [
  { pack: 'delivery', id: 'dispatch' },
  { pack: 'delivery', id: 'success' },
  { pack: 'lifestyle', id: 'cool' },
];

export function PollyGallery() {
  return (
    <div className="grid grid-cols-3 gap-4">
      {scenes.map((scene) => (
        <PollyAsset
          key={`${scene.pack}_${scene.id}`}
          pack={scene.pack}
          sceneId={scene.id}
          width={256}
          height={256}
        />
      ))}
    </div>
  );
}
```

### Performance: Pre-generate & Cache

```jsx
// generateStaticProps for pre-rendering
export async function getStaticProps() {
  // Pre-generate all Polly assets at build time
  const scenes = [
    'delivery_dispatch',
    'delivery_success',
    'lifestyle_cool',
    'promo_hero'
  ];

  // Call API for each scene
  // Results cached with long TTL

  return {
    props: { preGenerated: true },
    revalidate: 2592000 // 30 days
  };
}
```

---

## 🎯 INTEGRATION POINTS FOR DELIVERY FUNNEL

### Slide 0: YOU GOT SENT FOR
```jsx
<PollyAsset pack="delivery" sceneId="dispatch" />
// Or custom: "Polly saying 'You got sent for, {name}'" in a dramatic pose
```

### Slide 1: THE OPPORTUNITY
```jsx
<PollyAsset pack="pauli_effect" sceneId="presentation" />
// Polly as your personal guide/mentor
```

### Slide 2: THE VALUE
```jsx
<PollyAsset pack="pauli_effect" sceneId="wally_vs" />
// Polly defeating WordPress bloat
```

### Slide 3: THE REFERRAL
```jsx
<PollyAsset pack="pauli_effect" sceneId="referral" />
// Polly with golden coin, celebrating earnings
```

### Slide 4: THE FINISH
```jsx
<PollyAsset pack="delivery" sceneId="success" />
// Polly celebrating your success
```

---

## 📊 CACHING STRATEGY

### How It Works
1. **First request**: `PollyAsset` with `sceneId="delivery_dispatch"` → generates image → stores in cache
2. **Second request**: Same `sceneId` → retrieved from cache (instant)
3. **Cache key**: `polly_{pack}_{sceneId}_{hash(customizations)}`
4. **TTL**: 30 days (configurable)
5. **Storage**: Vercel Blob / Cloudinary / Redis KV (your choice)

### Enable Caching in Production

Edit `pages/api/polly/generate.js`:

```javascript
// With Vercel KV
import { kv } from '@vercel/kv';

// Check cache
const cached = await kv.get(finalCacheKey);
if (cached) {
  return res.json({ ...cached, cached: true });
}

// Generate
const generated = await generatePollyImage(...);

// Store (30 days TTL)
await kv.setex(finalCacheKey, 2592000, {
  imageUrl: generated.url,
  format: generated.format
});
```

---

## 🛡️ QUALITY ASSURANCE

### Character Identity Checks

Before deploying, verify:
- ✅ Polly has round dark sunglasses (always)
- ✅ Polly is a sheep (never another animal)
- ✅ Polly has oversized bare hooves (no shoes)
- ✅ Polly wears a long, worn coat
- ✅ Polly has scruffy beard and fluffy wool
- ✅ Polly's expression is confident & mischievous

### Style Consistency Checks

- ✅ Black & white only (no colors)
- ✅ Heavy ink lines visible
- ✅ Stippling texture present
- ✅ High contrast (no soft gradients)
- ✅ Hand-drawn appearance (not 3D/vector)
- ✅ Comic/graphic novel aesthetic

### Test Prompt

If images don't look right, add `showPrompt={true}` to see what's being sent:

```jsx
<PollyAsset 
  pack="delivery" 
  sceneId="dispatch"
  showPrompt={true}  // Shows prompt in <details> element
/>
```

---

## 🚀 DEPLOYMENT CHECKLIST

- [ ] API endpoint working (`/api/polly/generate`)
- [ ] Image generation provider configured
- [ ] Environment variables set
- [ ] Caching enabled (Redis/Vercel KV)
- [ ] Image storage configured (Blob/Cloudinary)
- [ ] PollyAsset component imported correctly
- [ ] Scene packs loaded
- [ ] Test images rendering in all slides
- [ ] Character identity verified
- [ ] Style consistency verified
- [ ] Performance acceptable (load times < 2s)
- [ ] Error handling working
- [ ] Fallback placeholder in place

---

## 💰 COST ESTIMATION

### Image Generation Costs
- **Hugging Face**: $0.01-0.05 per image (inference credits)
- **Replicate**: $0.025-0.10 per image
- **Stability AI**: $0.01-0.08 per image
- **Custom Model**: Depends on infrastructure

### Caching Impact
- **Without cache**: 1,000 sessions/month × 5 images/session × cost = expensive
- **With cache**: 1,000 sessions × 2 unique images (cached) × cost = 60% savings

### Example Monthly Cost
```
1,000 sessions/month
= 400 unique Polly images (with caching)
= 400 × $0.05 (avg cost) = $20/month
+ Storage: $5-10/month
+ Cache/Redis: $5-10/month
= ~$30-40/month for production
```

---

## 🐛 TROUBLESHOOTING

### "Image generation failed"
- Check API key in `.env.local`
- Verify image generation provider is online
- Check rate limits (add delays between requests)
- Review console logs for API errors

### "Polly doesn't look right"
- Add `showPrompt={true}` to see the prompt
- Compare to reference image
- Adjust prompt in `prompt.master.txt`
- Re-test with new prompt version

### "Images not caching"
- Verify cache backend is running (Redis/Vercel KV)
- Check cache TTL settings
- Clear cache if testing: `kv.del(cacheKey)`

### "Performance slow"
- Enable image compression (WebP format)
- Implement lazy loading
- Pre-generate popular scenes at build time
- Use Cloudinary for CDN + transforms

---

## 📚 NEXT STEPS

1. **Implement in delivery funnel** - Add PollyAsset to each slide
2. **Test with warm prospects** - Send delivery URLs with Polly images
3. **Monitor performance** - Track image load times and generation costs
4. **Iterate on scenes** - Add new Polly scenes based on what works
5. **Scale** - Pre-generate all scenes at build time for instant delivery
6. **Expand** - Create scene packs for other characters (Pauli, Wally, etc.)

---

## 🎯 SUCCESS METRICS

Track these to measure Polly's impact:

- **Engagement**: Do slides with Polly have higher scroll-through rates?
- **Conversion**: Do Polly-featuring slides drive more CTAs?
- **Perception**: Do prospects mention Polly in feedback?
- **Brand**: Does Polly make your funnel more memorable/shareable?
- **Load time**: How long does Polly generation add? (Target: <2s)

---

**Polly is ready. Your delivery funnel is about to get a lot more memorable. 🐾**

Questions? Check the schema (`prompt-api-schema.json`) or the component (`PollyAsset.jsx`) for detailed docs.
