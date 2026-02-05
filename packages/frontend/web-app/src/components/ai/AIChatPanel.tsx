import { useState, useRef, useEffect } from 'react';
import {
  X,
  Send,
  Sparkles,
  Loader2,
  User,
  Bot,
  Copy,
  ThumbsUp,
  ThumbsDown,
  Maximize2,
  Minimize2,
} from 'lucide-react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

interface AIChatPanelProps {
  isOpen: boolean;
  onClose: () => void;
  context?: string;
  initialMessage?: string;
}

export function AIChatPanel({ isOpen, onClose, context = 'general', initialMessage }: AIChatPanelProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  useEffect(() => {
    if (initialMessage && messages.length === 0) {
      handleSend(initialMessage);
    }
  }, [initialMessage]);

  const handleSend = async (messageText?: string) => {
    const text = messageText || input.trim();
    if (!text || isLoading) return;

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: text,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      // Simulated AI response - in production, call the AI service
      await new Promise((resolve) => setTimeout(resolve, 1500));

      const aiResponse: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: getSimulatedResponse(text, context),
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, aiResponse]);
    } catch (error) {
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'I apologize, but I encountered an error processing your request. Please try again.',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (!isOpen) return null;

  return (
    <div
      className={`fixed z-50 bg-white shadow-2xl border border-gray-200 flex flex-col transition-all duration-300 ${
        isExpanded
          ? 'inset-4 rounded-2xl'
          : 'bottom-4 right-4 w-96 h-[600px] rounded-2xl'
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-100 bg-gradient-to-r from-primary-50 to-purple-50 rounded-t-2xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-primary-100 rounded-lg">
            <Sparkles className="w-4 h-4 text-primary-600" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-gray-900">Sysilo AI Assistant</h3>
            <p className="text-xs text-gray-500 capitalize">{context} context</p>
          </div>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="p-1.5 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-white/50"
          >
            {isExpanded ? <Minimize2 className="w-4 h-4" /> : <Maximize2 className="w-4 h-4" />}
          </button>
          <button
            onClick={onClose}
            className="p-1.5 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-white/50"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center py-8">
            <div className="p-4 bg-primary-50 rounded-full w-16 h-16 mx-auto mb-4 flex items-center justify-center">
              <Sparkles className="w-8 h-8 text-primary-600" />
            </div>
            <h4 className="text-lg font-medium text-gray-900 mb-2">How can I help?</h4>
            <p className="text-sm text-gray-500 max-w-xs mx-auto">
              Ask me about your applications, integrations, or get recommendations for your portfolio.
            </p>
            <div className="mt-4 space-y-2">
              {getSuggestedPrompts(context).map((prompt, i) => (
                <button
                  key={i}
                  onClick={() => handleSend(prompt)}
                  className="block w-full text-left px-3 py-2 text-sm text-gray-600 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
                >
                  {prompt}
                </button>
              ))}
            </div>
          </div>
        )}

        {messages.map((message) => (
          <div
            key={message.id}
            className={`flex gap-3 ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            {message.role === 'assistant' && (
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-100 flex items-center justify-center">
                <Bot className="w-4 h-4 text-primary-600" />
              </div>
            )}
            <div
              className={`max-w-[80%] ${
                message.role === 'user'
                  ? 'bg-primary-600 text-white rounded-2xl rounded-tr-md'
                  : 'bg-gray-100 text-gray-900 rounded-2xl rounded-tl-md'
              } px-4 py-2.5`}
            >
              <p className="text-sm whitespace-pre-wrap">{message.content}</p>
              {message.role === 'assistant' && (
                <div className="flex items-center gap-2 mt-2 pt-2 border-t border-gray-200/50">
                  <button
                    onClick={() => copyToClipboard(message.content)}
                    className="p-1 text-gray-400 hover:text-gray-600 rounded"
                  >
                    <Copy className="w-3.5 h-3.5" />
                  </button>
                  <button className="p-1 text-gray-400 hover:text-green-600 rounded">
                    <ThumbsUp className="w-3.5 h-3.5" />
                  </button>
                  <button className="p-1 text-gray-400 hover:text-red-600 rounded">
                    <ThumbsDown className="w-3.5 h-3.5" />
                  </button>
                </div>
              )}
            </div>
            {message.role === 'user' && (
              <div className="flex-shrink-0 w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center">
                <User className="w-4 h-4 text-gray-600" />
              </div>
            )}
          </div>
        ))}

        {isLoading && (
          <div className="flex gap-3">
            <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary-100 flex items-center justify-center">
              <Bot className="w-4 h-4 text-primary-600" />
            </div>
            <div className="bg-gray-100 rounded-2xl rounded-tl-md px-4 py-3">
              <div className="flex items-center gap-2 text-gray-500">
                <Loader2 className="w-4 h-4 animate-spin" />
                <span className="text-sm">Thinking...</span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-4 border-t border-gray-100">
        <div className="flex items-end gap-2">
          <div className="flex-1 relative">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask anything..."
              rows={1}
              className="w-full px-4 py-2.5 pr-12 border border-gray-200 rounded-xl text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              style={{ minHeight: '44px', maxHeight: '120px' }}
            />
          </div>
          <button
            onClick={() => handleSend()}
            disabled={!input.trim() || isLoading}
            className="p-2.5 bg-primary-600 text-white rounded-xl hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Send className="w-5 h-5" />
          </button>
        </div>
        <p className="text-xs text-gray-400 mt-2 text-center">
          AI responses may not always be accurate. Verify important information.
        </p>
      </div>
    </div>
  );
}

