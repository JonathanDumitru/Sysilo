# Rationalization Engine

## Intent

Describe the workflow for discovering redundancy, scoring applications, and orchestrating consolidation.

## Lifecycle phases

```mermaid
flowchart LR
  Discover[Discover] --> Score[Score]
  Score --> Decide[Decide]
  Decide --> Migrate[Migrate]
```

## TIME quadrant

```mermaid
quadrantChart
  title TIME Quadrant (Tolerate, Invest, Migrate, Eliminate)
  x-axis Low Value --> High Value
  y-axis Low Health --> High Health
  quadrant-1 Invest
  quadrant-2 Tolerate
  quadrant-3 Eliminate
  quadrant-4 Migrate
  ExampleApp1: [0.8, 0.8]
  ExampleApp2: [0.2, 0.7]
  ExampleApp3: [0.3, 0.2]
  ExampleApp4: [0.7, 0.3]
```

## Scoring dimensions

- Business value
- Technical health
- Integration complexity
- Cost
- Strategic fit

## Open questions

- What data sources feed scoring in V1?
- Who owns approval for migration plans?
