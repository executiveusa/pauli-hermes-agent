'use client';

import { useRef, useEffect, useState } from 'react';
import { PreviewMessage, ThinkingMessage } from './message.js';
import { Greeting } from './greeting.js';
import { ArrowDown } from 'lucide-react';

export function Messages({ messages, status, onRetry, onEdit }) {
  const containerRef = useRef(null);
  const endRef = useRef(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (isAtBottom && endRef.current) {
      endRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, status, isAtBottom]);

  // Track scroll position
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleScroll = () => {
      const { scrollTop, scrollHeight, clientHeight } = container;
      setIsAtBottom(scrollHeight - scrollTop - clientHeight < 40);
    };

    container.addEventListener('scroll', handleScroll, { passive: true });
    return () => container.removeEventListener('scroll', handleScroll);
  }, []);

  const scrollToBottom = () => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <div className="relative flex-1">
      <div
        className="absolute inset-0 touch-pan-y overflow-y-auto"
        ref={containerRef}
      >
        <div className="mx-auto flex min-w-0 max-w-4xl flex-col gap-4 px-4 py-4 md:gap-6 md:px-6">
          {messages.length === 0 && <Greeting />}

          {messages.map((message, index) => (
            <PreviewMessage
              key={message.id}
              message={message}
              isLoading={status === 'streaming' && index === messages.length - 1}
              onRetry={onRetry}
              onEdit={onEdit}
            />
          ))}

          {status === 'submitted' && <ThinkingMessage />}

          <div className="min-h-[24px] shrink-0" ref={endRef} />
        </div>
      </div>

      {!isAtBottom && (
        <button
          className="absolute bottom-4 left-1/2 z-10 -translate-x-1/2 rounded-full border border-border bg-background p-2 shadow-lg hover:bg-muted"
          onClick={scrollToBottom}
          aria-label="Scroll to bottom"
        >
          <ArrowDown className="size-4" />
        </button>
      )}
    </div>
  );
}
