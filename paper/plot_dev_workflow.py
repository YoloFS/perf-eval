#!/usr/bin/env python3
"""Plot developer workflow phase breakdown."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import plot_style as S
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib.patheffects as pe
import numpy as np
from matplotlib.ticker import MaxNLocator
import csv

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <results-dir>", file=sys.stderr)
        sys.exit(1)

    results_dir = sys.argv[1]
    paper_dir = os.path.join(results_dir, 'paper')
    out_path = os.path.join(paper_dir, 'dev-workflow-figure.pdf')

    with open(os.path.join(paper_dir, 'dev-workflow.csv')) as f:
        rows = list(csv.DictReader(f))

    facets = ['Worktree', 'Init. Build', 'Read', 'Edit', 'Incr. Build', 'Git']
    backends = ['YoloFS', 'OverlayFS']
    colors = {
        'YoloFS': S.BACKEND_COLORS['YoloFS'],
        'OverlayFS': S.BACKEND_COLORS['OverlayFS'],
    }
    native_line_kw = dict(S.NATIVE_LINE_KW)
    native_line_kw['path_effects'] = [pe.withStroke(linewidth=1.8, foreground='white', alpha=0.45)]
    native_handle = S.native_legend_handle('Base')
    native_handle.set_path_effects(native_line_kw['path_effects'])

    plt.rcParams.update({'font.size': 7.3, 'axes.labelsize': 7.3, 'xtick.labelsize': 7.3,
                          'ytick.labelsize': 7.3, 'legend.fontsize': 7.3})

    fig = plt.figure(figsize=(2.85, 1.82))
    gs = fig.add_gridspec(2, 4, width_ratios=[0.56, 0.56, 0.56, 0.82], wspace=0.42, hspace=0.42)
    axes = [
        fig.add_subplot(gs[0, 0]),
        fig.add_subplot(gs[0, 1]),
        fig.add_subplot(gs[0, 2]),
        fig.add_subplot(gs[1, 0]),
        fig.add_subplot(gs[1, 1]),
        fig.add_subplot(gs[1, 2]),
    ]
    ax_total = fig.add_subplot(gs[:, 3])

    def facet_rows(name):
        return [r for r in rows if r['facet'] == name]

    for idx, facet in enumerate(facets):
        ax = axes[idx]
        fr = facet_rows(facet)
        x = np.array([0.0, 0.18])
        run = [next((float(r['run_s']) for r in fr if r['backend'] == b), 0.0) for b in backends]
        chk = [next((float(r['checkpoint_s']) for r in fr if r['backend'] == b), 0.0) for b in backends]
        native_run = next((float(r['native_run_s']) for r in fr), 0.0)
        for i, backend in enumerate(backends):
            color = colors[backend]
            ax.bar(x[i], run[i], color=color, edgecolor=color, linewidth=0.6, width=0.14)
            ax.bar(x[i], chk[i], bottom=run[i], color='white', edgecolor=color, linewidth=0.6,
                   width=0.14)
            ax.bar(x[i], chk[i], bottom=run[i], color='none', edgecolor=color, linewidth=0.0,
                   hatch='////', width=0.14, zorder=3)
        ax.axhline(native_run, **native_line_kw)
        ax.set_xticks([])
        ymax = max(max((r + c) for r, c in zip(run, chk)), native_run, 0.0)
        ax.set_ylim(0, ymax * 1.08 if ymax > 0 else 1.0)
        ax.set_xlim(-0.14, 0.32)
        ax.yaxis.set_major_locator(MaxNLocator(nbins=3))
        ax.text(0.5, -0.095, facet, transform=ax.transAxes, ha='center', va='top',
                fontsize=7.3, fontweight='bold')

    ax = ax_total
    x = np.array([0.0, 0.2])
    run_total = [next((float(r['run_total_s']) for r in rows if r['facet'] == facets[0] and r['backend'] == b), 0.0) for b in backends]
    commit = [next((float(r['commit_s']) for r in rows if r['facet'] == facets[0] and r['backend'] == b), 0.0) for b in backends]
    checkpoint_total = [next((float(r['checkpoint_total_s']) for r in rows if r['facet'] == facets[0] and r['backend'] == b), 0.0) for b in backends]
    native_total = next((float(r['native_total_s']) for r in rows if r['facet'] == facets[0]), 0.0)
    stack_base = np.array(run_total) + np.array(checkpoint_total)
    for i, backend in enumerate(backends):
        color = colors[backend]
        ax.bar(x[i], run_total[i], color=color, edgecolor=color, linewidth=0.6, width=0.14)
        ax.bar(x[i], checkpoint_total[i], bottom=run_total[i], color='white', edgecolor=color, linewidth=0.6,
               width=0.14)
        ax.bar(x[i], checkpoint_total[i], bottom=run_total[i], color='none', edgecolor=color, linewidth=0.0,
               hatch='////', width=0.14, zorder=3)
        ax.bar(x[i], commit[i], bottom=stack_base[i], color='white', edgecolor=color, linewidth=0.6,
               width=0.14)
        ax.bar(x[i], commit[i], bottom=stack_base[i], color='none', edgecolor=color, linewidth=0.0,
               hatch='....', width=0.14, zorder=3)
    ax.axhline(native_total, **native_line_kw)
    ax.set_xticks([])
    ax.set_ylim(bottom=0)
    ax.set_xlim(-0.14, 0.34)
    ax.yaxis.set_major_locator(MaxNLocator(nbins=4))

    for ax in axes + [ax_total]:
        ax.tick_params(axis='x', length=0)
        ax.tick_params(axis='y', length=2, pad=1)
        ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda y, _: '0' if abs(y) < 1e-12 else f'{y:.2g}'))

    legend_handles = [
        native_handle,
        S.backend_legend_handle('YoloFS'),
        S.backend_legend_handle('OverlayFS'),
        mpatches.Patch(facecolor='#666', edgecolor='#666', label='run'),
        mpatches.Patch(facecolor='white', edgecolor='#666', hatch='////', label='snapshot'),
        mpatches.Patch(facecolor='white', edgecolor='#666', hatch='....', label='commit'),
    ]
    fig.legend(handles=legend_handles, loc='upper center', bbox_to_anchor=(0.5, 0.905),
               ncol=6, handlelength=1.1, handletextpad=0.35, borderpad=0.15, columnspacing=0.55)
    fig.text(0.055, 0.46, 'Time (s)', rotation=90, va='center', ha='center', fontsize=8.5)
    fig.subplots_adjust(left=0.12, right=0.99, top=0.8, bottom=0.125)

    edit_bbox = axes[3].get_position()
    title_y = edit_bbox.y0 - 0.095 * edit_bbox.height
    total_bbox = ax_total.get_position()
    fig.text(total_bbox.x0 + total_bbox.width / 2, title_y, 'Total', ha='center', va='top',
             fontsize=7.3, fontweight='bold')

    fig.savefig(out_path, bbox_inches='tight', dpi=300, metadata={"CreationDate": None})
    plt.close(fig)
    print(f"Figure written to {out_path}", file=sys.stderr)

if __name__ == '__main__':
    main()
