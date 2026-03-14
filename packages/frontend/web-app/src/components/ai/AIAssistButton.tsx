import { useCallback } from 'react';
import { Sparkles } from 'lucide-react';
import { AIChatPanel } from './AIChatPanel';
import { useAIContext } from '../../hooks/useAIContext';

interface AIAssistButtonProps {
  context?: string;
  className?: string;
}

export function AIAssistButton({ context = 'general', className = '' }: AIAssistButtonProps) {
  const { isOpen, openAt } = useAIContext();

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      if (isOpen) {
        useAIContext.getState().close();
        return;
      }
      const rect = e.currentTarget.getBoundingClientRect();
      // Position the panel above and to the left of the button
      openAt(
        { x: rect.left - 360, y: rect.top - 420 },
        {
          type: context as 'asset' | 'integration' | 'governance' | 'general',
        }
      );
    },
    [isOpen, context, openAt]
  );

  return (
    <>
      <button
        onClick={handleClick}
        className={`fixed bottom-12 right-6 p-4 rounded-full z-40 transition-all transform hover:scale-105 bg-[rgba(163,113,247,0.2)] backdrop-blur border border-[rgba(163,113,247,0.3)] shadow-[0_0_15px_rgba(163,113,247,0.3)] hover:shadow-[0_0_25px_rgba(163,113,247,0.5)] text-purple-400 ${className}`}
      >
        <Sparkles className="w-6 h-6" />
        {/* Pulsing ring */}
        <span className="absolute inset-0 rounded-full animate-ping bg-purple-500/10 pointer-events-none" />
      </button>

      <AIChatPanel />
    </>
  );
}
