import { NextRequest, NextResponse } from 'next/server';
import { getRuns, runSandcastleTask } from '@/lib/sandcastle/runner';

export async function GET(req: NextRequest) {
  try {
    const limit = parseInt(req.nextUrl.searchParams.get('limit') || '20');
    const offset = parseInt(req.nextUrl.searchParams.get('offset') || '0');
    const status = req.nextUrl.searchParams.get('status');

    const runs = await getRuns();
    const filtered = status ? runs.filter(r => r.status === status) : runs;
    const total = filtered.length;
    const paginated = filtered.slice(offset, offset + limit);

    return NextResponse.json({
      runs: paginated,
      total,
      limit,
      offset,
      hasMore: offset + limit < total,
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}

export async function POST(req: NextRequest) {
  try {
    const body = await req.json();
    const { beadId, title, prompt, requireApproval, maxIterations, requireTests } = body;

    if (!beadId || !title || !prompt) {
      return NextResponse.json(
        { error: 'Missing required fields: beadId, title, prompt' },
        { status: 400 }
      );
    }

    const result = await runSandcastleTask({
      beadId,
      title,
      prompt,
      requireApproval: requireApproval !== false,
      maxIterations: maxIterations || 5,
      requireTests: requireTests !== false,
    });

    return NextResponse.json(result.run, { status: 201 });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
