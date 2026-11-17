# User manual

OMC is a closed-loop system that measures and proves the financial value of work shared between people, AI models and agents. It senses how work runs, prices the risk of each step, decides who handles each case within a risk budget, and proves the effect with evidence you can re-run.

> **Status.** OMC is in the design phase. The M0 walking skeleton runs on **synthetic data only**; nothing has been validated on real tickets yet. Read the Reference page's limitations before relying on any result.

## Run the demo

In a Python environment with the repository checked out:

```
pip install -e ".[dev,spike]"
python scripts/make_vectors.py
pytest -q                  # fast tests
python scripts/demo.py     # end-to-end demo, about 15 seconds
```

The demo shows three outcomes of a model-migration audit:

| Scenario | Verdict | Exit code |
| --- | --- | --- |
| benign (replacement is no worse) | promoted | 0 |
| regression (replacement is worse) | rejected | 20 |
| unclear (too close to call) | held | 10 |

It then verifies the signed statements and hash chains, and prints a model exposure table with a retirement calendar.

## The four entry audits

1. **Certified model migration qualification** with a CI promotion gate
2. **Outcome billing audit and vendor bake-off**
3. **Action authority audit** for money that agents move
4. **Human access and disclosure compliance audit**, a module for chatbot laws

What is built today is the first: the paired audit and the gate. The others are in the backlog.
