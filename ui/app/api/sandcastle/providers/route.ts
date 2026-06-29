import { NextRequest, NextResponse } from 'next/server';
import { listSandcastleProviders } from '@/lib/sandcastle/runner';

export async function GET(req: NextRequest) {
  try {
    const providers = await listSandcastleProviders();
    return NextResponse.json({ providers });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
