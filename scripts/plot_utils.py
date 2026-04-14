"""Shared helpers and style for yolo-bench paper plot scripts."""

from pathlib import Path
import csv
import sys
import os

import matplotlib

matplotlib.use('Agg')
import matplotlib.font_manager as fm
import matplotlib.pyplot as plt
import matplotlib.patheffects as pe


def generated_dir_from_argv(argv: list[str]) -> Path:
    if len(argv) < 2:
        print(f"Usage: {argv[0]} <generated-dir>", file=sys.stderr)
        sys.exit(1)
    return Path(argv[1])


def read_csv_rows(generated_dir: Path, name: str):
    with (generated_dir / name).open() as f:
        return list(csv.DictReader(f))


def save_figure(fig, out_path: Path):
    fig.savefig(out_path, bbox_inches='tight', dpi=300, metadata={"CreationDate": None})
    plt.close(fig)
    print(f"Figure written to {out_path}", file=sys.stderr)


_libertine_dir = '/usr/share/fonts/opentype/linux-libertine'
if os.path.isdir(_libertine_dir):
    for _f in fm.findSystemFonts(fontpaths=[_libertine_dir]):
        fm.fontManager.addfont(_f)

plt.rcParams.update({
    'font.family': 'serif',
    'font.serif': ['Linux Libertine O'],
    'font.size': 12,
    'axes.titlesize': 13,
    'axes.labelsize': 12,
    'xtick.labelsize': 11,
    'ytick.labelsize': 11,
    'legend.fontsize': 11,
    'legend.frameon': False,
    'axes.spines.top': False,
    'axes.spines.right': False,
})

TABLEAU10 = {
    'blue': '#4e79a7',
    'orange': '#f28e2c',
    'red': '#e15759',
    'teal': '#76b7b2',
    'green': '#59a14f',
    'yellow': '#edc949',
    'purple': '#af7aa1',
    'pink': '#ff9da7',
    'brown': '#9c755f',
    'gray': '#bab0ab',
}

BACKEND_COLORS = {
    'YoloFS (no perm)': '#a0c8e2',
    'YoloFS': '#4e79a7',
    'OverlayFS': '#59a14f',
    'BranchFS': '#f28e2c',
}
NATIVE_COLOR = 'black'
UNCAPPABLE = {'OverlayFS'}

NATIVE_LINE_KW = dict(
    color=NATIVE_COLOR, linewidth=1.0, linestyle='-', zorder=5,
    path_effects=[pe.withStroke(linewidth=3.0, foreground='white', alpha=0.5)],
)


def fmt_lat(v):
    if v >= 1_000_000:
        return f'{v/1_000_000:.0f}s'
    if v >= 1000:
        return f'{v/1000:.0f}ms'
    return f'{v:.0f}'


def native_legend_handle(label='Base'):
    import matplotlib.lines as mlines
    return mlines.Line2D([], [], label=label, **NATIVE_LINE_KW)


def backend_legend_handle(name):
    import matplotlib.patches as mpatches
    return mpatches.Patch(facecolor=BACKEND_COLORS[name], edgecolor='white',
                          label=name)
