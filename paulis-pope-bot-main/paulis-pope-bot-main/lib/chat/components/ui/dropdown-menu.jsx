'use client';

import { createContext, useContext, useState, useEffect, useRef } from 'react';
import { cn } from '../../utils.js';

const DropdownContext = createContext({ open: false, onOpenChange: () => {} });

export function DropdownMenu({ children, open: controlledOpen, onOpenChange: controlledOnOpenChange }) {
  const [internalOpen, setInternalOpen] = useState(false);
  const open = controlledOpen !== undefined ? controlledOpen : internalOpen;
  const onOpenChange = controlledOnOpenChange || setInternalOpen;

  return (
    <DropdownContext.Provider value={{ open, onOpenChange }}>
      <div className="relative inline-block">{children}</div>
    </DropdownContext.Provider>
  );
}

export function DropdownMenuTrigger({ children, asChild, ...props }) {
  const { open, onOpenChange } = useContext(DropdownContext);
  const handleClick = (e) => {
    e.stopPropagation();
    onOpenChange(!open);
  };
  if (asChild && children) {
    return (
      <span onClick={handleClick} {...props}>
        {children}
      </span>
    );
  }
  return (
    <button onClick={handleClick} {...props}>
      {children}
    </button>
  );
}

export function DropdownMenuContent({ children, className, align = 'start', side = 'bottom', sideOffset = 4, ...props }) {
  const { open, onOpenChange } = useContext(DropdownContext);
  const ref = useRef(null);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e) => {
      if (ref.current && !ref.current.contains(e.target)) {
        onOpenChange(false);
      }
    };
    const handleEsc = (e) => {
      if (e.key === 'Escape') onOpenChange(false);
    };
    setTimeout(() => document.addEventListener('click', handleClickOutside), 0);
    document.addEventListener('keydown', handleEsc);
    return () => {
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleEsc);
    };
  }, [open, onOpenChange]);

  if (!open) return null;

  return (
    <div
      ref={ref}
      className={cn(
        'absolute z-50 min-w-[8rem] overflow-hidden rounded-md border border-border bg-background/80 backdrop-blur-sm p-1 text-foreground shadow-lg',
        side === 'bottom' && `top-full mt-1`,
        side === 'top' && `bottom-full mb-1`,
        align === 'end' && 'right-0',
        align === 'start' && 'left-0',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export function DropdownMenuItem({ children, className, onClick, ...props }) {
  const { onOpenChange } = useContext(DropdownContext);
  return (
    <div
      role="menuitem"
      className={cn(
        'relative flex cursor-pointer select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-background focus:bg-background',
        className
      )}
      onClick={(e) => {
        onClick?.(e);
        onOpenChange(false);
      }}
      {...props}
    >
      {children}
    </div>
  );
}

export function DropdownMenuSeparator({ className }) {
  return <div className={cn('-mx-1 my-1 h-px bg-border', className)} />;
}

export function DropdownMenuLabel({ children, className }) {
  return (
    <div className={cn('px-2 py-1.5 text-sm font-semibold', className)}>
      {children}
    </div>
  );
}

export function DropdownMenuGroup({ children }) {
  return <div>{children}</div>;
}
