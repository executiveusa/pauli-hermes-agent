import { NextRequest, NextResponse } from 'next/server';
import { discardRun } from '@/lib/sandcastle/runner';

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params;
    const body = await req.json();
    const { reason } = body;

    const success = await discardRun(runId, reason);

    if (!success) {
      return NextResponse.json(
        { error: 'Run not found' },
        { status: 404 }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Run discarded',
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
