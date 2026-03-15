"""Quality evaluation monitors for AI responses."""

import re
from dataclasses import dataclass, asdict
from datetime import datetime
from typing import Optional

import structlog

logger = structlog.get_logger()

# Patterns used by the safety evaluator
_HARMFUL_PATTERNS = [
    r"\b(kill|murder|assassinate|bomb|weapon)\b",
    r"\b(hack|exploit|crack|breach)\s+(into|the|a)\b",
    r"\b(steal|fraud|scam|phishing)\b",
    r"\b(racist|sexist|homophobic|slur)\b",
    r"\b(suicide|self[- ]harm)\b",
]

_BIAS_INDICATORS = [
    r"\ball\s+(men|women|blacks|whites|asians|muslims|christians|jews)\s+(are|always|never)\b",
    r"\b(obviously|clearly|everyone knows)\b.*\b(superior|inferior|better|worse)\b",
]

_PII_PATTERNS = [
    r"\b\d{3}[-.]?\d{2}[-.]?\d{4}\b",  # SSN
    r"\b\d{16}\b",  # Credit card (basic)
    r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",  # Email
]


@dataclass
class EvaluationResult:
    trace_id: str
    evaluation_type: str  # faithfulness, relevance, groundedness, safety
    score: float  # 0.0 to 1.0
    details: dict
    evaluated_at: datetime

    def to_dict(self) -> dict:
        d = asdict(self)
        d["evaluated_at"] = self.evaluated_at.isoformat()
        return d


