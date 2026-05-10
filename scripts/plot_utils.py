"""Shared helpers and style for yolo-bench paper plot scripts."""

from pathlib import Path
import csv
import shutil
import subprocess
import sys
import os

import matplotlib

matplotlib.use('Agg')
import matplotlib.font_manager as fm
import matplotlib.pyplot as plt
import matplotlib.patheffects as pe


def generated_dir_from_argv(argv: list[str]) -> Path:
    if len(argv) >= 2:
        return Path(argv[1])
    # Default: ../../paper/generated/ relative to this file (umbrella layout).
    script_dir = Path(__file__).resolve().parent
    default = script_dir.parent.parent / "paper" / "generated"
    if not default.is_dir():
        print(f"Usage: {argv[0]} <generated-dir>  (default {default} not found)",
              file=sys.stderr)
        sys.exit(1)
    return default


def read_csv_rows(generated_dir: Path, name: str):
    with (generated_dir / name).open() as f:
        return list(csv.DictReader(f))


def save_figure(fig, out_path: Path):
    meta = {"CreationDate": None}
    fig.savefig(out_path, bbox_inches='tight', dpi=300, metadata=meta)
    png_path = out_path.with_suffix('.png')
    fig.savefig(png_path, bbox_inches='tight', dpi=300, metadata=meta)
    plt.close(fig)
    print(f"Figure written to {out_path} (+ {png_path.name})", file=sys.stderr)


def _find_libertine_dir():
    """Locate Linux Libertine fonts. Prefer kpsewhich (texlive); fall back to
    OS font dirs. Returns None if Libertine isn't installed anywhere."""
    if shutil.which('kpsewhich'):
        try:
            r = subprocess.run(
                ['kpsewhich', '-format=opentype fonts', 'LinLibertine_R.otf'],
                capture_output=True, text=True, timeout=2,
            )
            if r.returncode == 0 and r.stdout.strip():
                return Path(r.stdout.strip()).parent
        except Exception:
            pass
    for c in (
        '/usr/share/fonts/opentype/linux-libertine',          # Ubuntu fonts-linuxlibertine
        '/usr/share/texlive/texmf-dist/fonts/opentype/public/libertine',  # Ubuntu texlive
    ):
        if os.path.isdir(c):
            return Path(c)
    return None


_libertine_dir = _find_libertine_dir()
if _libertine_dir is not None:
    for _f in fm.findSystemFonts(fontpaths=[str(_libertine_dir)]):
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
