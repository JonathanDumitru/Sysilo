import { create } from 'zustand';

interface AIContext {
  type: 'asset' | 'integration' | 'governance' | 'general';
  id?: string;
  name?: string;
  data?: Record<string, unknown>;
}

interface AIContextState {
  isOpen: boolean;
  position: { x: number; y: number } | null;
  context: AIContext | null;
  openAt: (position: { x: number; y: number }, context: AIContext) => void;
  close: () => void;
}

export const useAIContext = create<AIContextState>((set) => ({
  isOpen: false,
  position: null,
  context: null,
  openAt: (position, context) => set({ isOpen: true, position, context }),
  close: () => set({ isOpen: false, position: null, context: null }),
}));
