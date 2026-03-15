import { useState, useRef, useEffect } from 'react';
import {
  Send,
  Sparkles,
  Loader2,
  User,
  Bot,
  Copy,
  ThumbsUp,
  ThumbsDown,
  Lightbulb,
  BarChart3,
  Shield,
  Workflow,
  Server,
  Database,
  TrendingUp,
} from 'lucide-react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

const QUICK_ACTIONS = [
  { label: 'Portfolio Health', icon: BarChart3, prompt: 'Give me an overview of my application portfolio health' },
  { label: 'Cost Savings', icon: TrendingUp, prompt: 'What are the top cost-saving opportunities across my portfolio?' },
  { label: 'Integration Status', icon: Workflow, prompt: 'Show me the health status of all my integrations' },
  { label: 'Compliance', icon: Shield, prompt: 'What is my current compliance posture across all frameworks?' },
  { label: 'Agent Health', icon: Server, prompt: 'Show me the status of all connected agents' },
  { label: 'Data Quality', icon: Database, prompt: 'What data quality issues should I address first?' },
];

const SUGGESTED_PROMPTS = [
  'What applications should I consider retiring?',
  'Which integrations had the most failures this week?',
  'Are there any policy violations that need immediate attention?',
  'Summarize my rationalization progress',
  'What are the top recommendations for reducing IT spend?',
  'Show me applications with the lowest health scores',
];

