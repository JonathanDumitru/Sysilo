import { useState } from 'react';
import { Sparkles } from 'lucide-react';
import { AIChatPanel } from './AIChatPanel';

interface AIAssistButtonProps {
  context?: string;
  className?: string;
}

export function AIAssistButton({ context = 'general', className = '' }: AIAssistButtonProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <button
        onClick={() => setIsOpen(true)}
        className={`fixed bottom-6 right-6 p-4 bg-gradient-to-r from-primary-600 to-purple-600 text-white rounded-full shadow-lg hover:shadow-xl transform hover:scale-105 transition-all z-40 ${className}`}
      >
        <Sparkles className="w-6 h-6" />
      </button>

      <AIChatPanel
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        context={context}
      />
    </>
  );
}
