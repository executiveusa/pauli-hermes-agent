import { NextRequest, NextResponse } from 'next/server';
import { approveMerge } from '@/lib/sandcastle/runner';

export async function POST(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params;
    const body = await req.json();
    const { notes } = body;

    const success = await approveMerge(runId, notes);

    if (!success) {
      return NextResponse.json(
        { error: 'Run not found or not eligible for merge' },
        { status: 404 }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Merge approved',
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