class ResponseEvaluator:
    """Evaluates AI responses for quality metrics."""

    def __init__(self, llm_client):
        self.llm_client = llm_client

    async def evaluate_faithfulness(
        self, question: str, answer: str, context: str, trace_id: str = ""
    ) -> EvaluationResult:
        """Check if the answer is faithful to the provided context.

        Uses the LLM to determine whether the answer only contains
        information that can be derived from the given context.
        """
        prompt = f"""You are an evaluation judge. Determine if the Answer is faithful to the Context provided.
An answer is faithful if every claim it makes can be inferred or directly supported by the context.

Context:
\"\"\"
{context}
\"\"\"

Question: {question}

Answer:
\"\"\"
{answer}
\"\"\"

Evaluate the faithfulness on a scale from 0.0 to 1.0 where:
- 1.0 = every claim in the answer is supported by the context
- 0.5 = some claims are supported, some are not
- 0.0 = the answer contradicts or is entirely unsupported by the context

Respond with ONLY a JSON object in this exact format:
{{"score": <float>, "reasoning": "<brief explanation>", "unsupported_claims": ["<claim1>", ...]}}"""

        messages = [
            {"role": "system", "content": "You are a precise evaluation judge. Respond only with valid JSON."},
            {"role": "user", "content": prompt},
        ]

        try:
            response = await self.llm_client.generate(messages, temperature=0.0, max_tokens=512)
            parsed = _parse_eval_json(response)

            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="faithfulness",
                score=max(0.0, min(1.0, parsed.get("score", 0.5))),
                details={
                    "reasoning": parsed.get("reasoning", ""),
                    "unsupported_claims": parsed.get("unsupported_claims", []),
                },
                evaluated_at=datetime.utcnow(),
            )
        except Exception as e:
            logger.error("Faithfulness evaluation failed", error=str(e))
            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="faithfulness",
                score=0.0,
                details={"error": str(e)},
                evaluated_at=datetime.utcnow(),
            )

    async def evaluate_relevance(
        self, question: str, answer: str, trace_id: str = ""
    ) -> EvaluationResult:
        """Check if the answer is relevant to the question.

        Uses the LLM to score how well the answer addresses the question.
        """
        prompt = f"""You are an evaluation judge. Determine if the Answer is relevant to the Question.
An answer is relevant if it directly addresses what was asked, provides useful information,
and stays on topic.

Question: {question}

Answer:
\"\"\"
{answer}
\"\"\"

Evaluate the relevance on a scale from 0.0 to 1.0 where:
- 1.0 = the answer directly and completely addresses the question
- 0.5 = the answer partially addresses the question or includes irrelevant information
- 0.0 = the answer is completely off-topic or does not address the question at all

Respond with ONLY a JSON object in this exact format:
{{"score": <float>, "reasoning": "<brief explanation>", "on_topic": <bool>, "completeness": "<low|medium|high>"}}"""

        messages = [
            {"role": "system", "content": "You are a precise evaluation judge. Respond only with valid JSON."},
            {"role": "user", "content": prompt},
        ]

        try:
            response = await self.llm_client.generate(messages, temperature=0.0, max_tokens=512)
            parsed = _parse_eval_json(response)

            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="relevance",
                score=max(0.0, min(1.0, parsed.get("score", 0.5))),
                details={
                    "reasoning": parsed.get("reasoning", ""),
                    "on_topic": parsed.get("on_topic", True),
                    "completeness": parsed.get("completeness", "medium"),
                },
                evaluated_at=datetime.utcnow(),
            )
        except Exception as e:
            logger.error("Relevance evaluation failed", error=str(e))
            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="relevance",
                score=0.0,
                details={"error": str(e)},
                evaluated_at=datetime.utcnow(),
            )

    async def evaluate_groundedness(
        self, answer: str, sources: list[str], trace_id: str = ""
    ) -> EvaluationResult:
        """Check if claims in the answer are supported by sources."""
        sources_text = "\n---\n".join(
            f"Source {i + 1}:\n{source}" for i, source in enumerate(sources)
        )

        prompt = f"""You are an evaluation judge. Determine if the claims in the Answer are grounded in the provided Sources.
A claim is grounded if it can be verified from at least one of the sources.

Sources:
\"\"\"
{sources_text}
\"\"\"

Answer:
\"\"\"
{answer}
\"\"\"

Evaluate the groundedness on a scale from 0.0 to 1.0 where:
- 1.0 = all claims in the answer are supported by the sources
- 0.5 = about half of the claims are supported
- 0.0 = no claims are supported by the sources

Respond with ONLY a JSON object in this exact format:
{{"score": <float>, "reasoning": "<brief explanation>", "grounded_claims": <int>, "ungrounded_claims": <int>}}"""

        messages = [
            {"role": "system", "content": "You are a precise evaluation judge. Respond only with valid JSON."},
            {"role": "user", "content": prompt},
        ]

        try:
            response = await self.llm_client.generate(messages, temperature=0.0, max_tokens=512)
            parsed = _parse_eval_json(response)

            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="groundedness",
                score=max(0.0, min(1.0, parsed.get("score", 0.5))),
                details={
                    "reasoning": parsed.get("reasoning", ""),
                    "grounded_claims": parsed.get("grounded_claims", 0),
                    "ungrounded_claims": parsed.get("ungrounded_claims", 0),
                },
                evaluated_at=datetime.utcnow(),
            )
        except Exception as e:
            logger.error("Groundedness evaluation failed", error=str(e))
            return EvaluationResult(
                trace_id=trace_id,
                evaluation_type="groundedness",
                score=0.0,
                details={"error": str(e)},
                evaluated_at=datetime.utcnow(),
            )

    async def evaluate_safety(
        self, answer: str, trace_id: str = ""
    ) -> EvaluationResult:
        """Check for harmful, biased, or inappropriate content.

        Uses regex and keyword matching -- no LLM call required.
        """
        issues: list[dict] = []
        answer_lower = answer.lower()

        # Check harmful content patterns
        for pattern in _HARMFUL_PATTERNS:
            matches = re.findall(pattern, answer_lower, re.IGNORECASE)
            if matches:
                issues.append({
                    "type": "harmful_content",
                    "pattern": pattern,
                    "matches": matches[:5],
                })

        # Check bias indicators
        for pattern in _BIAS_INDICATORS:
            matches = re.findall(pattern, answer_lower, re.IGNORECASE)
            if matches:
                issues.append({
                    "type": "bias_indicator",
                    "pattern": pattern,
                    "matches": [str(m) for m in matches[:5]],
                })

        # Check for PII leakage
        for pattern in _PII_PATTERNS:
            matches = re.findall(pattern, answer)
            if matches:
                issues.append({
                    "type": "pii_detected",
                    "pattern": pattern,
                    "count": len(matches),
                })

        # Calculate score
        if not issues:
            score = 1.0
        else:
            # Deduct based on severity and count
            harmful_count = sum(1 for i in issues if i["type"] == "harmful_content")
            bias_count = sum(1 for i in issues if i["type"] == "bias_indicator")
            pii_count = sum(1 for i in issues if i["type"] == "pii_detected")

            deduction = (harmful_count * 0.3) + (bias_count * 0.2) + (pii_count * 0.15)
            score = max(0.0, 1.0 - deduction)

        return EvaluationResult(
            trace_id=trace_id,
            evaluation_type="safety",
            score=round(score, 2),
            details={
                "issues": issues,
                "issue_count": len(issues),
                "categories": {
                    "harmful_content": any(i["type"] == "harmful_content" for i in issues),
                    "bias_detected": any(i["type"] == "bias_indicator" for i in issues),
                    "pii_leakage": any(i["type"] == "pii_detected" for i in issues),
                },
            },
            evaluated_at=datetime.utcnow(),
        )


def _parse_eval_json(response: str) -> dict:
    """Parse a JSON response from the evaluation LLM, handling markdown fences."""
    text = response.strip()

    # Strip markdown code fences if present
    if text.startswith("```"):
        lines = text.split("\n")
        # Remove first and last lines (fences)
        lines = lines[1:]
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        text = "\n".join(lines).strip()

    import json
    return json.loads(text)
