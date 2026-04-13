"""Combine hero.png + workflow.png into a single media item.

Produces two outputs:

  combined.gif   - 2-slide loop, 4.5s each. Both platforms autoplay.
                   Use this when you can only attach one item.
  combined.png   - tall stacked PNG (hero over workflow). Use this if
                   you prefer a single static image.

Both reuse the existing 1200x675 slides — no re-rendering needed.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ASSETS = Path("/home/reuben/workspace/reposix/social/assets")
HERO = ASSETS / "hero.png"
WORK = ASSETS / "workflow.png"
OUT_GIF = ASSETS / "combined.gif"
OUT_PNG = ASSETS / "combined.png"

# LinkedIn/Twitter dwell: ~4.5s per slide. GIFs loop, so ~9s round-trip.
HOLD_MS = 4500

F_DIR = Path("/usr/share/fonts/truetype/dejavu")
F_SANS = F_DIR / "DejaVuSans.ttf"
F_SANS_B = F_DIR / "DejaVuSans-Bold.ttf"
DIM = (139, 148, 158)
BG = (13, 17, 23)


def build_gif() -> None:
    a = Image.open(HERO).convert("RGB")
    b = Image.open(WORK).convert("RGB")
    assert a.size == b.size, f"size mismatch: {a.size} vs {b.size}"

    # Quantize each frame with its own adaptive palette so the blues, oranges,
    # and YAML cyan don't get crushed by a shared 256-colour palette.
    frames = [f.quantize(colors=255, method=Image.Quantize.FASTOCTREE)
              for f in (a, b)]

    frames[0].save(
        OUT_GIF,
        save_all=True,
        append_images=frames[1:],
        duration=[HOLD_MS, HOLD_MS],
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


def build_stacked_png() -> None:
    """Hero on top, workflow below. Preserve each slide's full height
    (675 each). Add a slim divider band with label text."""
    a = Image.open(HERO).convert("RGB")
    b = Image.open(WORK).convert("RGB")
    W = a.width
    DIV_H = 68
    H = a.height + DIV_H + b.height
    canvas = Image.new("RGB", (W, H), BG)
    canvas.paste(a, (0, 0))
    canvas.paste(b, (0, a.height + DIV_H))

    # Divider band
    d = ImageDraw.Draw(canvas)
    band_y0 = a.height
    band_y1 = a.height + DIV_H
    d.rectangle((0, band_y0, W, band_y1), fill=BG)
    # Hairline dividers
    d.line((40, band_y0 + 1, W - 40, band_y0 + 1), fill=(48, 54, 61), width=1)
    d.line((40, band_y1 - 1, W - 40, band_y1 - 1), fill=(48, 54, 61), width=1)

    # Arrow between the two panels: "above: what it looks like — below: what the agent does"
    label = "above: what it looks like     \u2193     below: what the agent does"
    f = ImageFont.truetype(str(F_SANS_B), 18)
    bb = d.textbbox((0, 0), label, font=f)
    lw = bb[2] - bb[0]
    d.text(((W - lw) / 2, band_y0 + (DIV_H - (bb[3] - bb[1])) / 2 - 2),
           label, font=f, fill=DIM)

    canvas.save(OUT_PNG, format="PNG", optimize=True)
    with Image.open(OUT_PNG) as v:
        print(f"PNG: {OUT_PNG} size={v.size} bytes={OUT_PNG.stat().st_size}")


def main() -> None:
    if not HERO.exists():
        raise SystemExit(f"missing {HERO}")
    if not WORK.exists():
        raise SystemExit(f"missing {WORK}")
    build_gif()
    build_stacked_png()


if __name__ == "__main__":
    main()
