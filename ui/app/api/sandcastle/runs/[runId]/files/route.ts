import { NextRequest, NextResponse } from 'next/server';
import { getRun } from '@/lib/sandcastle/runner';

export async function GET(
  req: NextRequest,
  { params }: { params: Promise<{ runId: string }> }
) {
  try {
    const { runId } = await params;
    const file = req.nextUrl.searchParams.get('file');
    const run = await getRun(runId);

    if (!run) {
      return NextResponse.json(
        { error: 'Run not found' },
        { status: 404 }
      );
    }

    const files = file
      ? run.changedFiles.filter((f: any) => f.path === file)
      : run.changedFiles;

    return NextResponse.json({
      files,
      total: files.length,
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    return NextResponse.json(
      { error: message },
      { status: 500 }
    );
  }
}
