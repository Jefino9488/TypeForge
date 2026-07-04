# 3. Positive Reinforcement Learning Model

Date: 2026-07-01

## Status
Accepted

## Context
A major goal of TypeForge is to learn from user behavior to improve prediction accuracy over time. We needed a strategy for scoring and penalizing candidates.

## Alternatives Considered
- **Penalized Learning**: Increasing the weight of accepted words and decreasing the weight of ignored words.
  - *Cons*: Users often ignore perfectly good words because they want to type something else in that specific instance, or they make a typo. Penalizing words causes the dictionary to degrade over time.
- **Pure Statistical Language Model**: Building n-grams from everything the user types.
  - *Cons*: Extremely memory-intensive and computationally expensive for an input method.

## Decision
We chose a **Positive Reinforcement Model**. 
- Accepted predictions receive a massive boost (`+10`).
- Manually typed words that aren't common dictionary words receive a small nudge (`+2`).
- Ignored words receive **no penalty**.
- The base dictionary acts as a *prior*, and learned weights only ever add evidence.

## Consequences
- The learning database (`learning.db`) only stores positive evidence.
- Baseline dictionary words are never forgotten or penalized, ensuring a stable foundation.
- We must eventually implement a decay mechanism or upper bound for scores so that early learned words do not permanently dominate the rankings.
