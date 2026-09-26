import { NextRequest, NextResponse } from 'next/server';
import { stopRun } from '@/lib/sandcastle/runner';

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params;
    const success = await stopRun(runId);

    if (!success) {
      return NextResponse.json(
        { error: 'Run not found or already stopped' },
        { status: 404 }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Run stopped',
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
