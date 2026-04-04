'use client';

import { useState } from 'react';
import { WrenchIcon, SpinnerIcon, CheckIcon, XIcon, ChevronDownIcon } from './icons.js';
import { cn } from '../utils.js';

const TOOL_DISPLAY_NAMES = {
  create_job: 'Create Job',
  get_job_status: 'Check Job Status',
  get_system_technical_specs: 'Read System Docs',
};

function getToolDisplayName(toolName) {
  return TOOL_DISPLAY_NAMES[toolName] || toolName.replace(/_/g, ' ');
}

function formatContent(content) {
  if (content == null) return null;
  if (typeof content === 'string') {
    try {
      const parsed = JSON.parse(content);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return content;
    }
  }
  return JSON.stringify(content, null, 2);
}

export function ToolCall({ part }) {
  const [expanded, setExpanded] = useState(false);

  const toolName = part.toolName || (part.type?.startsWith('tool-') ? part.type.slice(5) : 'tool');
  const displayName = getToolDisplayName(toolName);
  const state = part.state || 'input-available';

  const isRunning = state === 'input-streaming' || state === 'input-available';
  const isDone = state === 'output-available';
  const isError = state === 'output-error';

  return (
    <div className="my-1 rounded-lg border border-border bg-background">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-muted/50 rounded-lg"
      >
        <WrenchIcon size={14} className="text-muted-foreground shrink-0" />
        <span className="font-medium text-foreground">{displayName}</span>
        <span className="ml-auto flex items-center gap-1.5 text-xs text-muted-foreground">
          {isRunning && (
            <>
              <SpinnerIcon size={12} />
              <span>Running...</span>
            </>
          )}
          {isDone && (
            <>
              <CheckIcon size={12} className="text-green-500" />
              <span>Done</span>
            </>
          )}
          {isError && (
            <>
              <XIcon size={12} className="text-red-500" />
              <span>Error</span>
            </>
          )}
        </span>
        <ChevronDownIcon
          size={14}
          className={cn(
            'text-muted-foreground transition-transform shrink-0',
            expanded && 'rotate-180'
          )}
        />
      </button>

      {expanded && (
        <div className="border-t border-border px-3 py-2 text-xs">
          {part.input != null && (
            <div className="mb-2">
              <div className="font-medium text-muted-foreground mb-1">Input</div>
              <pre className="whitespace-pre-wrap break-all rounded bg-muted p-2 text-foreground overflow-x-auto">
                {formatContent(part.input)}
              </pre>
            </div>
          )}
          {part.output != null && (
            <div>
              <div className="font-medium text-muted-foreground mb-1">Output</div>
              <pre className="whitespace-pre-wrap break-all rounded bg-muted p-2 text-foreground overflow-x-auto max-h-64 overflow-y-auto">
                {formatContent(part.output)}
              </pre>
            </div>
          )}
          {part.input == null && part.output == null && (
            <div className="text-muted-foreground italic">Waiting for data...</div>
          )}
        </div>
      )}
    </div>
  );
}
