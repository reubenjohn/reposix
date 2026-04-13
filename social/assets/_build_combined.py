"""Combine hero.png + workflow.png + a benchmark summary into a single media item.

Produces two outputs:

  combined.gif   - 3-slide loop, 4.5s each. Both platforms autoplay.
                   Uses the full 1200x675 benchmark.png slide.
  combined.png   - tall stacked PNG (hero, workflow, compact benchmark strip).
                   Renders the benchmark as a slim summary band so the
                   page has rhythm instead of three equal slabs.

Slide order tells a story:
  1. hero      - "what it looks like" (file browser)
  2. workflow  - "what the agent does" (annotated shell session)
  3. benchmark - "why it matters" (92.3% fewer tokens, in simulation)
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ASSETS = Path("/home/reuben/workspace/reposix/social/assets")
HERO = ASSETS / "hero.png"
WORK = ASSETS / "workflow.png"
BENCH = ASSETS / "benchmark.png"
OUT_GIF = ASSETS / "combined.gif"
OUT_PNG = ASSETS / "combined.png"

HOLD_MS = 4500

F_DIR = Path("/usr/share/fonts/truetype/dejavu")
F_SANS = F_DIR / "DejaVuSans.ttf"
F_SANS_B = F_DIR / "DejaVuSans-Bold.ttf"
F_MONO = F_DIR / "DejaVuSansMono.ttf"

# Palette (matches benchmark.png + hero.png)
BG = (13, 17, 23)
FG = (201, 209, 217)
WHITE = (255, 255, 255)
DIM = (139, 148, 158)
DIV_BORDER = (48, 54, 61)
MCP_COLOR = (239, 108, 0)
REPOSIX_COLOR = (0, 137, 123)
AMBER_BG = (58, 39, 8)
AMBER_BORDER = (200, 140, 40)
AMBER_FG = (255, 196, 80)


def _ft(p: Path, s: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(p), s)


def _sources_for_gif() -> list[Image.Image]:
    """Three full-size 1200x675 slides for the GIF."""
    imgs = [Image.open(p).convert("RGB") for p in (HERO, WORK, BENCH)]
    w, h = imgs[0].size
    for im, p in zip(imgs, (HERO, WORK, BENCH)):
        assert im.size == (w, h), f"size mismatch: {p.name}={im.size} vs {(w, h)}"
    return imgs


def build_gif() -> None:
    frames = [
        im.quantize(colors=255, method=Image.Quantize.FASTOCTREE)
        for im in _sources_for_gif()
    ]

    frames[0].save(
        OUT_GIF,
        save_all=True,
        append_images=frames[1:],
        duration=[HOLD_MS] * len(frames),
        loop=0,
        optimize=True,
        disposal=2,
    )

    with Image.open(OUT_GIF) as v:
        print(
            f"GIF: {OUT_GIF} size={v.size} "
            f"n_frames={getattr(v, 'n_frames', 'n/a')} "
            f"bytes={OUT_GIF.stat().st_size}"
        )


def _draw_divider(
    d: ImageDraw.ImageDraw, y0: int, y1: int, W: int, label: str
) -> None:
    d.rectangle((0, y0, W, y1), fill=BG)
    d.line((40, y0 + 1, W - 40, y0 + 1), fill=DIV_BORDER, width=1)
    d.line((40, y1 - 1, W - 40, y1 - 1), fill=DIV_BORDER, width=1)
    f = _ft(F_SANS_B, 18)
    bb = d.textbbox((0, 0), label, font=f)
    lw = bb[2] - bb[0]
    th = bb[3] - bb[1]
    d.text(((W - lw) / 2, y0 + (y1 - y0 - th) / 2 - 2),
           label, font=f, fill=DIM)


def _draw_compact_benchmark(
    canvas: Image.Image, x: int, y: int, W: int, H: int
) -> None:
    """Render a slim, horizontal benchmark summary band for the stacked PNG.

    Layout: full-width rows stacked top-to-bottom (headline, MCP bar, reposix
    bar, footer). Avoids the side-by-side crowding of the full slide while
    preserving the same data.
    """
    d = ImageDraw.Draw(canvas)
    d.rectangle((x, y, x + W, y + H), fill=BG)

    left_x = x + 60
    right_x = x + W - 60

    # --- Row 1: headline + "in simulation" pill
    head_font = _ft(F_SANS_B, 44)
    head_txt = "92.3% fewer tokens"
    d.text((left_x, y + 24), head_txt, font=head_font, fill=WHITE)
    head_bb = d.textbbox((left_x, y + 24), head_txt, font=head_font)

    pill_font = _ft(F_SANS_B, 16)
    pill_txt = "in simulation"
    pb = d.textbbox((0, 0), pill_txt, font=pill_font)
    pill_w = (pb[2] - pb[0]) + 22
    pill_h = 28
    pill_x0 = head_bb[2] + 20
    pill_y0 = head_bb[1] + (head_bb[3] - head_bb[1] - pill_h) // 2 + 4
    d.rounded_rectangle(
        (pill_x0, pill_y0, pill_x0 + pill_w, pill_y0 + pill_h),
        radius=14,
        fill=AMBER_BG,
        outline=AMBER_BORDER,
        width=2,
    )
    d.text(
        (pill_x0 + 11, pill_y0 + 5),
        pill_txt,
        font=pill_font,
        fill=AMBER_FG,
    )

    # --- Bar rows: labels on the left (width 220), bars fill the rest
    label_col_w = 220
    bars_x0 = left_x + label_col_w
    bars_right = right_x
    # Reserve ~150px for the "4,068 tokens" label at bar end
    label_reserve = 150
    scale = (bars_right - bars_x0 - label_reserve) / 4068.0
    bar_h = 36
    row_gap = 58  # distance between the two bars

    # Row 2: MCP
    row1_y = y + 110
    d.text(
        (left_x, row1_y + 3),
        "MCP-mediated",
        font=_ft(F_SANS_B, 17),
        fill=FG,
    )
    d.text(
        (left_x, row1_y + 22),
        "tool catalog + schemas",
        font=_ft(F_SANS, 12),
        fill=DIM,
    )
    mcp_w = int(4068 * scale)
    d.rounded_rectangle(
        (bars_x0, row1_y, bars_x0 + mcp_w, row1_y + bar_h),
        radius=4,
        fill=MCP_COLOR,
    )
    d.text(
        (bars_x0 + mcp_w + 14, row1_y + 4),
        "4,068 tokens",
        font=_ft(F_SANS_B, 22),
        fill=WHITE,
    )

    # Row 3: reposix
    row2_y = row1_y + row_gap
    d.text(
        (left_x, row2_y + 3),
        "reposix",
        font=_ft(F_SANS_B, 17),
        fill=FG,
    )
    d.text(
        (left_x, row2_y + 22),
        "shell session transcript",
        font=_ft(F_SANS, 12),
        fill=DIM,
    )
    rep_w = max(int(315 * scale), 6)
    d.rounded_rectangle(
        (bars_x0, row2_y, bars_x0 + rep_w, row2_y + bar_h),
        radius=4,
        fill=REPOSIX_COLOR,
    )
    d.text(
        (bars_x0 + rep_w + 14, row2_y + 4),
        "315 tokens",
        font=_ft(F_SANS_B, 22),
        fill=WHITE,
    )

    # Footer band
    footer_y = y + H - 34
    d.text(
        (left_x, footer_y),
        "github.com/reubenjohn/reposix",
        font=_ft(F_MONO, 15),
        fill=DIM,
    )
    right_foot = "fixture: 35-tool Jira-shaped MCP catalog.  \u00b7  real-world TBD."
    rf_font = _ft(F_SANS, 15)
    rfb = d.textbbox((0, 0), right_foot, font=rf_font)
    d.text(
        (x + W - 60 - (rfb[2] - rfb[0]), footer_y),
        right_foot,
        font=rf_font,
        fill=DIM,
    )


def build_stacked_png() -> None:
    a = Image.open(HERO).convert("RGB")
    b = Image.open(WORK).convert("RGB")
    assert a.size == b.size

    W = a.width
    DIV_H = 68
    BENCH_H = 300     # slim summary band
    H = a.height + DIV_H + b.height + DIV_H + BENCH_H

    canvas = Image.new("RGB", (W, H), BG)
    canvas.paste(a, (0, 0))
    canvas.paste(b, (0, a.height + DIV_H))

    d = ImageDraw.Draw(canvas)
    y = a.height
    _draw_divider(
        d, y, y + DIV_H, W,
        "above: what it looks like     \u2193     below: what the agent does",
    )
    y += DIV_H + b.height
    _draw_divider(
        d, y, y + DIV_H, W,
        "above: what the agent does     \u2193     below: why it matters",
    )
    y += DIV_H
    _draw_compact_benchmark(canvas, 0, y, W, BENCH_H)

    canvas.save(OUT_PNG, format="PNG", optimize=True)
    with Image.open(OUT_PNG) as v:
        print(f"PNG: {OUT_PNG} size={v.size} bytes={OUT_PNG.stat().st_size}")


def main() -> None:
    for p in (HERO, WORK, BENCH):
        if not p.exists():
            raise SystemExit(f"missing {p}")
    build_gif()
    build_stacked_png()


if __name__ == "__main__":
    main()
