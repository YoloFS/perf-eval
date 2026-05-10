"""Plot commit time per file operation."""
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.ticker import MaxNLocator
from matplotlib.patches import Patch
from .plot_utils import (
    NATIVE_LINE_KW,
    TABLEAU10,
    native_legend_handle,
    read_csv_rows,
    save_figure,
)


def plot_commit(generated_dir):
    out_path = generated_dir / 'commit.pdf'

    rows = read_csv_rows(generated_dir, 'commit.csv')

    ops = ['create', 'overwrite', 'rename', 'unlink']
    backends = ['YoloFS', 'OverlayFS', 'BranchFS']

    lookup = {}
    baseline = {}
    for r in rows:
        if r['backend'] == 'Base':
            baseline[r['op']] = float(r['us_per_op'])
        else:
            lookup[(r['op'], r['backend'], r['metric'])] = float(r['us_per_op'])

    backend_colors = {
        'YoloFS': TABLEAU10['blue'],
        'OverlayFS': TABLEAU10['green'],
        'BranchFS': TABLEAU10['orange'],
    }

    plt.rcParams.update({'font.size': 8.5, 'axes.labelsize': 8.5, 'xtick.labelsize': 8.5,
                         'ytick.labelsize': 8.5, 'legend.fontsize': 8.5})

    nb = len(backends)
    bar_height = 0.5 / nb
    y = np.arange(len(ops)) * 0.62

    commit_max = max(
        [lookup.get((op, b, 'commit'), 0) for op in ops for b in backends]
        + [baseline.get(op, 0) for op in ops]
    ) * 1.1

    fig, ax_commit = plt.subplots(1, 1, figsize=(2.7, 1.08))

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
            ax_commit.vlines(
                val,
                y[oi] - group_half,
                y[oi] + group_half,
                **NATIVE_LINE_KW,
            )
    ax_commit.set_xlabel('Commit time (\u00b5s/file)')
    ax_commit.xaxis.labelpad = 1
    ax_commit.set_xlim(left=0)
    ax_commit.set_yticks(y)
    ax_commit.set_yticklabels([op for op in ops], fontweight='bold')
    ax_commit.tick_params(axis='y', length=0, pad=4)

    ax_commit.set_xlim(0, commit_max)
    ax_commit.xaxis.set_major_locator(MaxNLocator(nbins=6, integer=True))

    legend_items = [Patch(facecolor=backend_colors[b], edgecolor='white', label=b) for b in backends]
    legend_items.append(native_legend_handle('Base'))
    fig.legend(handles=legend_items, loc='upper center', bbox_to_anchor=(0.5, 0.995),
               ncol=nb + 1, handlelength=0.95, handletextpad=0.35,
               borderpad=0.15, columnspacing=0.55)

    fig.subplots_adjust(left=0.2, right=0.99, top=0.79, bottom=0.22)

    save_figure(fig, out_path)