function getSimulatedResponse(question: string): string {
  const q = question.toLowerCase();

  if (q.includes('portfolio') || q.includes('overview')) {
    return `**Application Portfolio Health Summary**

**Total Applications:** 41
**Average Health Score:** 6.8/10

**Distribution by TIME Quadrant:**
- **Invest** (12): Strategic assets performing well
- **Tolerate** (15): Stable but low business value
- **Migrate** (8): High value but needs modernization
- **Eliminate** (6): Candidates for retirement

**Key Highlights:**
- 3 applications have critical health issues requiring immediate attention
- 8 applications are overdue for security patching
- Overall portfolio cost: $4.2M/year

**Top Actions:**
1. Review 6 elimination candidates (potential savings: $253K/yr)
2. Address 3 critical health issues
3. Plan migration for 8 high-value legacy apps

Would you like me to drill into any of these areas?`;
  }

  if (q.includes('cost') || q.includes('saving') || q.includes('spend')) {
    return `**Cost Optimization Analysis**

**Current Annual IT Spend:** $4.2M

**Top Savings Opportunities:**

1. **Application Consolidation** - $120K/yr
   Merge 3 redundant reporting tools (Tableau, Looker, custom BI)

2. **Cloud Right-sizing** - $85K/yr
   12 instances are over-provisioned by 40%+

3. **License Cleanup** - $45K/yr
   Remove 28 unused Salesforce licenses

4. **Legacy Retirement** - $253K/yr
   6 applications in the Eliminate quadrant

5. **Integration Optimization** - $32K/yr
   Consolidate 4 duplicate ETL pipelines

**Total Potential Savings: $535K/year (12.7% reduction)**

**Quick Wins (< 1 month):** Items 3 and 5
**Medium Term (1-3 months):** Items 1 and 2
**Long Term (3-6 months):** Item 4

Shall I create a scenario for any of these?`;
  }

  if (q.includes('integration') || q.includes('health')) {
    return `**Integration Health Dashboard**

**Total Integrations:** 16 active

**Status Breakdown:**
- Healthy: 12 (75%)
- Degraded: 3 (19%)
- Critical: 1 (6%)

**Critical Issues:**
- **Salesforce Sync** - Authentication failures (expired OAuth token)
  - 23 failures in last hour
  - Action: Re-authenticate connection

**Degraded Integrations:**
1. **Data Warehouse ETL** - Elevated latency (890ms avg, normally 200ms)
2. **HubSpot Contact Sync** - Intermittent timeouts during peak hours
3. **SAP Order Pipeline** - Retry rate increased to 8%

**Performance Trends (7 days):**
- Overall success rate: 94.6% (down from 97.2%)
- Average run duration: 2.3s (up from 1.8s)
- Total records processed: 1.2M

Would you like to investigate any specific integration?`;
  }

  if (q.includes('compliance') || q.includes('policy')) {
    return `**Compliance Posture Overview**

**Overall Compliance Score: 82.5%**

**Framework Scores:**
- SOC 2: 91% (28/31 controls passing)
- GDPR: 85% (17/20 controls passing)
- HIPAA: 78% (14/18 controls passing)
- ISO 27001: 76% (22/29 controls passing)

**Open Violations: 7**
- Critical: 1 (unencrypted PII data transfer)
- High: 2 (missing access controls)
- Medium: 3 (documentation gaps)
- Low: 1 (naming convention)

**Pending Approvals: 4**
- 2 production deployment approvals
- 1 new connection approval
- 1 data export request

**Recent Activity:**
- 3 violations resolved this week
- Last assessment: 2 days ago
- Next scheduled assessment: Friday

Would you like me to explain any specific compliance issue?`;
  }

  if (q.includes('agent')) {
    return `**Agent Status Overview**

**Connected Agents: 3/4**

| Agent | Status | Version | Tasks | Location |
|-------|--------|---------|-------|----------|
| prod-agent-01 | Connected | v1.2.0 | 3/10 | AWS us-east-1 |
| prod-agent-02 | Connected | v1.2.0 | 5/10 | AWS us-west-2 |
| on-prem-agent | Connected | v1.1.5 | 2/5 | On-Premise DC |
| dev-agent | Disconnected | v1.2.0 | 0/5 | Local Dev |

**Alerts:**
- dev-agent has been disconnected for 2 hours
- on-prem-agent is running an older version (1.1.5 vs 1.2.0)

**Recommendations:**
1. Investigate dev-agent disconnect
2. Schedule on-prem-agent upgrade to v1.2.0
3. Consider scaling prod-agent-02 (50% task capacity used)

Need details on any specific agent?`;
  }

  if (q.includes('data quality') || q.includes('quality')) {
    return `**Data Quality Report**

**Overall Quality Score: 94.2%**

**Issues by Priority:**

1. **customers table** - 87% quality
   - 156 records with missing email addresses
   - 23 duplicate entries detected
   - Action: Run deduplication pipeline

2. **invoices view** - 87% quality
   - Schema drift detected (3 new columns)
   - 12 records with null required fields
   - Action: Update canonical model

3. **orders table** - 95% quality
   - Minor: 45 records with future dates
   - Action: Add validation rule

**PII Fields Detected: 23**
- 18 properly encrypted
- 5 need encryption review

**Lineage Coverage: 89%**
- 342 lineage links mapped
- 42 entities missing lineage data

Would you like me to create a remediation plan?`;
  }

  if (q.includes('retire') || q.includes('eliminat')) {
    return `**Retirement Candidates Analysis**

I've identified **6 applications** in the Eliminate quadrant:

**High Priority (recommend immediate action):**
1. **Legacy CRM v2** - Health: 3.2/10, Value: 2.1/10
   - Cost: $180K/yr, 2 active users
   - Replacement: Salesforce (already in portfolio)

2. **Old Reporting Tool** - Health: 4.1/10, Value: 1.8/10
   - Cost: $45K/yr, redundant with Tableau
   - No active integrations

3. **Archive System** - Health: 5.0/10, Value: 2.5/10
   - Cost: $28K/yr, data can migrate to S3

**Medium Priority:**
4. **Internal Wiki** - $15K/yr (replace with Confluence)
5. **Custom Logger** - $12K/yr (replace with Datadog)
6. **Legacy Auth Module** - $8K/yr (absorbed by SSO)

**Total Savings: $288K/year**
**Migration Effort: ~3 months**

Shall I create a retirement scenario with migration plans?`;
  }

  if (q.includes('failure') || q.includes('error')) {
    return `**Integration Failure Analysis (Last 7 Days)**

**Total Failures: 47**

**Top Failing Integrations:**
1. **Salesforce Sync** - 23 failures (49%)
   - Root cause: Expired OAuth token
   - Impact: Contact data 6 hours stale

2. **Data Warehouse ETL** - 12 failures (26%)
   - Root cause: Source table schema change
   - Impact: Analytics dashboards incomplete

3. **SAP Order Pipeline** - 8 failures (17%)
   - Root cause: API rate limiting during peak
   - Impact: Order processing delays (avg 12 min)

4. **Other** - 4 failures (8%)
   - Minor timeout issues across 3 integrations

**Trends:**
- Failure rate up 3.2% vs last week
- Most failures occur 2-4 PM UTC
- MTTR (Mean Time to Recovery): 18 minutes

**Immediate Actions:**
1. Rotate Salesforce OAuth token
2. Update ETL schema mapping
3. Implement rate limiting backoff for SAP

Would you like me to create automated playbooks for these?`;
  }

  if (q.includes('rationalization') || q.includes('progress')) {
    return `**Rationalization Progress Summary**

**Portfolio Overview:**
- Total applications assessed: 41/48 (85%)
- TIME quadrant distribution updated: 3 days ago

**Active Scenarios: 3**
1. "Q1 Consolidation" - 8 apps, $180K projected savings, In Progress
2. "Legacy Migration" - 5 apps, analyzing phase
3. "Cloud Optimization" - 12 apps, completed analysis

**Completed This Quarter:**
- 2 applications retired (saved $63K/yr)
- 1 migration completed (Legacy CRM → Salesforce)
- 4 new assessments completed

**Pending Actions:**
- 7 applications awaiting initial assessment
- 3 retirement playbooks need approval
- 1 scenario needs stakeholder review

**Key Metrics:**
- Realized savings YTD: $63K
- Projected savings (full year): $535K
- Portfolio health improvement: +0.4 points

What area would you like to explore further?`;
  }

  return `I can help you analyze your Sysilo platform. Here's what I can assist with:

- **Portfolio Analysis** - Application health, costs, and optimization
- **Integration Monitoring** - Status, failures, and performance
- **Compliance & Governance** - Policy adherence and violations
- **Rationalization** - TIME analysis, retirement candidates, scenarios
- **Operations** - Agent health, alerts, and incidents
- **Data Quality** - Entity health, lineage, and PII tracking

What would you like to explore? Try asking about specific areas or use the quick actions above.`;
}

