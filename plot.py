#!/usr/bin/env -S uv run
"""Render paper charts. Default: render all into ../paper/generated/."""
import argparse
import sys
from pathlib import Path

from scripts.plot_checkpoint import plot_checkpoint
from scripts.plot_commit import plot_commit
from scripts.plot_dev import plot_dev
from scripts.plot_metadata import plot_metadata

CHARTS = {
    'metadata':   plot_metadata,
    'commit':     plot_commit,
    'checkpoint': plot_checkpoint,
    'dev':        plot_dev,
}

DEFAULT_DIR = Path(__file__).resolve().parent.parent / "paper" / "generated"


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('charts', nargs='*', metavar='CHART',
                   help=f'Charts to render: {", ".join(CHARTS)} (default: all)')
    p.add_argument('-o', '--output-dir', type=Path, default=DEFAULT_DIR,
                   help=f'Output directory (default: {DEFAULT_DIR})')
    args = p.parse_args()

    charts = args.charts or list(CHARTS)
    unknown = [c for c in charts if c not in CHARTS]
    if unknown:
        sys.exit(f"error: unknown chart(s): {', '.join(unknown)} "
                 f"(choose from {', '.join(CHARTS)})")
    if not args.output_dir.is_dir():
        sys.exit(f"error: output dir does not exist: {args.output_dir}")

    for name in charts:
        try:
            CHARTS[name](args.output_dir)
        except Exception as e:
            print(f"warning: {name}: {e}", file=sys.stderr)


if __name__ == '__main__':
    main()
