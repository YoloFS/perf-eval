"""Shared plot style for yolo-bench paper figures.

Provides consistent fonts, colors, sizes, and helpers across all figures.
"""
import matplotlib
matplotlib.use('Agg')
import matplotlib.font_manager as fm
import matplotlib.pyplot as plt
import matplotlib.patheffects as pe
import os

# ── Font setup: Linux Libertine to match acmart ──
_libertine_dir = '/usr/share/fonts/opentype/linux-libertine'
if os.path.isdir(_libertine_dir):
    for _f in fm.findSystemFonts(fontpaths=[_libertine_dir]):
        fm.fontManager.addfont(_f)

plt.rcParams.update({
    'font.family': 'serif',
    'font.serif': ['Linux Libertine O'],
    # Sizes compensate for ~2x shrink (14" figure -> ~6.5" \textwidth).
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

# ── Tableau-10 palette ──
TABLEAU10 = {
    'blue':   '#4e79a7',
    'orange': '#f28e2c',
    'red':    '#e15759',
    'teal':   '#76b7b2',
    'green':  '#59a14f',
    'yellow': '#edc949',
    'purple': '#af7aa1',
    'pink':   '#ff9da7',
    'brown':  '#9c755f',
    'gray':   '#bab0ab',
}

# ── Backend colors (consistent across all figures) ──
BACKEND_COLORS = {
    'YoloFS (no perm)': '#a0c8e2',   # Tableau-20 light blue
    'YoloFS':           '#4e79a7',   # Tableau blue
    'OverlayFS':      '#59a14f',   # Tableau green
    'BranchFS':       '#f28e2c',   # Tableau orange
}
NATIVE_COLOR = 'black'

# Backends whose bars are never capped or broken out.
UNCAPPABLE = {'OverlayFS'}

# ── Base baseline drawing ──
NATIVE_LINE_KW = dict(
    color=NATIVE_COLOR, linewidth=1.0, linestyle='-', zorder=5,
    path_effects=[pe.withStroke(linewidth=3.0, foreground='white', alpha=0.5)],
)

# ── Helpers ──

def fmt_lat(v):
    """Format a latency value for annotation labels."""
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
