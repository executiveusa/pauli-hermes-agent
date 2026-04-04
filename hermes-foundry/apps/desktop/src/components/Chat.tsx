// components/Chat.tsx — Streaming chat interface

import React, { useEffect, useRef, useState, useCallback } from "react";
import { useChatStore } from "../stores/chat";
import type { Message } from "../lib/types";

export function Chat() {
  const [input, setInput] = useState("");
  const bottomRef       = useRef<HTMLDivElement>(null);
  const textareaRef     = useRef<HTMLTextAreaElement>(null);

  const activeConvId    = useChatStore((s) => s.activeConversationId);
  const messages        = useChatStore((s) =>
    s.activeConversationId ? (s.messages[s.activeConversationId] ?? []) : []
  );
  const streaming       = useChatStore((s) => s.streaming);
  const sendMessage     = useChatStore((s) => s.sendMessage);
  const stopStreaming    = useChatStore((s) => s.stopStreaming);
  const newConversation = useChatStore((s) => s.newConversation);
  const currentModel    = useChatStore((s) => s.currentModel);

  // Auto-scroll to bottom when new tokens arrive
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Auto-resize textarea
  useEffect(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    ta.style.height = "auto";
    ta.style.height = Math.min(ta.scrollHeight, 160) + "px";
  }, [input]);

  const handleSubmit = useCallback(async () => {
    const text = input.trim();
    if (!text || streaming) return;
    setInput("");
    if (!activeConvId) newConversation();
    await sendMessage(text);
  }, [input, streaming, activeConvId, sendMessage, newConversation]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  if (!activeConvId) {
    return (
      <div className="chat chat--empty">
        <div className="chat__empty-state">
          <div className="chat__empty-icon">Ω</div>
          <h2 className="chat__empty-title">Hermes</h2>
          <p className="chat__empty-sub">Start a conversation or press N for new</p>
          <button
            className="btn btn-primary"
            onClick={() => newConversation()}
          >
            New conversation
          </button>
        </div>
        <style>{chatStyles}</style>
      </div>
    );
  }

  return (
    <div className="chat">
      {/* Header */}
      <div className="chat__header">
        <span className="chat__model-badge">{currentModel}</span>
        <div className="chat__header-actions">
          <button
            className="btn btn-ghost"
            onClick={() => newConversation()}
          >
            New
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className="chat__messages selectable">
        {messages.length === 0 && (
          <div className="chat__welcome">
            <p>Send a message to begin.</p>
          </div>
        )}
        {messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div className="chat__input-area">
        <div className="chat__input-wrapper">
          <textarea
            ref={textareaRef}
            className="chat__textarea"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Message Hermes…"
            rows={1}
            disabled={streaming}
          />
          <div className="chat__input-controls">
            {streaming ? (
              <button
                className="btn btn-danger"
                onClick={stopStreaming}
                title="Stop generation"
              >
                ■ Stop
              </button>
            ) : (
              <button
                className="btn btn-primary"
                onClick={handleSubmit}
                disabled={!input.trim()}
              >
                Send
              </button>
            )}
          </div>
        </div>
        <p className="chat__hint">Enter to send · Shift+Enter for newline</p>
      </div>

      <style>{chatStyles}</style>
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
  const isUser = message.role === "user";
  const isError = !!message.error;

  return (
    <div className={`message message--${message.role}`}>
      <div className="message__meta">
        <span className="message__role">{isUser ? "You" : "Hermes"}</span>
        {message.model && (
          <span className="message__model">{message.model}</span>
        )}
      </div>
      <div className={`message__content ${isError ? "message--error" : ""}`}>
        {isError
          ? <span className="message__error-text">{message.error}</span>
          : <ContentRenderer content={message.content} streaming={!!message.streaming} />
        }
      </div>
    </div>
  );
}

function ContentRenderer({ content, streaming }: { content: string; streaming: boolean }) {
  // Simple code block detection — split on ``` boundaries
  const parts = content.split(/(```[\s\S]*?```|`[^`]+`)/g);

  return (
    <span>
      {parts.map((part, i) => {
        if (part.startsWith("```")) {
          const match = part.match(/^```(\w*)\n?([\s\S]*?)```$/);
          const lang = match?.[1] ?? "";
          const code = match?.[2] ?? part.slice(3, -3);
          return (
            <pre key={i} className="message__code-block">
              {lang && <span className="message__code-lang">{lang}</span>}
              <code>{code}</code>
            </pre>
          );
        }
        if (part.startsWith("`") && part.endsWith("`") && part.length > 2) {
          return <code key={i}>{part.slice(1, -1)}</code>;
        }
        return <span key={i}>{part}</span>;
      })}
      {streaming && <span className="message__cursor" aria-hidden />}
    </span>
  );
}

