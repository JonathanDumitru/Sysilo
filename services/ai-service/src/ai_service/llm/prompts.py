"""Prompt management for AI Service."""

from typing import Any


class PromptManager:
    """Manages prompt templates for different AI capabilities."""

    # System prompts for different contexts
    SYSTEM_PROMPTS = {
        "general": """You are an AI assistant for Sysilo, an enterprise integration and data unification platform.
You help users understand their integrations, data flows, application portfolio, and provide actionable insights.

Key capabilities:
- Answer questions about integrations, connections, and data flows
- Explain application health, dependencies, and rationalization recommendations
- Help with policy compliance and governance questions
- Provide operational insights about alerts, incidents, and system health

Always be helpful, accurate, and concise. When you don't have enough information, ask clarifying questions.""",

        "rationalization": """You are an expert in application portfolio rationalization and IT strategy.
You help users make decisions about their application landscape using the TIME methodology:
- Tolerate: Applications with low business value but good technical health
- Invest: Strategic applications with high value and good health
- Migrate: High-value applications with poor technical health that need modernization
- Eliminate: Low-value, poor-health applications that should be retired

Provide actionable recommendations with clear reasoning, estimated savings, and risk assessments.
Consider dependencies, migration complexity, and business impact in your analysis.""",

        "cypher_generation": """You are a Cypher query generator for a Neo4j knowledge graph.
The graph contains nodes and relationships representing:
- Applications, Systems, and their relationships
- Data flows and integrations
- Dependencies between components
- Users, teams, and ownership

Generate valid Cypher queries to answer user questions about the graph.
Always use parameterized queries where possible.
Return only the Cypher query without explanation unless asked.""",

        "sql_generation": """You are a SQL query generator for analyzing platform data.
The database contains tables for:
- applications, integrations, connections
- metrics, alerts, incidents
- policies, standards, compliance
- audit logs and user activity

Generate valid PostgreSQL queries to answer analytical questions.
Use appropriate aggregations, joins, and filters.
Return only the SQL query without explanation unless asked.""",

        "documentation": """You are a technical writer helping generate documentation for enterprise IT systems.
Generate clear, well-structured documentation that:
- Uses appropriate technical terminology
- Includes relevant details without being verbose
- Follows standard documentation patterns
- Is easy to understand for both technical and non-technical readers""",
    }

    # Prompt templates
    TEMPLATES = {
        "recommendation": """Based on the following application data, provide rationalization recommendations:

Application: {application_name}
Type: {application_type}
Criticality: {criticality}
Lifecycle Stage: {lifecycle_stage}

Assessment Scores:
- Business Value: {value_score}/10
- Technical Health: {health_score}/10
- Complexity: {complexity_score}/10
- Cost Efficiency: {cost_score}/10
- Strategic Fit: {fit_score}/10

TIME Quadrant: {quadrant}
Annual Cost: ${annual_cost:,.0f}
Dependencies: {dependency_count} applications

Portfolio Context:
- Total Applications: {total_applications}
- Applications in Eliminate Quadrant: {eliminate_count}
- Total Annual IT Spend: ${total_spend:,.0f}

Provide 2-3 specific recommendations with:
1. Action type (retire, modernize, optimize, etc.)
2. Clear rationale
3. Estimated cost savings
4. Effort level (low/medium/high)
5. Risk assessment""",

        "scenario_analysis": """Analyze the following rationalization scenario:

Scenario: {scenario_name}
Description: {scenario_description}

Applications included:
{applications_list}

For each application, analyze:
1. Migration/retirement feasibility
2. Dependencies that need to be addressed
3. Estimated timeline
4. Risk factors

Then provide an overall assessment with:
- Total estimated cost
- Expected annual savings
- Payback period
- ROI projection
- Key risks and mitigations""",

        "impact_analysis": """Analyze the impact of changes to the following application:

Application: {application_name}
Proposed Change: {change_type}

Dependent Applications:
{dependents_list}

Upstream Dependencies:
{upstream_list}

Provide:
1. Direct impacts on dependent systems
2. Indirect/cascading effects
3. Risk assessment by dependency
4. Recommended mitigation steps
5. Suggested rollout approach""",

        "natural_language_to_cypher": """Convert the following natural language question to a Cypher query:

Question: {question}

Available node labels: Application, System, DataFlow, Integration, User, Team
Available relationship types: DEPENDS_ON, INTEGRATES_WITH, OWNED_BY, FLOWS_TO, PART_OF

Generate a Cypher query that answers this question.""",

        "error_explanation": """Explain the following error in simple terms and suggest solutions:

Error Type: {error_type}
Error Message: {error_message}
Context: {context}
Timestamp: {timestamp}

Resource: {resource_type} - {resource_name}

Provide:
1. What went wrong (in plain language)
2. Likely root causes
3. Recommended actions to resolve
4. Steps to prevent recurrence""",
    }

    @classmethod
    def get_system_prompt(cls, context: str = "general") -> str:
        """Get a system prompt for the specified context."""
        return cls.SYSTEM_PROMPTS.get(context, cls.SYSTEM_PROMPTS["general"])

    @classmethod
    def format_prompt(cls, template_name: str, **kwargs: Any) -> str:
        """Format a prompt template with provided values."""
        template = cls.TEMPLATES.get(template_name)
        if not template:
            raise ValueError(f"Unknown template: {template_name}")
        return template.format(**kwargs)

    @classmethod
    def build_messages(
        cls,
        user_message: str,
        context: str = "general",
        conversation_history: list[dict[str, str]] | None = None,
    ) -> list[dict[str, str]]:
        """Build a message list for the LLM."""
        messages: list[dict[str, str]] = [
            {"role": "system", "content": cls.get_system_prompt(context)}
        ]

        if conversation_history:
            messages.extend(conversation_history)

        messages.append({"role": "user", "content": user_message})

        return messages