export function AIAssistantPage() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingText]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const simulateStreaming = async (text: string) => {
    setStreamingText('');
    const words = text.split(' ');
    for (let i = 0; i < words.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 20));
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
      await new Promise((resolve) => setTimeout(resolve, 600));
      const responseText = getSimulatedResponse(text);
      await simulateStreaming(responseText);

      const aiResponse: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: responseText,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, aiResponse]);
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

  const showWelcome = messages.length === 0 && !streamingText;

  return (
    <div className="h-[calc(100vh-7rem)] md:h-[calc(100vh-8rem)] flex flex-col -mx-4 md:-mx-6 -my-4 md:-my-6">
      {/* Chat area */}
      <div className="flex-1 overflow-y-auto">
        {showWelcome ? (
          <div className="max-w-3xl mx-auto px-4 py-8 md:py-12">
            {/* Welcome header */}
            <div className="text-center mb-8">
              <div className="inline-flex p-4 bg-status-ai/10 rounded-2xl mb-4">
                <Sparkles className="w-8 h-8 text-status-ai" />
              </div>
              <h1 className="text-2xl md:text-3xl font-bold text-white mb-2">AI Assistant</h1>
              <p className="text-gray-400 max-w-md mx-auto">
                Ask questions about your portfolio, integrations, compliance, and more. Get instant insights powered by AI.
              </p>
            </div>

            {/* Quick actions */}
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3 mb-8">
              {QUICK_ACTIONS.map((action) => (
                <button
                  key={action.label}
                  onClick={() => handleSend(action.prompt)}
                  className="glass-card p-4 text-left hover:border-status-ai/30 transition-all group"
                >
                  <action.icon className="w-5 h-5 text-gray-400 group-hover:text-status-ai mb-2 transition-colors" />
                  <span className="text-sm font-medium text-gray-300 group-hover:text-white transition-colors">
                    {action.label}
                  </span>
                </button>
              ))}
            </div>

            {/* Suggested prompts */}
            <div>
              <div className="flex items-center gap-2 mb-3">
                <Lightbulb className="w-4 h-4 text-gray-500" />
                <span className="text-xs font-medium text-gray-500 uppercase tracking-wider">Suggested Questions</span>
              </div>
              <div className="space-y-2">
                {SUGGESTED_PROMPTS.map((prompt, i) => (
                  <button
                    key={i}
                    onClick={() => handleSend(prompt)}
                    className="block w-full text-left px-4 py-3 text-sm text-gray-400 glass-card hover:border-surface-border-strong hover:text-gray-200 transition-all"
                  >
                    {prompt}
                  </button>
                ))}
              </div>
            </div>
          </div>
        ) : (
          <div className="max-w-3xl mx-auto px-4 py-6 space-y-6">
            {messages.map((message) => (
              <div
                key={message.id}
                className={`flex gap-3 ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
              >
                {message.role === 'assistant' && (
                  <div className="flex-shrink-0 w-8 h-8 rounded-full bg-status-ai/20 flex items-center justify-center mt-1">
                    <Bot className="w-4 h-4 text-status-ai" />
                  </div>
                )}
                <div
                  className={`max-w-[85%] ${
                    message.role === 'user'
                      ? 'bg-primary-600/80 text-white rounded-2xl rounded-tr-md px-4 py-3'
                      : 'glass-panel px-4 py-3 rounded-2xl rounded-tl-md'
                  }`}
                >
                  <div className="text-sm whitespace-pre-wrap leading-relaxed">
                    {message.content}
                  </div>
                  {message.role === 'assistant' && (
                    <div className="flex items-center gap-2 mt-3 pt-2 border-t border-surface-border">
                      <button
                        onClick={() => copyToClipboard(message.content)}
                        className="p-1 text-gray-500 hover:text-gray-300 rounded"
                        title="Copy"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>
                      <button className="p-1 text-gray-500 hover:text-green-400 rounded" title="Helpful">
                        <ThumbsUp className="w-3.5 h-3.5" />
                      </button>
                      <button className="p-1 text-gray-500 hover:text-red-400 rounded" title="Not helpful">
                        <ThumbsDown className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )}
                </div>
                {message.role === 'user' && (
                  <div className="flex-shrink-0 w-8 h-8 rounded-full bg-surface-overlay flex items-center justify-center mt-1">
                    <User className="w-4 h-4 text-gray-400" />
                  </div>
                )}
              </div>
            ))}

            {/* Streaming */}
            {isLoading && streamingText && (
              <div className="flex gap-3">
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-status-ai/20 flex items-center justify-center mt-1">
                  <Bot className="w-4 h-4 text-status-ai" />
                </div>
                <div className="glass-panel px-4 py-3 rounded-2xl rounded-tl-md max-w-[85%]">
                  <p className="text-sm whitespace-pre-wrap leading-relaxed text-gray-300">
                    {streamingText}
                    <span className="inline-block w-1.5 h-4 bg-status-ai ml-0.5 animate-pulse" />
                  </p>
                </div>
              </div>
            )}

            {/* Loading */}
            {isLoading && !streamingText && (
              <div className="flex gap-3">
                <div className="flex-shrink-0 w-8 h-8 rounded-full bg-status-ai/20 flex items-center justify-center mt-1">
                  <Bot className="w-4 h-4 text-status-ai" />
                </div>
                <div className="glass-panel px-4 py-3 rounded-2xl rounded-tl-md">
                  <div className="flex items-center gap-2 text-gray-500">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span className="text-sm">Analyzing your platform data...</span>
                  </div>
                </div>
              </div>
            )}

            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Input area */}
      <div className="border-t border-surface-border bg-surface-raised/80 backdrop-blur-glass px-4 py-3">
        <div className="max-w-3xl mx-auto">
          <div className="flex items-end gap-3">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleInputKeyDown}
              placeholder="Ask about your portfolio, integrations, compliance..."
              rows={1}
              className="flex-1 glass-input text-sm resize-none"
              style={{ minHeight: '44px', maxHeight: '120px' }}
            />
            <button
              onClick={() => handleSend()}
              disabled={!input.trim() || isLoading}
              className="p-3 bg-status-ai/20 text-status-ai rounded-lg hover:bg-status-ai/30 disabled:opacity-30 disabled:cursor-not-allowed transition-colors flex-shrink-0"
            >
              <Send className="w-5 h-5" />
            </button>
          </div>
          <p className="text-[10px] text-gray-600 mt-2 text-center">
            AI responses are simulated. Production will connect to the Sysilo AI Engine.
          </p>
        </div>
      </div>
    </div>
  );
}
