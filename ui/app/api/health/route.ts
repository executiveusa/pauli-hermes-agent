import { NextRequest, NextResponse } from 'next/server';

const VPS_API = process.env.VPS_API_URL || 'http://31.220.58.212:8642';

export async function GET(_req: NextRequest) {
  try {
    const resp = await fetch(`${VPS_API}/health`, {
      signal: AbortSignal.timeout(5000),
    });
    const data = await resp.json();
    return NextResponse.json({ ui: 'ok', api: data });
  } catch {
    return NextResponse.json({ ui: 'ok', api: 'unreachable' }, { status: 200 });
  }
}