function getSuggestedPrompts(context: string): string[] {
  const prompts: Record<string, string[]> = {
    general: [
      'What applications should I consider retiring?',
      'Show me the health status of my integrations',
      'What are the top cost-saving opportunities?',
    ],
    rationalization: [
      'Which applications are in the Eliminate quadrant?',
      'What would be the ROI of retiring legacy systems?',
      'Recommend a migration strategy for my portfolio',
    ],
    operations: [
      'What alerts need immediate attention?',
      'Summarize the incidents from the last 24 hours',
      'What services are showing degraded performance?',
    ],
    governance: [
      'What policy violations exist in my integrations?',
      'Show me pending approval requests',
      'What standards are most frequently violated?',
    ],
  };

  return prompts[context] || prompts.general;
}

function getSimulatedResponse(question: string, context: string): string {
  // Simulated responses - in production, these come from the AI service
  const lowerQ = question.toLowerCase();

  if (lowerQ.includes('retire') || lowerQ.includes('eliminate')) {
    return `Based on my analysis of your portfolio, I've identified 6 applications that are strong candidates for retirement:

**High Priority (Eliminate Quadrant):**
1. **Legacy CRM** - Low business value (3.2/10), poor health (2.8/10), $180K annual cost
2. **Old Reporting Tool** - Redundant functionality, 2 active users, $45K annual cost
3. **Archive System** - No active integrations, data can be migrated, $28K annual cost

**Potential Annual Savings:** $253,000

Would you like me to create a retirement scenario to analyze the full impact?`;
  }

  if (lowerQ.includes('health') || lowerQ.includes('status')) {
    return `Here's a health summary of your integrations:

**Healthy (12):** Operating normally, no issues detected
**Degraded (3):** Data Service showing elevated latency (890ms avg)
**Critical (1):** Salesforce Sync experiencing authentication failures

The most urgent issue is the Salesforce Sync integration - it has failed 23 times in the last hour. This appears to be related to an expired OAuth token.

**Recommended Action:** Re-authenticate the Salesforce connection in the Connections page.`;
  }

  if (lowerQ.includes('cost') || lowerQ.includes('saving')) {
    return `I've analyzed your portfolio for cost optimization opportunities:

**Top Opportunities:**

1. **Application Consolidation** - Merge 3 overlapping reporting tools
   - Estimated savings: $120K/year
   - Effort: Medium

2. **Cloud Right-sizing** - Reduce over-provisioned resources for 8 apps
   - Estimated savings: $85K/year
   - Effort: Low

3. **License Optimization** - Remove unused licenses from Legacy CRM
   - Estimated savings: $45K/year
   - Effort: Low

**Total Potential Savings:** $250K/year

Would you like me to create a detailed analysis for any of these?`;
  }

  return `I understand you're asking about ${question.substring(0, 50)}...

Based on the current ${context} context, here's what I found:

- Your portfolio contains 41 applications
- 6 applications are in the Eliminate quadrant
- Total annual IT spend is $4.2M
- Average health score across the portfolio is 6.8/10

Is there something specific you'd like me to dive deeper into?`;
}
