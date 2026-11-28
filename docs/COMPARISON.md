# Ecosystem Comparison

| Dimension | `secondtake` | General Eval Frameworks (Promptfoo, Braintrust, Inspect) | `flighttape compare` |
| :--- | :--- | :--- | :--- |
| **Unit of comparison** | Full multi-turn agent session directories (`before/` vs `after/`) | Single prompt/response cells or custom Python scripts | Two individual tape files (`a` vs `b`) |
| **Hypothesis hygiene** | Mandatory pre-registration (`change`, `date`, `margins`, `min_pairs`) | Post-hoc threshold tuning common | Raw factual diff without verdict |
| **Behavioral axes** | Cost, wall-clock duration, rework moments, and error-rescue rate | Primarily output accuracy / LLM-as-judge | Token, duration, and event-kind deltas |
| **Model usage** | Zero LLM calls on `keep`, `revert`, and `< min_pairs`; judge consulted only in 1-regression tradeoff band | Often invokes LLM-as-judge on every row | Zero LLM calls |
| **Write safety** | Refuses `-o` inside `before/`, `after/`, or input paths | Writes local SQLite/JSON caches | Refuses overwriting inputs |
