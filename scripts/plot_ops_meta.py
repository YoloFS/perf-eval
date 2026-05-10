#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["matplotlib>=3.7", "numpy>=1.24"]
# ///
"""Plot metadata operation latency small multiples."""
import sys

import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np
from matplotlib.ticker import MaxNLocator
from plot_utils import (
    BACKEND_COLORS,
    NATIVE_LINE_KW,
    UNCAPPABLE,
    backend_legend_handle,
    fmt_lat,
    generated_dir_from_argv,
    native_legend_handle,
    read_csv_rows,
    save_figure,
)

CAP_FACTOR = 5.0


def main():
    generated_dir = generated_dir_from_argv(sys.argv)
    out_path = generated_dir / 'ops-meta.pdf'
    rows = read_csv_rows(generated_dir, 'ops-meta.csv')

    ops = ['create', 'open', 'stat', 'readdir', 'append', 'rename', 'unlink']
    bar_backends = ['YoloFS (no perm)', 'YoloFS', 'OverlayFS', 'BranchFS']
    native_key = 'Base'
    sources = ['Base', 'Snap', 'Stage']
    source_full = {'Base': 'base', 'Snap': 'checkpoint', 'Stage': 'stage'}

    lookup = {}
    for r in rows:
        key = (r['op'].strip(), int(r['size'].strip()), r['source'].strip(), r['backend'].strip())
        lookup[key] = float(r['lat_us'].strip())

    nb = len(bar_backends)
    bar_width = 0.8 / nb

    def compute_cap(vals, floor_vals):
        sv = sorted(v for v in vals if v > 0)
        if len(sv) < 2:
            return sv[-1] * 1.35 if sv else 1.0
        reasonable = sv[:]
        while len(reasonable) > 2:
            ref_idx = max(0, int(len(reasonable) * 0.6) - 1)
            if reasonable[-1] > reasonable[ref_idx] * CAP_FACTOR:
                reasonable.pop()
            else:
                break
        cap = reasonable[-1] * 1.35
        floor_max = max((v for v in floor_vals if v > 0), default=0)
        if floor_max > 0:
            cap = max(cap, floor_max * 1.15)
        return cap

    plt.rcParams.update({'font.size': 17, 'axes.labelsize': 17, 'xtick.labelsize': 13,
                         'ytick.labelsize': 14, 'legend.fontsize': 14})

    fig = plt.figure(figsize=(14, 1.8))
    gs = gridspec.GridSpec(1, len(ops), figure=fig, wspace=0.35, hspace=0.35)

    drew_native_line = False
    for col_idx, op in enumerate(ops):
        if op == 'create':
            src_labels = ['\u2014']
            src_keys = ['stage']
        else:
            src_labels = list(sources)
            src_keys = [source_full[s] for s in sources]

        x = np.arange(len(src_labels))
        all_vals = []
        floor_vals = []
        for sk in src_keys:
            for b in [native_key] + bar_backends:
                v = lookup.get((op, 100, sk, b), 0)
                if v > 0:
                    all_vals.append(v)
                    if b in UNCAPPABLE:
                        floor_vals.append(v)
        if not all_vals:
            continue

        ax = fig.add_subplot(gs[0, col_idx])
        cap = compute_cap(all_vals, floor_vals)
        ax.set_ylim(0, cap)
        ax.spines['top'].set_visible(False)
        ax.spines['right'].set_visible(False)

        native_src = 'base' if op != 'create' else 'stage'
        nv = lookup.get((op, 100, native_src, native_key), 0)
        if nv > 0:
            ax.axhline(y=nv, **NATIVE_LINE_KW,
                       label=native_key if not drew_native_line else None)
            if not drew_native_line:
                drew_native_line = True

        annotations = []
        capped_at_src = {}
        for bi, b in enumerate(bar_backends):
            vals = [lookup.get((op, 100, sk, b), 0) for sk in src_keys]
            offset = (bi - (nb - 1) / 2) * bar_width
            display = [min(v, cap) if v > 0 else 0 for v in vals]

            ax.bar(x + offset, display, bar_width * 0.9,
                   color=BACKEND_COLORS[b],
                   edgecolor='white', linewidth=0.3,
                   label=b if col_idx == 0 else None)

            bw = bar_width * 0.9
            for i, v in enumerate(vals):
                if v > cap:
                    fade_bottom = cap * 0.65
                    fade_height = cap - fade_bottom
                    xpos = x[i] + offset
                    n_steps = 80
                    step_h = fade_height / n_steps
                    for s in range(n_steps):
                        alpha = (s / (n_steps - 1)) * 0.92
                        y_bot = fade_bottom + s * step_h
                        ax.bar(xpos, step_h, bw, bottom=y_bot,
                               color=(1, 1, 1, alpha), edgecolor='none',
                               zorder=6)
                    capped_at_src.setdefault(i, []).append((v, xpos, b))

        all_capped = []
        for entries in capped_at_src.values():
            all_capped.extend(entries)
        if all_capped:
            ranked = sorted(all_capped, key=lambda e: e[0], reverse=True)
            for rank, (v, xp, _b) in enumerate(ranked):
                yp = cap * (0.95 - rank * 0.05)
                annotations.append((xp, yp, fmt_lat(v)))

        for (xp, yp, txt) in annotations:
            ax.annotate(txt, (xp, yp), fontsize=14, ha='center', va='center',
                        color='#b00', fontweight='bold', zorder=7,
                        bbox=dict(boxstyle='round,pad=0.15', fc='white',
                                  ec='none', alpha=0.7))

        ax.set_xlabel(op, fontweight='bold', labelpad=4, fontsize=14)
        ax.yaxis.set_major_locator(MaxNLocator(nbins=4, integer=True))
        ax.set_xticks(x)
        ax.set_xticklabels(src_labels)
        ax.tick_params(axis='x', length=0, pad=2)
        if col_idx == 0:
            ax.set_ylabel('Latency (\u00b5s)')

    legend_items = [native_legend_handle(native_key)]
    for b in bar_backends:
        legend_items.append(backend_legend_handle(b))

    fig.legend(handles=legend_items, loc='upper center', ncol=nb + 1,
               bbox_to_anchor=(0.5, 1.0))
    fig.subplots_adjust(top=0.72)

    save_figure(fig, out_path)


if __name__ == '__main__':
    main()
