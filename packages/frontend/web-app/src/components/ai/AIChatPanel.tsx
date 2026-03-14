import { useState, useRef, useEffect, useCallback } from 'react';
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
} from 'lucide-react';
import { useAIContext } from '../../hooks/useAIContext';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export function AIChatPanel() {
  const { isOpen, position, context, close } = useAIContext();
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  // Reset messages when context changes
  useEffect(() => {
    if (isOpen) {
      setMessages([]);
      setInput('');
      setStreamingText('');
    }
  }, [isOpen, context?.type, context?.id]);

  // Focus input when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingText]);

  // Close on Escape
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        close();
      }
    },
    [isOpen, close]
  );

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Close on click outside
  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        close();
      }
    };
    // Delay to prevent immediate close from the opening click
    const timer = setTimeout(() => {
      document.addEventListener('mousedown', handleClickOutside);
    }, 100);
    return () => {
      clearTimeout(timer);
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, close]);

  const simulateStreaming = async (text: string) => {
    setStreamingText('');
    const words = text.split(' ');
    for (let i = 0; i < words.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 30));
      setStreamingText((prev) => (prev ? prev + ' ' + words[i] : words[i]));
    }
    return text;
  };

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
    setStreamingText('');

    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      const responseText = getContextualResponse(text, context?.type ?? 'general', context?.name);
      await simulateStreaming(responseText);

      const aiResponse: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: responseText,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, aiResponse]);
      setStreamingText('');
    } catch {
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: 'I apologize, but I encountered an error. Please try again.',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
      setStreamingText('');
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (!isOpen || !position) return null;

  // Calculate panel position - keep it within viewport
  const panelStyle = computePanelPosition(position);

  const contextLabel = context?.type ?? 'general';
  const contextName = context?.name;

  return (
    <div
      ref={panelRef}
      className="fixed z-50 w-[400px] max-h-[60vh] bg-surface-raised/95 backdrop-blur-sm border border-surface-border rounded-xl shadow-2xl shadow-black/30 flex flex-col"
      style={panelStyle}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-surface-border rounded-t-xl">
        <div className="flex items-center gap-2">
          <div className="p-1.5 bg-purple-500/20 rounded-lg">
            <Sparkles className="w-4 h-4 text-purple-400" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-gray-200">AI Insights</h3>
            <p className="text-xs text-gray-500 capitalize">
              {contextName ? `${contextName} - ${contextLabel}` : contextLabel}
            </p>
          </div>
        </div>
        <button
          onClick={close}
          className="p-1.5 text-gray-500 hover:text-gray-300 rounded-lg hover:bg-surface-overlay/50 transition-colors"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 min-h-[200px]">
        {messages.length === 0 && !streamingText && (
          <div className="text-center py-6">
            <div className="p-3 bg-purple-500/10 rounded-full w-12 h-12 mx-auto mb-3 flex items-center justify-center">
              <Sparkles className="w-6 h-6 text-purple-400" />
            </div>
            <h4 className="text-sm font-medium text-gray-300 mb-1">
              {getContextHeading(context?.type)}
            </h4>
            <p className="text-xs text-gray-500 max-w-xs mx-auto mb-3">
              {getContextDescription(context?.type, context?.name)}
            </p>
            <div className="space-y-1.5">
              {getContextPrompts(context?.type, context?.name).map((prompt, i) => (
                <button
                  key={i}
                  onClick={() => handleSend(prompt)}
                  className="block w-full text-left px-3 py-2 text-xs text-gray-400 bg-surface-overlay/50 rounded-lg hover:bg-surface-overlay hover:text-gray-300 transition-colors"
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
            className={`flex gap-2.5 ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            {message.role === 'assistant' && (
              <div className="flex-shrink-0 w-7 h-7 rounded-full bg-purple-500/20 flex items-center justify-center">
                <Bot className="w-3.5 h-3.5 text-purple-400" />
              </div>
            )}
            <div
              className={`max-w-[85%] ${
                message.role === 'user'
                  ? 'bg-primary-600/80 text-white rounded-2xl rounded-tr-md'
                  : 'bg-surface-overlay text-gray-300 rounded-2xl rounded-tl-md'
              } px-3 py-2`}
            >
              <p className="text-xs whitespace-pre-wrap leading-relaxed">{message.content}</p>
              {message.role === 'assistant' && (
                <div className="flex items-center gap-1.5 mt-2 pt-1.5 border-t border-white/5">
                  <button
                    onClick={() => copyToClipboard(message.content)}
                    className="p-0.5 text-gray-500 hover:text-gray-300 rounded"
                  >
                    <Copy className="w-3 h-3" />
                  </button>
                  <button className="p-0.5 text-gray-500 hover:text-green-400 rounded">
                    <ThumbsUp className="w-3 h-3" />
                  </button>
                  <button className="p-0.5 text-gray-500 hover:text-red-400 rounded">
                    <ThumbsDown className="w-3 h-3" />
                  </button>
                </div>
              )}
            </div>
            {message.role === 'user' && (
              <div className="flex-shrink-0 w-7 h-7 rounded-full bg-surface-overlay flex items-center justify-center">
                <User className="w-3.5 h-3.5 text-gray-400" />
              </div>
            )}
          </div>
        ))}

        {/* Streaming text display */}
        {isLoading && streamingText && (
          <div className="flex gap-2.5">
            <div className="flex-shrink-0 w-7 h-7 rounded-full bg-purple-500/20 flex items-center justify-center">
              <Bot className="w-3.5 h-3.5 text-purple-400" />
            </div>
            <div className="bg-surface-overlay text-gray-300 rounded-2xl rounded-tl-md px-3 py-2 max-w-[85%]">
              <p className="text-xs whitespace-pre-wrap leading-relaxed">
                {streamingText}
                <span className="inline-block w-1.5 h-3.5 bg-purple-400 ml-0.5 animate-pulse" />
              </p>
            </div>
          </div>
        )}

        {/* Loading indicator before streaming starts */}
        {isLoading && !streamingText && (
          <div className="flex gap-2.5">
            <div className="flex-shrink-0 w-7 h-7 rounded-full bg-purple-500/20 flex items-center justify-center">
              <Bot className="w-3.5 h-3.5 text-purple-400" />
            </div>
            <div className="bg-surface-overlay rounded-2xl rounded-tl-md px-3 py-2.5">
              <div className="flex items-center gap-2 text-gray-500">
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                <span className="text-xs">Analyzing...</span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-3 border-t border-surface-border">
        <div className="flex items-end gap-2">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleInputKeyDown}
            placeholder="Ask a follow-up..."
            rows={1}
            className="flex-1 px-3 py-2 bg-surface-overlay border border-surface-border rounded-lg text-xs text-gray-300 placeholder-gray-600 resize-none focus:outline-none focus:ring-1 focus:ring-purple-500/50 focus:border-purple-500/30"
            style={{ minHeight: '36px', maxHeight: '80px' }}
          />
          <button
            onClick={() => handleSend()}
            disabled={!input.trim() || isLoading}
            className="p-2 bg-purple-500/20 text-purple-400 rounded-lg hover:bg-purple-500/30 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
        <p className="text-[10px] text-gray-600 mt-1.5 text-center">
          AI responses may not always be accurate. Verify important information.
        </p>
      </div>
    </div>
  );
}

// --- Helper functions ---

function computePanelPosition(pos: { x: number; y: number }): React.CSSProperties {
  const panelWidth = 400;
  const panelMaxHeight = window.innerHeight * 0.6;
  const padding = 16;

  let left = pos.x;
  let top = pos.y;

  // Keep within horizontal bounds
  if (left + panelWidth > window.innerWidth - padding) {
    left = window.innerWidth - panelWidth - padding;
  }
  if (left < padding) {
    left = padding;
  }

  // Keep within vertical bounds - prefer showing above if near bottom
  if (top + panelMaxHeight > window.innerHeight - padding) {
    top = window.innerHeight - panelMaxHeight - padding;
  }
  if (top < padding) {
    top = padding;
  }

  return { left, top };
}

function getContextHeading(type?: string): string {
  switch (type) {
    case 'asset':
      return 'Asset Insights';
    case 'integration':
      return 'Integration Analysis';
    case 'governance':
      return 'Policy Explainer';
    default:
      return 'How can I help?';
  }
}

function getContextDescription(type?: string, name?: string): string {
  switch (type) {
    case 'asset':
      return `Get AI-powered insights about ${name ?? 'this asset'} including health, dependencies, and recommendations.`;
    case 'integration':
      return `Analyze run history, failure patterns, and optimization opportunities for ${name ?? 'this integration'}.`;
    case 'governance':
      return `Understand ${name ?? 'this policy'} in plain English and get compliance guidance.`;
    default:
      return 'Ask about your applications, integrations, or get recommendations for your portfolio.';
  }
}

function getContextPrompts(type?: string, name?: string): string[] {
  const label = name ?? 'this';
  switch (type) {
    case 'asset':
      return [
        `What is the health status of ${label}?`,
        `Show downstream dependencies for ${label}`,
        `What are the top recommendations for ${label}?`,
      ];
    case 'integration':
      return [
        `Summarize recent run history for ${label}`,
        `Analyze failure patterns for ${label}`,
        `How can I optimize ${label}?`,
      ];
    case 'governance':
      return [
        `Explain ${label} in plain English`,
        `What resources does ${label} affect?`,
        `How do I achieve compliance with ${label}?`,
      ];
    default:
      return [
        'What applications should I consider retiring?',
        'Show me the health status of my integrations',
        'What are the top cost-saving opportunities?',
      ];
  }
}

function getContextualResponse(question: string, type: string, name?: string): string {
  const lowerQ = question.toLowerCase();
  const label = name ?? 'the resource';

  if (type === 'asset') {
    if (lowerQ.includes('health') || lowerQ.includes('status')) {
      return `Health analysis for ${label}:

**Overall Health Score:** 7.8/10

**Metrics:**
- Uptime: 99.94% (last 30 days)
- Response Time: 145ms avg (P95: 320ms)
- Error Rate: 0.12%
- CPU Utilization: 42% avg

**Recent Issues:**
- Minor latency spike detected 2h ago (resolved)
- No critical incidents in the last 30 days

**Recommendation:** Health is good. Consider scaling down during off-peak hours to optimize costs.`;
    }

    if (lowerQ.includes('dependencies') || lowerQ.includes('downstream')) {
      return `Dependency analysis for ${label}:

**Direct Dependencies (3):**
1. Auth Service - healthy
2. Database Cluster - healthy
3. Cache Layer - degraded (elevated latency)

**Downstream Consumers (5):**
1. Customer Portal - 2.1K req/min
2. Mobile API Gateway - 890 req/min
3. Analytics Pipeline - batch processing
4. Notification Service - event-driven
5. Billing Service - 120 req/min

**Blast Radius:** If ${label} goes down, it would affect approximately 8 services and 15K active users.`;
    }

    return `Here is an analysis of ${label}:

- Type: Application Asset
- Health Score: 7.8/10
- Active Users: 1,200
- Annual Cost: $85,000
- Last Incident: 14 days ago

Would you like me to dive deeper into any specific aspect?`;
  }

  if (type === 'integration') {
    if (lowerQ.includes('run history') || lowerQ.includes('recent')) {
      return `Run history summary for ${label} (last 7 days):

**Total Runs:** 168
**Success Rate:** 94.6%
**Failed Runs:** 9

**Failure Breakdown:**
- Timeout errors: 5 (55%)
- Authentication failures: 3 (33%)
- Data validation errors: 1 (12%)

**Trend:** Failure rate increased from 2% to 5.4% this week, primarily due to timeout issues during peak hours.

**Recommendation:** Consider increasing the timeout threshold or implementing retry logic with exponential backoff.`;
    }

    if (lowerQ.includes('failure') || lowerQ.includes('error')) {
      return `Failure analysis for ${label}:

**Pattern Detected:** Recurring timeout failures during 2-4 PM UTC

**Root Cause (likely):** Upstream API rate limiting coincides with your batch processing window.

**Impact:**
- 9 failed runs this week
- Average recovery time: 12 minutes
- No data loss (idempotent operations)

**Suggested Fix:**
1. Stagger batch processing across a wider time window
2. Implement circuit breaker pattern
3. Add request queuing with backpressure

Would you like me to generate a playbook for implementing these fixes?`;
    }

    return `Integration overview for ${label}:

- Status: Active
- Last Run: 23 minutes ago (success)
- Success Rate (7d): 94.6%
- Avg Duration: 2.3s
- Data Volume: 12.4K records/day

Is there something specific you would like to know?`;
  }

  if (type === 'governance') {
    if (lowerQ.includes('explain') || lowerQ.includes('plain english')) {
      return `**${label} - Plain English Explanation:**

This policy requires that all production deployments go through a formal approval process before being executed. Specifically:

1. **Who it applies to:** Any team deploying code or configuration changes to production environments
2. **What is required:** A pull request review by at least 2 team members, plus sign-off from a team lead
3. **When it kicks in:** Before any deployment to production (staging and dev are exempt)
4. **Why it exists:** To prevent unauthorized or untested changes from reaching production, reducing incident risk

**Current Compliance:** 87% of deployments in the last 30 days followed this policy.`;
    }

    return `Policy overview for ${label}:

- Scope: Production environments
- Compliance Rate: 87%
- Violations (30d): 4
- Last Assessment: 2 days ago

Would you like me to explain this policy in detail or analyze compliance trends?`;
  }

  // General context fallback
  if (lowerQ.includes('retire') || lowerQ.includes('eliminate')) {
    return `Based on my analysis, I have identified 6 applications that are strong candidates for retirement:

**High Priority:**
1. **Legacy CRM** - Low business value (3.2/10), $180K annual cost
2. **Old Reporting Tool** - Redundant, 2 active users, $45K annual cost
3. **Archive System** - No active integrations, $28K annual cost

**Potential Annual Savings:** $253,000

Would you like me to create a retirement scenario?`;
  }

  if (lowerQ.includes('health') || lowerQ.includes('status')) {
    return `Integration health summary:

**Healthy (12):** Operating normally
**Degraded (3):** Data Service showing elevated latency (890ms avg)
**Critical (1):** Salesforce Sync - authentication failures

The most urgent issue is Salesforce Sync with 23 failures in the last hour due to an expired OAuth token.

**Recommended Action:** Re-authenticate the Salesforce connection.`;
  }

  if (lowerQ.includes('cost') || lowerQ.includes('saving')) {
    return `Cost optimization opportunities:

1. **Application Consolidation** - Merge 3 reporting tools, save $120K/yr
2. **Cloud Right-sizing** - Reduce over-provisioned resources, save $85K/yr
3. **License Optimization** - Remove unused CRM licenses, save $45K/yr

**Total Potential Savings:** $250K/year

Would you like a detailed analysis?`;
  }

  return `I understand you are asking about: "${question.substring(0, 60)}..."

Based on the current ${type} context:
- Portfolio: 41 applications
- 6 in the Eliminate quadrant
- Annual IT spend: $4.2M
- Average health score: 6.8/10

What would you like me to explore further?`;
}
