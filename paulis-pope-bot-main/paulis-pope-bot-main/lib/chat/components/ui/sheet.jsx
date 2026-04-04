'use client';

import { createContext, useContext, useEffect } from 'react';
import { cn } from '../../utils.js';

const SheetContext = createContext({ open: false, onOpenChange: () => {} });

export function Sheet({ children, open, onOpenChange }) {
  return (
    <SheetContext.Provider value={{ open, onOpenChange }}>
      {children}
    </SheetContext.Provider>
  );
}

export function SheetTrigger({ children, asChild, ...props }) {
  const { onOpenChange } = useContext(SheetContext);
  if (asChild && children) {
    return (
      <span onClick={() => onOpenChange(true)} {...props}>
        {children}
      </span>
    );
  }
  return (
    <button onClick={() => onOpenChange(true)} {...props}>
      {children}
    </button>
  );
}

export function SheetContent({ children, className, side = 'left', ...props }) {
  const { open, onOpenChange } = useContext(SheetContext);

  useEffect(() => {
    if (!open) return;
    const handleEsc = (e) => {
      if (e.key === 'Escape') onOpenChange(false);
    };
    document.addEventListener('keydown', handleEsc);
    return () => document.removeEventListener('keydown', handleEsc);
  }, [open, onOpenChange]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50">
      {/* Overlay */}
      <div
        className="fixed inset-0 bg-black/50"
        onClick={() => onOpenChange(false)}
      />
      {/* Content */}
      <div
        className={cn(
          'fixed z-50 bg-background shadow-lg transition-transform',
          side === 'left' && 'inset-y-0 left-0 w-3/4 max-w-sm border-r border-border',
          side === 'right' && 'inset-y-0 right-0 w-3/4 max-w-sm border-l border-border',
          className
        )}
        {...props}
      >
        {children}
      </div>
    </div>
  );
}

export function SheetHeader({ children, className }) {
  return <div className={cn('flex flex-col space-y-2 p-4', className)}>{children}</div>;
}

export function SheetTitle({ children, className }) {
  return <h2 className={cn('text-lg font-semibold text-foreground', className)}>{children}</h2>;
}

export function SheetDescription({ children, className }) {
  return <p className={cn('text-sm text-muted-foreground', className)}>{children}</p>;
}

export function SheetClose({ children, asChild, ...props }) {
  const { onOpenChange } = useContext(SheetContext);
  if (asChild && children) {
    return (
      <span onClick={() => onOpenChange(false)} {...props}>
        {children}
      </span>
    );
  }
  return (
    <button onClick={() => onOpenChange(false)} {...props}>
      {children}
    </button>
  );
}
