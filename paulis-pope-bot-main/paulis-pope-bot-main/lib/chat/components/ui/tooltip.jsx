'use client';

import { useState, useRef, useEffect, createContext, useContext, cloneElement } from 'react';
import { cn } from '../../utils.js';

const TooltipContext = createContext({ open: false });

export function TooltipProvider({ children, delayDuration = 200 }) {
  return children;
}

export function Tooltip({ children }) {
  const [open, setOpen] = useState(false);
  const timeoutRef = useRef(null);

  const handleOpen = () => {
    timeoutRef.current = setTimeout(() => setOpen(true), 200);
  };

  const handleClose = () => {
    clearTimeout(timeoutRef.current);
    setOpen(false);
  };

  useEffect(() => () => clearTimeout(timeoutRef.current), []);

  return (
    <TooltipContext.Provider value={{ open, handleOpen, handleClose }}>
      <div className="relative inline-flex" onMouseEnter={handleOpen} onMouseLeave={handleClose}>
        {children}
      </div>
    </TooltipContext.Provider>
  );
}

export function TooltipTrigger({ children, asChild }) {
  if (asChild && children) {
    return children;
  }
  return children;
}

export function TooltipContent({ children, className, align = 'center', side = 'bottom', ...props }) {
  const { open } = useContext(TooltipContext);
  if (!open) return null;

  return (
    <div
      className={cn(
        'absolute z-50 overflow-hidden rounded-md border border-border bg-muted px-3 py-1.5 text-sm text-foreground shadow-md',
        'animate-in fade-in-0 zoom-in-95',
        side === 'bottom' && 'top-full mt-1',
        side === 'top' && 'bottom-full mb-1',
        side === 'right' && 'left-full ml-1 top-1/2 -translate-y-1/2',
        side === 'left' && 'right-full mr-1 top-1/2 -translate-y-1/2',
        side !== 'right' && side !== 'left' && align === 'center' && 'left-1/2 -translate-x-1/2',
        side !== 'right' && side !== 'left' && align === 'end' && 'right-0',
        side !== 'right' && side !== 'left' && align === 'start' && 'left-0',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}
