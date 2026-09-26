import { NextRequest, NextResponse } from 'next/server';
import { getRunEvents } from '@/lib/sandcastle/runner';

export async function GET(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params;
    const limit = parseInt(req.nextUrl.searchParams.get('limit') || '50');
    const offset = parseInt(req.nextUrl.searchParams.get('offset') || '0');
    const severity = req.nextUrl.searchParams.get('severity');

    const events = await getRunEvents(runId);

    if (!events) {
      return NextResponse.json(
        { error: 'Run not found' },
        { status: 404 }
      );
    }

    const filtered = severity ? events.filter(e => e.severity === severity) : events;
    const total = filtered.length;
    const paginated = filtered.slice(offset, offset + limit);

    return NextResponse.json({
      events: paginated,
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
