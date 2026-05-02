"""Render the file-browser-style hero image for reposix social posts.

A dark, macOS-ish file browser window: sidebar on the left showing the
Phase-13 FUSE layout — `reposix-github-issues/issues/*.md` — and a
preview pane on the right showing the markdown contents of the selected
issue. Caption at the bottom.

The canonical mount path under the current layout is
`/tmp/reposix-mnt/issues/<11-digit>.md` (e.g.
`/tmp/reposix-mnt/issues/00000000001.md`). The hero image uses the
short 4-digit form in the sidebar visual because the fixed ~310 px
sidebar width cannot fit 11-digit filenames legibly at the current
font size; the sidebar's `issues/` folder structure remains the true
visual anchor.

The point, for semi-technical viewers: "it's a folder. that's it."
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

# ---- config ----------------------------------------------------------------

OUT = Path("/home/reuben/workspace/reposix/docs/social/assets/hero.png")
W, H = 1200, 675

# Palette (GitHub dark + Finder-ish accents)
BG = (13, 17, 23)           # #0d1117 outside frame
WIN_BG = (22, 27, 34)       # #161b22 window body
SIDE_BG = (15, 19, 25)
HEADER_BG = (33, 38, 45)
BORDER = (48, 54, 61)       # #30363d
SELECTED = (31, 111, 235)   # blue selection bar
SELECTED_TXT = (255, 255, 255)
FG = (230, 237, 243)
DIM = (139, 148, 158)
FOLDER = (227, 179, 65)
FILE_ICON = (139, 148, 158)
CYAN = (121, 192, 255)      # YAML keys
GREEN = (63, 185, 80)
ORANGE = (255, 140, 64)
TRAFFIC_RED = (255, 95, 87)
TRAFFIC_YELLOW = (254, 188, 46)
TRAFFIC_GREEN = (40, 200, 64)

FONT_DIR = Path("/usr/share/fonts/truetype/dejavu")
F_SANS = FONT_DIR / "DejaVuSans.ttf"
F_SANS_B = FONT_DIR / "DejaVuSans-Bold.ttf"
F_MONO = FONT_DIR / "DejaVuSansMono.ttf"
F_MONO_B = FONT_DIR / "DejaVuSansMono-Bold.ttf"


def ft(path: Path, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(path), size=size)


# ---- geometry --------------------------------------------------------------

# Outer frame (window).
WIN_X0, WIN_Y0 = 40, 40
WIN_X1, WIN_Y1 = W - 40, H - 90  # leave room for caption band
CHROME_H = 38
SIDE_W = 310   # width of left sidebar
CONTENT_X0 = WIN_X0 + SIDE_W


def draw_chrome(d: ImageDraw.ImageDraw) -> None:
    # Title bar
    d.rectangle((WIN_X0, WIN_Y0, WIN_X1, WIN_Y0 + CHROME_H), fill=HEADER_BG)
    # Border around window
    d.rectangle((WIN_X0, WIN_Y0, WIN_X1, WIN_Y1), outline=BORDER, width=1)
    # Separator under chrome
    d.line(
        (WIN_X0, WIN_Y0 + CHROME_H, WIN_X1, WIN_Y0 + CHROME_H),
        fill=BORDER,
        width=1,
    )
    # Traffic lights
    cy = WIN_Y0 + CHROME_H // 2
    for i, c in enumerate((TRAFFIC_RED, TRAFFIC_YELLOW, TRAFFIC_GREEN)):
        cx = WIN_X0 + 18 + i * 22
        d.ellipse((cx - 7, cy - 7, cx + 7, cy + 7), fill=c)
    # Title text, centred
    title = "reposix-github-issues  —  Files"
    font = ft(F_SANS, 14)
    bb = d.textbbox((0, 0), title, font=font)
    tw = bb[2] - bb[0]
    d.text(
        ((WIN_X0 + WIN_X1) / 2 - tw / 2, WIN_Y0 + CHROME_H / 2 - 9),
        title,
        font=font,
        fill=DIM,
    )


def draw_sidebar(d: ImageDraw.ImageDraw) -> None:
    x0, y0 = WIN_X0 + 1, WIN_Y0 + CHROME_H + 1
    x1, y1 = CONTENT_X0 - 1, WIN_Y1 - 1
    d.rectangle((x0, y0, x1, y1), fill=SIDE_BG)
    # Vertical divider
    d.line((x1, y0, x1, y1), fill=BORDER, width=1)

    # Section label
    d.text(
        (x0 + 20, y0 + 18),
        "FAVOURITES",
        font=ft(F_SANS_B, 11),
        fill=DIM,
    )

    # Tree rows. Each row is (indent_level, kind, name, expanded)
    # kind: "folder" or "file".
    # indent_level: 0 = root, 1 = first child, 2 = grandchild.
    # expanded (folders only): ▾ if True, ▸ if False.
    rows = [
        (0, "folder", "reposix-github-issues", True),
        (1, "folder", "issues", True),
        (2, "file", "0001.md", None),     # selected
        (2, "file", "0002.md", None),
        (2, "file", "0003.md", None),
        (2, "file", "0004.md", None),
        (2, "file", "0005.md", None),
        (2, "file", "0006.md", None),
        (1, "folder", "labels", False),
        (1, "folder", "milestones", False),
    ]

    row_h = 30
    first_row_y = y0 + 48
    selected_idx = 2  # highlight first file (0001.md)

    for i, (indent, kind, name, expanded) in enumerate(rows):
        ry = first_row_y + i * row_h
        is_selected = (i == selected_idx)
        if is_selected:
            d.rectangle((x0 + 8, ry - 3, x1 - 6, ry + 23), fill=SELECTED)
            txt_col = SELECTED_TXT
            icon_col = SELECTED_TXT
        else:
            txt_col = FG
            icon_col = FILE_ICON

        # Indent padding: 18px per level
        ix_caret = x0 + 16 + indent * 18
        ix_icon = ix_caret + 16
        ix_label = ix_icon + 26

        # Caret for folders
        if kind == "folder":
            caret = "▾" if expanded else "▸"
            d.text(
                (ix_caret, ry + 1),
                caret,
                font=ft(F_SANS_B, 12),
                fill=icon_col if is_selected else DIM,
            )
            # Folder icon
            d.rectangle(
                (ix_icon, ry + 6, ix_icon + 16, ry + 18),
                fill=FOLDER,
            )
            d.rectangle(
                (ix_icon, ry + 3, ix_icon + 7, ry + 7),
                fill=FOLDER,
            )
            font = ft(F_SANS_B, 14)
        else:
            # File: dog-eared document icon
            iy = ry + 4
            d.polygon(
                [
                    (ix_icon, iy),
                    (ix_icon + 10, iy),
                    (ix_icon + 14, iy + 4),
                    (ix_icon + 14, iy + 16),
                    (ix_icon, iy + 16),
                ],
                outline=icon_col,
                fill=None,
                width=1,
            )
            d.line(
                (ix_icon + 10, iy, ix_icon + 10, iy + 4, ix_icon + 14, iy + 4),
                fill=icon_col,
                width=1,
            )
            font = ft(F_MONO, 13)

        d.text((ix_label, ry + 3), name, font=font, fill=txt_col)


def draw_preview(d: ImageDraw.ImageDraw) -> None:
    x0, y0 = CONTENT_X0, WIN_Y0 + CHROME_H + 1
    x1, y1 = WIN_X1 - 1, WIN_Y1 - 1
    d.rectangle((x0 + 1, y0, x1, y1), fill=WIN_BG)

    # Preview toolbar
    tb_h = 36
    d.rectangle((x0 + 1, y0, x1, y0 + tb_h), fill=HEADER_BG)
    d.line((x0 + 1, y0 + tb_h, x1, y0 + tb_h), fill=BORDER, width=1)
    d.text(
        (x0 + 18, y0 + 10),
        "0001.md",
        font=ft(F_SANS_B, 14),
        fill=FG,
    )
    d.text(
        (x0 + 110, y0 + 11),
        "— ~/mnt/reposix-github-issues/issues/",
        font=ft(F_SANS, 13),
        fill=DIM,
    )
    # Little "synced" pill on the right
    pill_r = 13
    pill_x = x1 - 180
    pill_y = y0 + 8
    d.rounded_rectangle(
        (pill_x, pill_y, pill_x + 165, pill_y + 20),
        radius=pill_r - 3,
        fill=(22, 68, 36),
    )
    # Small dot
    d.ellipse(
        (pill_x + 10, pill_y + 7, pill_x + 16, pill_y + 13),
        fill=GREEN,
    )
    d.text(
        (pill_x + 22, pill_y + 3),
        "synced to API",
        font=ft(F_SANS_B, 12),
        fill=(163, 224, 179),
    )

    # Content area — render the markdown with frontmatter.
    cx = x0 + 28
    cy = y0 + tb_h + 28
    mono = ft(F_MONO, 15)
    mono_b = ft(F_MONO_B, 15)
    sans_b = ft(F_SANS_B, 22)

    # Frontmatter block
    fm_lines = [
        ("---", DIM, mono),
        ("id: ", DIM, mono, "1", ORANGE),
        ("title: ", CYAN, mono, "database connection drops under load", FG),
        ("status: ", CYAN, mono, "open", ORANGE),
        ("labels:", CYAN, mono, "", None),
        ("  - bug", FG, mono, "", None),
        ("  - p1", FG, mono, "", None),
        ("assignee: ", CYAN, mono, "priya", FG),
        ("created_at: ", CYAN, mono, "2026-04-13T00:00:00Z", FG),
        ("---", DIM, mono),
    ]
    line_h = 24
    for i, row in enumerate(fm_lines):
        y = cy + i * line_h
        key = row[0]
        key_col = row[1]
        key_font = row[2]
        d.text((cx, y), key, font=key_font, fill=key_col)
        if len(row) >= 5 and row[3]:
            # Value after key
            bb = d.textbbox((0, 0), key, font=key_font)
            kw = bb[2] - bb[0]
            d.text((cx + kw, y), row[3], font=key_font, fill=row[4])

    # Body heading
    body_y = cy + len(fm_lines) * line_h + 16
    d.text((cx, body_y), "Database connection drops under load",
           font=sans_b, fill=FG)
    # Underline for the heading
    bb = d.textbbox((cx, body_y), "Database connection drops under load", font=sans_b)
    d.line(
        (cx, bb[3] + 4, bb[2], bb[3] + 4),
        fill=BORDER,
        width=1,
    )

    # Body paragraph
    para = (
        "Pool exhausts within 30s of spike; replica logs\n"
        "show `acquire() timeout`."
    )
    font_body = ft(F_SANS, 15)
    for i, ln in enumerate(para.split("\n")):
        d.text((cx, bb[3] + 22 + i * 22), ln, font=font_body, fill=FG)

    # "It's a folder. That's it." callout arrow/box bottom-right of preview
    callout_x = x1 - 330
    callout_y = y1 - 120
    # Dashed-ish box
    d.rounded_rectangle(
        (callout_x, callout_y, x1 - 28, callout_y + 86),
        radius=10,
        outline=SELECTED,
        width=2,
    )
    d.text(
        (callout_x + 16, callout_y + 12),
        "It's just a file.",
        font=ft(F_SANS_B, 20),
        fill=(121, 192, 255),
    )
    d.text(
        (callout_x + 16, callout_y + 44),
        "Your agent can cat, grep, sed,",
        font=ft(F_SANS, 14),
        fill=DIM,
    )
    d.text(
        (callout_x + 16, callout_y + 62),
        "and git-push it. No tool schemas.",
        font=ft(F_SANS, 14),
        fill=DIM,
    )


def draw_caption(d: ImageDraw.ImageDraw) -> None:
    # Caption band below window
    cy = WIN_Y1 + 18
    headline = "GitHub Issues, mounted as a folder."
    sub = "reposix — REST APIs as POSIX filesystems for autonomous agents"

    f_head = ft(F_SANS_B, 22)
    f_sub = ft(F_SANS, 14)

    bb = d.textbbox((0, 0), headline, font=f_head)
    hw = bb[2] - bb[0]
    d.text(
        ((W - hw) / 2, cy),
        headline,
        font=f_head,
        fill=FG,
    )
    bb2 = d.textbbox((0, 0), sub, font=f_sub)
    sw = bb2[2] - bb2[0]
    d.text(
        ((W - sw) / 2, cy + 30),
        sub,
        font=f_sub,
        fill=DIM,
    )


def main() -> None:
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    draw_chrome(d)
    draw_sidebar(d)
    draw_preview(d)
    draw_caption(d)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUT, format="PNG", optimize=True)
    with Image.open(OUT) as verify:
        print(f"wrote {OUT} size={verify.size} bytes={OUT.stat().st_size}")


if __name__ == "__main__":
    main()
