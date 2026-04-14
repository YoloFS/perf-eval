#!/usr/bin/env python3
"""Plot checkpoint depth scaling (create/read/commit latency)."""
import sys

import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator, FuncFormatter
from utils import (
    BACKEND_COLORS,
    TABLEAU10,
    generated_dir_from_argv,
    read_csv_rows,
    save_figure,
)


def main():
    generated_dir = generated_dir_from_argv(sys.argv)
    out_path = generated_dir / 'checkpoint-scaling.pdf'

    create_rows = read_csv_rows(generated_dir, 'checkpoint-scaling-create.csv')
    read_rows = read_csv_rows(generated_dir, 'checkpoint-scaling-read.csv')
    commit_rows = read_csv_rows(generated_dir, 'checkpoint-scaling-commit.csv')

    plt.rcParams.update({'font.size': 8.5, 'axes.labelsize': 8.5, 'xtick.labelsize': 8.5,
                         'ytick.labelsize': 8.5, 'legend.fontsize': 7.5})

    order = ['YoloFS', 'OverlayFS', 'BranchFS']
    colors = [BACKEND_COLORS.get(n, TABLEAU10['gray']) for n in order]
    overlay_name = 'OverlayFS'
    overlay_depths = (
        [int(r['depth']) for r in create_rows if r['backend'] == overlay_name]
        + [int(r['depth']) for r in read_rows if r['backend'] == overlay_name]
        + [int(r['depth']) for r in commit_rows if r['backend'] == overlay_name]
    )
    overlay_failure_depth = max(overlay_depths) if overlay_depths else None

    fig = plt.figure(figsize=(3.33, 1.3))
    pw = 0.24
    h = 0.54
    bot = 0.22
    left1 = 0.12
    gap = 0.045
    left2 = left1 + pw + gap
    left3 = left2 + pw + gap + 0.04
    ax_create = fig.add_axes([left1, bot, pw, h])
    ax_read = fig.add_axes([left2, bot, pw, h])
    ax_commit = fig.add_axes([left3, bot, pw, h])

    def plot_panel(ax, rows, show_ylabel, ylabel=None, exclude_backends=None,
                   mark_overlay_failure=False):
        for i, name in enumerate(order):
            if exclude_backends and name in exclude_backends:
                continue
            pts = [(int(r['depth']), float(r['mean_us'])) for r in rows if r['backend'] == name]
            if not pts:
                continue
            pts.sort()
            xs = [p[0] for p in pts]
            ys = [p[1] for p in pts]
            ax.plot(xs, ys, marker='o', markersize=2.5, linewidth=1.2,
                    color=colors[i], label=name)
            if mark_overlay_failure and name == overlay_name and overlay_failure_depth is not None and xs and xs[-1] == overlay_failure_depth:
                ax.plot([xs[-1] + 4], [ys[-1]], marker='x', markersize=4.0,
                        markeredgewidth=0.9, linestyle='None', color=colors[i],
                        clip_on=False, zorder=6)
        ax.set_ylim(bottom=0)
        if show_ylabel:
            ax.set_ylabel(ylabel or 'Latency (\u00b5s/op)')
        else:
            ax.tick_params(axis='y', labelleft=False)
        if overlay_failure_depth is not None:
            ax.set_xlim(right=max(ax.get_xlim()[1], overlay_failure_depth + 8))
        ax.yaxis.set_major_locator(MaxNLocator(nbins=4, min_n_ticks=3,
                                               steps=[1, 2, 2.5, 5, 10]))
        ax.tick_params(axis='x', length=2, pad=1.5)
        ax.tick_params(axis='y', length=2, pad=1.5)

    plot_panel(ax_create, create_rows, True, mark_overlay_failure=True)
    plot_panel(ax_read, read_rows, False, mark_overlay_failure=True)
    plot_panel(ax_commit, commit_rows, True, ylabel='ms',
               exclude_backends={'BranchFS'}, mark_overlay_failure=True)

    for ax, name in [(ax_create, 'create'), (ax_read, 'read'), (ax_commit, 'commit')]:
        ax.set_title(name, fontweight='bold', fontsize=8.5, pad=2)

    commit_non_branch = [float(r['mean_us']) for r in commit_rows if r['backend'] != 'BranchFS']
    if commit_non_branch:
        ax_commit.set_ylim(top=max(commit_non_branch) * 1.3)
    ax_commit.yaxis.set_major_locator(MaxNLocator(nbins=4, min_n_ticks=3,
                                                  steps=[1, 2, 2.5, 5, 10]))
    ax_commit.yaxis.set_major_formatter(FuncFormatter(lambda y, _: f'{y/1000:.0f}'))

    inset = ax_commit.inset_axes([0.58, 0.05, 0.28, 0.35])
    for i, name in enumerate(order):
        pts = [(int(r['depth']), float(r['mean_us'])) for r in commit_rows if r['backend'] == name]
        if not pts:
            continue
        pts.sort()
        inset.plot([p[0] for p in pts], [p[1] for p in pts],
                   marker='o', markersize=1.2, linewidth=0.8, color=colors[i])
        if name == overlay_name and overlay_failure_depth is not None and pts and pts[-1][0] == overlay_failure_depth:
            inset.plot([pts[-1][0] + 4], [pts[-1][1]], marker='x', markersize=2.4,
                       markeredgewidth=0.7, linestyle='None', color=colors[i],
                       clip_on=False, zorder=6)
    inset.set_ylim(bottom=0, top=11e6)
    inset.set_xticks([])
    inset.set_yticks([])
    inset.xaxis.set_minor_locator(plt.NullLocator())
    inset.yaxis.set_minor_locator(plt.NullLocator())
    inset.text(-0.05, 0.95, '10s', transform=inset.transAxes, fontsize=8.5,
               ha='right', va='top')
    inset.text(1.05, 0.02, '100', transform=inset.transAxes, fontsize=8.5,
               ha='left', va='bottom')
    inset.tick_params(labelsize=7, pad=0.5, length=1.5)
    inset.tick_params(axis='y', which='major', pad=0.5)
    for label in inset.yaxis.get_ticklabels():
        label.set_clip_on(False)
    inset.patch.set_facecolor('white')
    inset.patch.set_edgecolor('gray')
    for sp in inset.spines.values():
        sp.set_linewidth(0.4)
        sp.set_color('gray')

    handles, labels = ax_create.get_legend_handles_labels()
    fig.legend(handles=handles, labels=labels, loc='upper center',
               bbox_to_anchor=(0.5, 0.97), ncol=len(order),
               handlelength=1.5, handletextpad=0.4,
               borderpad=0.15, columnspacing=0.8)
    fig.text(0.5, 0.02, 'Number of snapshots', ha='center', fontsize=8.5)

    save_figure(fig, out_path)


if __name__ == '__main__':
    main()
