import { NextRequest, NextResponse } from 'next/server';
import { healthcheckSandcastle } from '@/lib/sandcastle/runner';

export async function POST(req: NextRequest) {
  try {
    const health = await healthcheckSandcastle();

    return NextResponse.json(health);
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
