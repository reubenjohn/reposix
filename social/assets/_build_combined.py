"""Combine hero.png + workflow.png + benchmark.png into a single media item.

Produces two outputs:

  combined.gif   - 3-slide loop, 4.5s each. Both platforms autoplay.
                   Use this when you can only attach one item.
  combined.png   - tall stacked PNG (hero, workflow, benchmark). Use this
                   if you prefer a single static image.

Slide order tells a story:
  1. hero      - "what it looks like" (file browser)
  2. workflow  - "what the agent does" (annotated shell session)
  3. benchmark - "why it matters" (92.3% fewer tokens, in simulation)

All three inputs are already 1200x675 — no re-rendering needed.
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
F_SANS_B = F_DIR / "DejaVuSans-Bold.ttf"
DIM = (139, 148, 158)
BG = (13, 17, 23)
DIV_BORDER = (48, 54, 61)


def _sources() -> list[Image.Image]:
    imgs = [Image.open(p).convert("RGB") for p in (HERO, WORK, BENCH)]
    w, h = imgs[0].size
    for im, p in zip(imgs, (HERO, WORK, BENCH)):
        assert im.size == (w, h), f"size mismatch: {p.name}={im.size} vs {(w, h)}"
    return imgs


def build_gif() -> None:
    frames = [
        im.quantize(colors=255, method=Image.Quantize.FASTOCTREE)
        for im in _sources()
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


def _draw_divider(d: ImageDraw.ImageDraw, y0: int, y1: int, W: int, label: str) -> None:
    d.rectangle((0, y0, W, y1), fill=BG)
    d.line((40, y0 + 1, W - 40, y0 + 1), fill=DIV_BORDER, width=1)
    d.line((40, y1 - 1, W - 40, y1 - 1), fill=DIV_BORDER, width=1)
    f = ImageFont.truetype(str(F_SANS_B), 18)
    bb = d.textbbox((0, 0), label, font=f)
    lw = bb[2] - bb[0]
    th = bb[3] - bb[1]
    d.text(((W - lw) / 2, y0 + (y1 - y0 - th) / 2 - 2),
           label, font=f, fill=DIM)


def build_stacked_png() -> None:
    a, b, c = _sources()
    W = a.width
    DIV_H = 68
    H = a.height + DIV_H + b.height + DIV_H + c.height
    canvas = Image.new("RGB", (W, H), BG)
    canvas.paste(a, (0, 0))
    canvas.paste(b, (0, a.height + DIV_H))
    canvas.paste(c, (0, a.height + DIV_H + b.height + DIV_H))

    d = ImageDraw.Draw(canvas)
    y = a.height
    _draw_divider(d, y, y + DIV_H, W,
                  "above: what it looks like     \u2193     below: what the agent does")
    y += DIV_H + b.height
    _draw_divider(d, y, y + DIV_H, W,
                  "above: what the agent does     \u2193     below: why it matters (in simulation)")

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
