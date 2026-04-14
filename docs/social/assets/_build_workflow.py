"""Render a single annotated shell-session image for social posts.

Complements the file-browser hero (hero.png): the hero shows *what reposix
looks like*; this image shows *what the agent does with it*. Reads
top-to-bottom like a static asciinema — no frames, no context loss.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

OUT = Path("/home/reuben/workspace/reposix/docs/social/assets/workflow.png")
W, H = 1200, 675

# Palette (matches hero.png / benchmark.png for consistency)
BG = (13, 17, 23)
WIN_BG = (22, 27, 34)
CHROME_BG = (33, 38, 45)
BORDER = (48, 54, 61)
FG = (230, 237, 243)
DIM = (139, 148, 158)
COMMENT = (139, 148, 158)
PROMPT = (63, 185, 80)       # green $
CMD = (230, 237, 243)        # white commands
KEY = (121, 192, 255)        # cyan YAML keys
VALUE = (255, 140, 64)       # orange values
OUTPUT = (200, 209, 218)     # output lines, slightly dimmer than cmd
ARROW = (255, 196, 80)       # gold annotation arrows/labels
SUCCESS = (63, 185, 80)
API = (210, 168, 255)        # mauve for the API-call trace line
TRAFFIC_RED = (255, 95, 87)
TRAFFIC_YELLOW = (254, 188, 46)
TRAFFIC_GREEN = (40, 200, 64)

F_DIR = Path("/usr/share/fonts/truetype/dejavu")
F_MONO = F_DIR / "DejaVuSansMono.ttf"
F_MONO_B = F_DIR / "DejaVuSansMono-Bold.ttf"
F_SANS = F_DIR / "DejaVuSans.ttf"
F_SANS_B = F_DIR / "DejaVuSans-Bold.ttf"


def ft(p: Path, s: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(p), s)


# ---- geometry --------------------------------------------------------------

WIN_X0, WIN_Y0 = 40, 40
WIN_X1, WIN_Y1 = W - 40, H - 80
CHROME_H = 38

BODY_X0 = WIN_X0 + 36
BODY_Y0 = WIN_Y0 + CHROME_H + 20
LINE_H = 24
# Split: terminal lines occupy ~720px wide, annotations on the right
TERM_RIGHT = WIN_X0 + 780
ANNOT_X = 830


# ---- types -----------------------------------------------------------------

def comment(text: str, note: str | None = None):
    return {"kind": "comment", "text": text, "note": note}


def blank(note: str | None = None):
    return {"kind": "blank", "note": note}


def prompt(cmd: str, note: str | None = None):
    return {"kind": "prompt", "cmd": cmd, "note": note}


def out(text: str, color=OUTPUT, note: str | None = None):
    return {"kind": "out", "text": text, "color": color, "note": note}


def yaml(key: str, value: str | None = None, note: str | None = None):
    return {"kind": "yaml", "key": key, "value": value, "note": note}


def api(text: str, color=API, note: str | None = None):
    return {"kind": "api", "text": text, "color": color, "note": note}


# ---- content ---------------------------------------------------------------

# Keep this tight. 18 lines total. Notes are the inline callouts.
SCRIPT = [
    comment("# Task: triage the open database bugs."),
    comment("# No MCP, no tool schemas — just shell and git."),
    blank(),
    prompt("cd ~/mnt/reposix-github-issues/issues"),
    blank(),
    prompt("grep -l 'status: open' *.md", note="find the open ones"),
    out("00000000001.md  00000000002.md  00000000005.md"),
    blank(),
    prompt("head -4 00000000001.md", note="it's just Markdown"),
    yaml("id: ", "1"),
    yaml("title: ", "database connection drops under load"),
    yaml("status: ", "open"),
    blank(),
    prompt("sed -i 's/open/in_progress/' 00000000001.md", note="edit in place"),
    prompt("git commit -am 'triage: #1'"),
    prompt("git push reposix main", note="becomes a real API call"),
    api("→ POST /issues/1  { \"status\": \"in_progress\" }"),
    api("✓ 200 OK     audit_event #47 logged", color=SUCCESS),
]


# ---- drawing ---------------------------------------------------------------

def draw_chrome(d: ImageDraw.ImageDraw) -> None:
    d.rectangle((WIN_X0, WIN_Y0, WIN_X1, WIN_Y0 + CHROME_H), fill=CHROME_BG)
    d.rectangle((WIN_X0, WIN_Y0, WIN_X1, WIN_Y1), outline=BORDER, width=1)
    d.line(
        (WIN_X0, WIN_Y0 + CHROME_H, WIN_X1, WIN_Y0 + CHROME_H),
        fill=BORDER,
        width=1,
    )
    d.rectangle(
        (WIN_X0 + 1, WIN_Y0 + CHROME_H + 1, WIN_X1 - 1, WIN_Y1 - 1),
        fill=WIN_BG,
    )
    cy = WIN_Y0 + CHROME_H // 2
    for i, c in enumerate((TRAFFIC_RED, TRAFFIC_YELLOW, TRAFFIC_GREEN)):
        cx = WIN_X0 + 18 + i * 22
        d.ellipse((cx - 7, cy - 7, cx + 7, cy + 7), fill=c)
    title = "agent@reposix — triaging open issues"
    font = ft(F_SANS, 14)
    bb = d.textbbox((0, 0), title, font=font)
    tw = bb[2] - bb[0]
    d.text(
        ((WIN_X0 + WIN_X1) / 2 - tw / 2, WIN_Y0 + CHROME_H / 2 - 9),
        title,
        font=font,
        fill=DIM,
    )


def draw_note(
    d: ImageDraw.ImageDraw, y: int, note: str, mono_offset_x: int
) -> None:
    """Draw an inline callout to the right of the terminal line.

    Arrow starts just right of the terminal content, ends at the annotation.
    """
    arrow_x0 = mono_offset_x + 10
    arrow_x1 = ANNOT_X - 8
    # small gap arrow: "── label"
    d.line(
        (arrow_x0, y + LINE_H // 2 - 1, arrow_x1, y + LINE_H // 2 - 1),
        fill=ARROW,
        width=1,
    )
    # tiny arrowhead at start of line (pointing LEFT toward cmd)
    d.polygon(
        [
            (arrow_x0, y + LINE_H // 2 - 1),
            (arrow_x0 + 6, y + LINE_H // 2 - 5),
            (arrow_x0 + 6, y + LINE_H // 2 + 3),
        ],
        fill=ARROW,
    )
    d.text(
        (ANNOT_X, y + 2),
        note,
        font=ft(F_SANS_B, 14),
        fill=ARROW,
    )


def draw_body(d: ImageDraw.ImageDraw) -> None:
    y = BODY_Y0
    mono = ft(F_MONO, 15)
    mono_b = ft(F_MONO_B, 15)

    for line in SCRIPT:
        kind = line["kind"]
        if kind == "blank":
            y += LINE_H
            continue

        if kind == "comment":
            d.text((BODY_X0, y), line["text"], font=mono, fill=COMMENT)
            cursor_x = BODY_X0 + d.textlength(line["text"], font=mono)
        elif kind == "prompt":
            d.text((BODY_X0, y), "$", font=mono_b, fill=PROMPT)
            d.text((BODY_X0 + 14, y), " " + line["cmd"], font=mono, fill=CMD)
            cursor_x = BODY_X0 + 14 + d.textlength(" " + line["cmd"], font=mono)
        elif kind == "out":
            d.text((BODY_X0, y), line["text"], font=mono, fill=line["color"])
            cursor_x = BODY_X0 + d.textlength(line["text"], font=mono)
        elif kind == "yaml":
            key = line["key"]
            val = line["value"] or ""
            d.text((BODY_X0, y), key, font=mono, fill=KEY)
            kw = d.textlength(key, font=mono)
            d.text((BODY_X0 + kw, y), val, font=mono, fill=VALUE if key.strip() == "status:" else FG)
            cursor_x = BODY_X0 + kw + d.textlength(val, font=mono)
        elif kind == "api":
            d.text((BODY_X0, y), line["text"], font=mono_b, fill=line["color"])
            cursor_x = BODY_X0 + d.textlength(line["text"], font=mono_b)
        else:
            cursor_x = BODY_X0

        if line.get("note"):
            draw_note(d, y, line["note"], int(cursor_x))

        y += LINE_H


def draw_caption(d: ImageDraw.ImageDraw) -> None:
    cy = WIN_Y1 + 16
    head = "The agent's entire SDK: grep, sed, git."
    sub = "reposix — REST APIs as POSIX filesystems. Complements GitHub Issues (and Jira, Confluence, …)."
    f_head = ft(F_SANS_B, 20)
    f_sub = ft(F_SANS, 13)
    bb = d.textbbox((0, 0), head, font=f_head)
    hw = bb[2] - bb[0]
    d.text(((W - hw) / 2, cy), head, font=f_head, fill=FG)
    bb2 = d.textbbox((0, 0), sub, font=f_sub)
    sw = bb2[2] - bb2[0]
    d.text(((W - sw) / 2, cy + 28), sub, font=f_sub, fill=DIM)


def main() -> None:
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    draw_chrome(d)
    draw_body(d)
    draw_caption(d)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUT, format="PNG", optimize=True)
    with Image.open(OUT) as v:
        print(f"wrote {OUT} size={v.size} bytes={OUT.stat().st_size}")


if __name__ == "__main__":
    main()
