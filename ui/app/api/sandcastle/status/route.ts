import { NextRequest, NextResponse } from 'next/server';
import {
  healthcheckSandcastle,
  listSandcastleProviders,
} from '@/lib/sandcastle/runner';

export async function GET(req: NextRequest) {
  try {
    const [health, providers] = await Promise.all([
      healthcheckSandcastle(),
      listSandcastleProviders(),
    ]);

    return NextResponse.json({
      healthy: health.healthy,
      providers,
      activeRuns: health.activeRuns,
      totalRuns: health.totalRuns,
      timestamp: new Date().toISOString(),
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
