'use client';

import { createContext, useContext } from 'react';

const ChatNavContext = createContext(null);

export const ChatNavProvider = ChatNavContext.Provider;

export function useChatNav() {
  return useContext(ChatNavContext);
}
