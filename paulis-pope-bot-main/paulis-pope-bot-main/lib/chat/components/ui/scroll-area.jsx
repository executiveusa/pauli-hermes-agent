'use client';

import { cn } from '../../utils.js';

export function ScrollArea({ children, className, ...props }) {
  return (
    <div className={cn('relative overflow-hidden', className)} {...props}>
      <div className="h-full w-full overflow-y-auto scrollbar-thin">
        {children}
      </div>
    </div>
  );
}

export function ScrollBar() {
  return null;
}
