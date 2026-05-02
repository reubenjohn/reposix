#!/usr/bin/env python3
"""Render the reposix benchmark social asset as a 1200x675 PNG using PIL.

Mirrors the hand-authored SVG at social/assets/benchmark.svg. Kept as a
committed script so the image is reproducible and the values trace back
to docs/benchmarks/token-economy.md.
"""
from __future__ import annotations

from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

OUT = Path("/home/reuben/workspace/reposix/docs/social/assets/benchmark.png")

W, H = 1200, 675
BG = "#0d1117"
WHITE = "#ffffff"
FG = "#c9d1d9"
MUTED = "#8b949e"
MCP_COLOR = "#ef6c00"
REPOSIX_COLOR = "#00897b"

FONT_DIR = Path("/usr/share/fonts/truetype/dejavu")
F_BOLD = FONT_DIR / "DejaVuSans-Bold.ttf"
F_REG = FONT_DIR / "DejaVuSans.ttf"
F_MONO = FONT_DIR / "DejaVuSansMono.ttf"


def font(path: Path, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(path), size=size)


def rounded_rect(draw: ImageDraw.ImageDraw, xy, radius: int, fill: str) -> None:
    draw.rounded_rectangle(xy, radius=radius, fill=fill)


def main() -> None:
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)

    # Headline
    d.text((60, 60), "89.1% fewer tokens", font=font(F_BOLD, 72), fill=WHITE)

    # "in simulation" caveat — warning-amber pill next to the headline.
    pill_font = font(F_BOLD, 20)
    pill_txt = "in simulation"
    pill_bbox = d.textbbox((0, 0), pill_txt, font=pill_font)
    pill_w = (pill_bbox[2] - pill_bbox[0]) + 28
    pill_h = 32
    # Place pill just right of the headline, vertically aligned with its mid.
    head_bbox = d.textbbox((60, 60), "89.1% fewer tokens", font=font(F_BOLD, 72))
    pill_x0 = head_bbox[2] + 24
    pill_y0 = 82
    d.rounded_rectangle(
        (pill_x0, pill_y0, pill_x0 + pill_w, pill_y0 + pill_h),
        radius=16,
        fill=(58, 39, 8),         # deep amber
        outline=(200, 140, 40),   # amber border
        width=2,
    )
    d.text(
        (pill_x0 + 14, pill_y0 + 5),
        pill_txt,
        font=pill_font,
        fill=(255, 196, 80),      # amber fg
    )

    # Subhead
    d.text(
        (60, 148),
        "reposix vs MCP  \u2014  same task: read 3 issues, edit 1, push",
        font=font(F_REG, 24),
        fill=MUTED,
    )

    # Bar scale: cap MCP bar at 780px so "4,068 tokens" label fits inside 1200px canvas
    scale = 780.0 / 4068.0
    bar_h = 70

    # MCP row
    d.text((60, 250), "MCP-mediated", font=font(F_BOLD, 22), fill=FG)
    d.text((60, 282), "tool catalog + schemas", font=font(F_REG, 15), fill=MUTED)
    mcp_w = int(4068 * scale)  # 900
    rounded_rect(d, (60, 305, 60 + mcp_w, 305 + bar_h), radius=4, fill=MCP_COLOR)
    # Token label at end of MCP bar
    mcp_label = "4,068 tokens"
    label_font = font(F_BOLD, 36)
    # Place 15px after bar end, vertically centred in bar
    lx = 60 + mcp_w + 15
    # Centre text vertically against bar
    bbox = d.textbbox((0, 0), mcp_label, font=label_font)
    lh = bbox[3] - bbox[1]
    ly = 305 + (bar_h - lh) // 2 - bbox[1]
    d.text((lx, ly), mcp_label, font=label_font, fill=WHITE)

    # reposix row
    d.text((60, 420), "reposix", font=font(F_BOLD, 22), fill=FG)
    d.text((60, 452), "shell session transcript", font=font(F_REG, 15), fill=MUTED)
    rep_w = max(int(315 * scale), 6)  # ~70
    rounded_rect(d, (60, 475, 60 + rep_w, 475 + bar_h), radius=4, fill=REPOSIX_COLOR)
    rep_label = "315 tokens"
    lx = 60 + rep_w + 15
    bbox = d.textbbox((0, 0), rep_label, font=label_font)
    lh = bbox[3] - bbox[1]
    ly = 475 + (bar_h - lh) // 2 - bbox[1]
    d.text((lx, ly), rep_label, font=label_font, fill=WHITE)

    # Branding bottom-left
    d.text(
        (60, 620),
        "github.com/reubenjohn/reposix",
        font=font(F_MONO, 18),
        fill=MUTED,
    )

    # Footnote bottom-right (right-aligned)
    foot = "fixture: 35-tool Jira-shaped MCP catalog.  real-world TBD."
    foot_font = font(F_REG, 18)
    fb = d.textbbox((0, 0), foot, font=foot_font)
    fw = fb[2] - fb[0]
    d.text((W - 60 - fw, 620), foot, font=foot_font, fill=MUTED)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUT, format="PNG", optimize=True)

    # Verify
    with Image.open(OUT) as verify:
        print(f"wrote {OUT} size={verify.size} mode={verify.mode}")


if __name__ == "__main__":
    main()
