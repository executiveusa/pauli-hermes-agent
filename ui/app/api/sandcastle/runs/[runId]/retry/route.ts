import { NextRequest, NextResponse } from 'next/server';
import { getRun, retryRun } from '@/lib/sandcastle/runner';

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params; const originalRun = await getRun(runId);

    if (!originalRun) {
      return NextResponse.json(
        { error: 'Run not found' },
        { status: 404 }
      );
    }

    const newRun = await retryRun(runId);

    return NextResponse.json(newRun, { status: 201 });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
