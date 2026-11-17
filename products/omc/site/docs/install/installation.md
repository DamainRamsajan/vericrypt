# Installation

OMC is run from source today.

## What you need

- Python 3 and `pip`
- git

## Install and test

```
git clone https://github.com/DamainRamsajan/Operating-Margin-Controller-
cd Operating-Margin-Controller-
pip install -e ".[dev,spike]"
python scripts/make_vectors.py
pytest -q
pytest -q -m slow          # three statistical simulations
```

The repository ships a dev container, so in a GitHub Codespace the dependencies install and the tests run for you.

## Not available yet

There is no packaged release, hosted control plane or console. The repository's `docs/dev/MILESTONE-M0.md` lists what is built and what is next.
