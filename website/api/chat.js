// Vercel serverless function — proxies voice agent requests to the VPS
// Browser calls /api/chat (HTTPS) → this function → VPS:8642 (HTTP, server-side OK)

const ALLOWED_ORIGINS = [
  'https://pauli-hermes-agent.vercel.app',
];

export default async function handler(req, res) {
  const origin = req.headers.origin || '';
  const isAllowed = ALLOWED_ORIGINS.includes(origin) || /^https:\/\/.*\.vercel\.app$/.test(origin);

  if (isAllowed) {
    res.setHeader('Access-Control-Allow-Origin', origin);
  }
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const vpsUrl = 'http://31.220.58.212:8642';

  try {
    const response = await fetch(`${vpsUrl}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req.body),
      signal: AbortSignal.timeout(30000),
    });

    if (!response.ok) {
      const text = await response.text();
      return res.status(response.status).json({ error: text });
    }

    const data = await response.json();
    return res.status(200).json(data);
  } catch (err) {
    if (err.name === 'TimeoutError') {
      return res.status(504).json({ error: 'VPS timeout — check that the agent is running on 31.220.58.212:8642' });
    }
    return res.status(502).json({ error: `Cannot reach VPS: ${err.message}` });
  }
}