const chatStyles = `
  .chat {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .chat--empty {
    align-items: center;
    justify-content: center;
  }

  .chat__empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-4);
    text-align: center;
    max-width: 320px;
  }

  .chat__empty-icon {
    font-size: 40px;
    color: var(--accent);
    opacity: 0.6;
  }

  .chat__empty-title {
    font-size: var(--text-xl);
    font-weight: var(--weight-semi);
    color: var(--text-primary);
  }

  .chat__empty-sub {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .chat__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--sp-3) var(--sp-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .chat__model-badge {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    border: 1px solid var(--border);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .chat__header-actions {
    display: flex;
    gap: var(--sp-2);
  }

  .chat__messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
  }

  .chat__welcome {
    color: var(--text-tertiary);
    font-size: var(--text-sm);
    text-align: center;
    padding: var(--sp-8) 0;
  }

  .message {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
    max-width: 820px;
  }

  .message--user {
    align-self: flex-end;
    align-items: flex-end;
  }

  .message--assistant {
    align-self: flex-start;
    align-items: flex-start;
  }

  .message__meta {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    padding: 0 var(--sp-1);
  }

  .message__role {
    font-weight: var(--weight-medium);
  }

  .message--assistant .message__role {
    color: var(--accent);
  }

  .message__model {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .message__content {
    padding: var(--sp-3) var(--sp-4);
    border-radius: var(--radius);
    font-size: var(--text-base);
    line-height: 1.65;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .message--user .message__content {
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-primary);
  }

  .message--assistant .message__content {
    background: transparent;
    color: var(--text-primary);
  }

  .message--error .message__content {
    background: rgba(239,68,68,0.08);
    border: 1px solid rgba(239,68,68,0.3);
    color: var(--error);
  }

  .message__code-block {
    margin: var(--sp-2) 0;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-3) var(--sp-4);
    font-size: var(--text-sm);
    overflow-x: auto;
    position: relative;
  }

  .message__code-lang {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin-bottom: var(--sp-2);
    font-family: var(--font-mono);
  }

  .message__cursor {
    display: inline-block;
    width: 8px;
    height: 1em;
    background: var(--accent);
    margin-left: 1px;
    vertical-align: text-bottom;
    animation: blink 0.8s step-end infinite;
    opacity: 0.8;
  }

  @keyframes blink {
    0%, 100% { opacity: 0.8; }
    50%       { opacity: 0; }
  }

  .chat__input-area {
    padding: var(--sp-3) var(--sp-4) var(--sp-4);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .chat__input-wrapper {
    display: flex;
    align-items: flex-end;
    gap: var(--sp-2);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: var(--sp-2) var(--sp-3);
    transition: border-color var(--transition);
  }

  .chat__input-wrapper:focus-within {
    border-color: var(--accent);
  }

  .chat__textarea {
    flex: 1;
    resize: none;
    border: none;
    background: transparent;
    outline: none;
    color: var(--text-primary);
    font-size: var(--text-base);
    line-height: 1.5;
    min-height: 22px;
    max-height: 160px;
    overflow-y: auto;
    padding: 0;
  }

  .chat__textarea::placeholder {
    color: var(--text-tertiary);
  }

  .chat__textarea:disabled {
    opacity: 0.6;
  }

  .chat__input-controls {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-shrink: 0;
  }

  .chat__hint {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin-top: var(--sp-1);
    padding: 0 var(--sp-1);
  }
`;
