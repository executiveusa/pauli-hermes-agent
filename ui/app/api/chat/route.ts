import { NextRequest, NextResponse } from 'next/server';

const VPS_API = process.env.VPS_API_URL || 'http://31.220.58.212:8642';
const HERMES_API_KEY = process.env.HERMES_API_KEY || '';

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();

    if (!HERMES_API_KEY) {
      return NextResponse.json(
        { error: 'HERMES_API_KEY not configured' },
        { status: 500 }
      );
    }

    const resp = await fetch(`${VPS_API}/api/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Hermes-Key': HERMES_API_KEY,
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(30000),
    });

    const data = await resp.json();
    return NextResponse.json(data, { status: resp.status });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Proxy error';
    return NextResponse.json({ error: msg }, { status: 502 });
  }
}

export async function OPTIONS() {
  return new NextResponse(null, { status: 204 });
}
