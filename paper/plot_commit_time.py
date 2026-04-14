#!/usr/bin/env python3
"""Plot commit time per file operation."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plot_style as S
import matplotlib.pyplot as plt
import matplotlib.patheffects as pe
import numpy as np
from matplotlib.ticker import MaxNLocator
from matplotlib.patches import Patch
import csv

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <results-dir>", file=sys.stderr)
        sys.exit(1)

    results_dir = sys.argv[1]
    paper_dir = os.path.join(results_dir, 'paper')

    data_path = os.path.join(paper_dir, 'commit-time.csv')
    baseline_path = os.path.join(paper_dir, 'commit-time-baseline.csv')
    out_path = os.path.join(paper_dir, 'commit-time-figure.pdf')

    with open(data_path) as f:
        rows = list(csv.DictReader(f))

    baseline = {}
    with open(baseline_path) as f:
        for r in csv.DictReader(f):
            baseline[r['op']] = float(r['us_per_op'])

    ops = ['create', 'overwrite', 'rename', 'unlink']
    backends = ['YoloFS', 'OverlayFS', 'BranchFS']

    # Build lookup: (op, backend, metric) -> us_per_op
    lookup = {}
    for r in rows:
        lookup[(r['op'], r['backend'], r['metric'])] = float(r['us_per_op'])

    backend_colors = {
        'YoloFS':     S.TABLEAU10['blue'],
        'OverlayFS':  S.TABLEAU10['green'],
        'BranchFS':   S.TABLEAU10['orange'],
    }

    plt.rcParams.update({'font.size': 8.5, 'axes.labelsize': 8.5, 'xtick.labelsize': 8.5,
                          'ytick.labelsize': 8.5, 'legend.fontsize': 8.5})

    nb = len(backends)
    bar_height = 0.5 / nb
    y = np.arange(len(ops)) * 0.62

    # Compute commit panel range.
    commit_max = max(
        [lookup.get((op, b, 'commit'), 0) for op in ops for b in backends]
        + [baseline.get(op, 0) for op in ops]
    ) * 1.1

    fig, ax_commit = plt.subplots(1, 1, figsize=(2.7, 1.08))

    # Left panel: commit time.
    for bi, b in enumerate(backends):
        vals = [lookup.get((op, b, 'commit'), 0) for op in ops]
        offset = ((nb - 1) / 2 - bi) * bar_height
        ax_commit.barh(y + offset, vals, bar_height * 0.85,
                       color=backend_colors.get(b, '#999'),
                       edgecolor='white', linewidth=0.3,
                       label=b)
    group_half = bar_height * nb * 0.5
    for oi, op in enumerate(ops):
        val = baseline.get(op)
        if val is not None:
            ax_commit.vlines(val, y[oi] - group_half, y[oi] + group_half, **S.NATIVE_LINE_KW)
    ax_commit.set_xlabel('Commit time (\u00b5s/file)')
    ax_commit.xaxis.labelpad = 1
    ax_commit.set_xlim(left=0)
    ax_commit.set_yticks(y)
    ax_commit.set_yticklabels([op for op in ops], fontweight='bold')
    ax_commit.tick_params(axis='y', length=0, pad=4)

    # Set axis range.
    ax_commit.set_xlim(0, commit_max)
    ax_commit.xaxis.set_major_locator(MaxNLocator(nbins=6, integer=True))

    # Legend at top, shared.
    legend_items = [Patch(facecolor=backend_colors[b], edgecolor='white', label=b) for b in backends]
    legend_items.append(S.native_legend_handle('Base'))
    fig.legend(handles=legend_items, loc='upper center', bbox_to_anchor=(0.5, 0.995),
               ncol=nb + 1, handlelength=0.95, handletextpad=0.35,
               borderpad=0.15, columnspacing=0.55)

    fig.subplots_adjust(left=0.2, right=0.99, top=0.79, bottom=0.22)

    fig.savefig(out_path, bbox_inches='tight', dpi=300, metadata={"CreationDate": None})
    plt.close(fig)
    print(f"Figure written to {out_path}", file=sys.stderr)

if __name__ == '__main__':
    main()
