import { NextRequest, NextResponse } from 'next/server';

const VPS_API = process.env.VPS_API_URL || 'http://31.220.58.212:8642';

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();

    const resp = await fetch(`${VPS_API}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
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
