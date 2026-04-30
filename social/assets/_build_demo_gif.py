"""Build the looping demo GIF for reposix social posts.

Reads from the committed transcripts (no live recording) and renders 8
hand-curated frames as a 1200x675 looping GIF, plus a PNG sanity-check of
frame 0.
"""

from __future__ import annotations

import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

# ---- config ----------------------------------------------------------------

OUT_DIR = Path("/home/reuben/workspace/reposix/docs/social/assets")
GIF_PATH = OUT_DIR / "demo.gif"
FRAME0_PNG = OUT_DIR / "demo-frame0.png"

W, H = 1200, 675
BG = (13, 17, 23)            # #0d1117 gh-dark
CHROME = (22, 27, 34)        # title bar
CHROME_BORDER = (48, 54, 61)
FG = (230, 237, 243)         # primary text
DIM = (139, 148, 158)        # #8b949e
PROMPT_GREEN = (63, 185, 80) # gh green
CYAN = (121, 192, 255)
YELLOW = (210, 153, 34)
RED = (248, 81, 73)
ORANGE = (255, 166, 87)
WATERMARK = (88, 96, 105)

FONT_PATH = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
FONT_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf"
FONT_SANS = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
FONT_SANS_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"

FONT_SIZE = 20
LINE_HEIGHT = 28
PAD_X = 44
PAD_Y_TOP = 72     # after chrome bar
CHROME_H = 44


# ---- frames (hand-curated) -------------------------------------------------
#
# Each frame is a list of line dicts: {"text": str, "color": tuple | str}.
# Special colors: "prompt" means render "$" green + rest white, parsing $ on col 0.
# "cmd" is a full white command line (no leading $).

def P(cmd: str) -> dict:
    """Prompt line: green $ + white command."""
    return {"kind": "prompt", "cmd": cmd}


def O(text: str, color=DIM) -> dict:
    return {"kind": "line", "text": text, "color": color}


def B(text: str, color=FG) -> dict:
    return {"kind": "line", "text": text, "color": color, "bold": True}


TITLE = "agent@dark-factory : /tmp/reposix-mnt"

FRAMES: list[dict] = [
    # -- Frame 1: the pitch --
    {
        "title": TITLE,
        "caption": "an issue tracker, mounted as a directory",
        "lines": [
            O("# reposix: REST issue trackers as a POSIX filesystem.", CYAN),
            O("# agents use cat / sed / git — not MCP tool schemas.", DIM),
            O(""),
            P("mount | grep reposix"),
            O("reposix on /tmp/reposix-mnt type fuse.reposix", FG),
            O("        (rw,nosuid,nodev,user_id=1000)", DIM),
        ],
        "duration": 2200,
    },
    # -- Frame 2: ls --
    {
        "title": TITLE,
        "caption": "ls — issues show up as markdown files under issues/",
        "lines": [
            P("ls /tmp/reposix-mnt/issues/"),
            O("00000000001.md  00000000002.md  00000000003.md", FG),
            O("00000000004.md  00000000005.md  00000000006.md", FG),
            O(""),
            P("file /tmp/reposix-mnt/issues/00000000001.md"),
            O("00000000001.md: ASCII text (YAML frontmatter)", DIM),
        ],
        "duration": 1800,
    },
    # -- Frame 3: cat / head --
    {
        "title": TITLE,
        "caption": "cat — frontmatter + markdown body",
        "lines": [
            P("head -8 /tmp/reposix-mnt/issues/00000000001.md"),
            O("---", DIM),
            {"kind": "kv", "key": "id", "val": "1"},
            {"kind": "kv", "key": "title", "val": "database connection drops under load"},
            {"kind": "kv", "key": "status", "val": "open", "val_color": YELLOW},
            {"kind": "kv", "key": "labels", "val": ""},
            O("- bug", FG),
            O("- p1", FG),
        ],
        "duration": 2400,
    },
    # -- Frame 4: sed edit --
    {
        "title": TITLE,
        "caption": "sed — in-place edit, no API client needed",
        "lines": [
            P("sed -i 's/status: open/status: in_progress/' 00000000001.md"),
            O(""),
            P("grep ^status: 00000000001.md"),
            {"kind": "kv", "key": "status", "val": "in_progress", "val_color": ORANGE},
            O(""),
            O("# FUSE already pushed the PATCH. server version bumped.", DIM),
        ],
        "duration": 2200,
    },
    # -- Frame 5: git commit + push --
    {
        "title": TITLE,
        "caption": "git push — reposix:: remote turns commits into REST",
        "lines": [
            P("git commit -am 'triage: move #1 to in_progress'"),
            O("[main 4c2a91e] triage: move #1 to in_progress", FG),
            O(" 1 file changed, 1 insertion(+), 1 deletion(-)", DIM),
            P("git push reposix main"),
            O("To reposix::http://127.0.0.1:7801/projects/demo", DIM),
            O(" * [new branch]      main -> main", PROMPT_GREEN),
        ],
        "duration": 2400,
    },
    # -- Frame 6: server confirms --
    {
        "title": TITLE,
        "caption": "the live API agrees — this is the real backend",
        "lines": [
            P("curl -s 127.0.0.1:7801/issues/1 | jq '{id,status,version}'"),
            O("{", FG),
            {"kind": "json", "key": "id", "val": "1", "val_color": CYAN},
            {"kind": "json", "key": "status", "val": '"in_review"', "val_color": PROMPT_GREEN},
            {"kind": "json", "key": "version", "val": "2", "val_color": CYAN},
            O("}", FG),
        ],
        "duration": 2400,
    },
    # -- Frame 7: guardrail fires --
    {
        "title": TITLE,
        "caption": "guardrails — outbound allowlist + bulk-delete cap",
        "lines": [
            P("git push reposix main   # tries to delete 6 issues"),
            O("error: refusing to push (would delete 6 issues; cap is 5;", RED),
            O("       commit tag '[allow-bulk-delete]' overrides)", RED),
            O(" ! [remote rejected] main -> main (bulk-delete)", RED),
            O(""),
            O("# lethal-trifecta mitigation: tainted-in, side-effects-out.", DIM),
        ],
        "duration": 2400,
    },
    # -- Frame 8: closing card --
    {
        "title": TITLE,
        "caption": "files in, git out. no tool-schemas in the middle.",
        "lines": [
            O("  reposix", CYAN),
            O("  REST issue trackers, mounted as a POSIX tree.", FG),
            O(""),
            O("  $ ls     — list issues", DIM),
            O("  $ sed    — edit in place", DIM),
            O("  $ git    — push the diff", DIM),
            O(""),
            O("  agents, but with files.", PROMPT_GREEN),
        ],
        "duration": 2600,
        "hero": True,
    },
]


# ---- rendering -------------------------------------------------------------

def _font(size: int, bold: bool = False, sans: bool = False) -> ImageFont.FreeTypeFont:
    path = (
        FONT_SANS_BOLD if (sans and bold)
        else FONT_SANS if sans
        else FONT_BOLD if bold
        else FONT_PATH
    )
    return ImageFont.truetype(path, size)


def _draw_chrome(draw: ImageDraw.ImageDraw, title: str) -> None:
    # terminal title bar
    draw.rectangle([0, 0, W, CHROME_H], fill=CHROME)
    draw.line([0, CHROME_H, W, CHROME_H], fill=CHROME_BORDER, width=1)
    # traffic lights
    cy = CHROME_H // 2
    for i, color in enumerate([(255, 95, 86), (255, 189, 46), (39, 201, 63)]):
        cx = 22 + i * 22
        draw.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=color)
    # title text
    f = _font(16, sans=True)
    bbox = draw.textbbox((0, 0), title, font=f)
    tw = bbox[2] - bbox[0]
    draw.text(((W - tw) // 2, (CHROME_H - 18) // 2), title, font=f, fill=DIM)


def _draw_watermark(draw: ImageDraw.ImageDraw) -> None:
    text = "reposix — agents, but with files"
    f = _font(14, sans=True)
    bbox = draw.textbbox((0, 0), text, font=f)
    tw = bbox[2] - bbox[0]
    th = bbox[3] - bbox[1]
    draw.text((W - tw - 20, H - th - 16), text, font=f, fill=WATERMARK)


def _draw_caption(draw: ImageDraw.ImageDraw, caption: str) -> None:
    f = _font(16, sans=True)
    y = H - 52
    draw.text((PAD_X, y), caption, font=f, fill=DIM)


def _draw_prompt_line(
    draw: ImageDraw.ImageDraw, x: int, y: int, cmd: str, mono: ImageFont.FreeTypeFont
) -> None:
    # green $, space, then white cmd with simple token coloring
    prompt = "$ "
    draw.text((x, y), prompt, font=mono, fill=PROMPT_GREEN)
    pw = draw.textbbox((0, 0), prompt, font=mono)[2]

    # super-lightweight token coloring: first token = command (cyan-ish), rest white.
    # but highlight in_progress / in_review / open / p1 / bug keywords
    tokens = cmd.split(" ")
    cur_x = x + pw
    first = True
    for t in tokens:
        if not t:
            cur_x += draw.textbbox((0, 0), " ", font=mono)[2]
            continue
        color = FG
        if first:
            color = CYAN
            first = False
        elif t.startswith("#"):
            color = DIM
        elif "in_progress" in t:
            color = ORANGE
        elif "in_review" in t or "[allow-bulk-delete]" in t:
            color = PROMPT_GREEN
        elif "reposix" in t:
            color = CYAN
        elif t.startswith("'") or t.startswith('"'):
            color = YELLOW
        draw.text((cur_x, y), t + " ", font=mono, fill=color)
        cur_x += draw.textbbox((0, 0), t + " ", font=mono)[2]


def render_frame(frame: dict) -> Image.Image:
    img = Image.new("RGB", (W, H), BG)
    draw = ImageDraw.Draw(img)
    _draw_chrome(draw, frame["title"])

    mono = _font(FONT_SIZE)
    mono_big = _font(44, bold=True)

    y = PAD_Y_TOP

    if frame.get("hero"):
        # Hero/closing card: centered-ish, larger text block.
        y = 130
        for line in frame["lines"]:
            if line.get("big"):
                f = mono_big
                draw.text((PAD_X + 40, y), line["text"], font=f, fill=line["color"])
                y += 68
            else:
                f = _font(22)
                draw.text((PAD_X + 40, y), line["text"], font=f, fill=line["color"])
                y += 34
    else:
        for line in frame["lines"]:
            if line["kind"] == "prompt":
                _draw_prompt_line(draw, PAD_X, y, line["cmd"], mono)
            elif line["kind"] == "kv":
                # YAML-ish key: val with colored val
                key = line["key"]
                val = line["val"]
                vcolor = line.get("val_color", FG)
                draw.text((PAD_X, y), f"{key}:", font=mono, fill=CYAN)
                kw = draw.textbbox((0, 0), f"{key}: ", font=mono)[2]
                draw.text((PAD_X + kw, y), val, font=mono, fill=vcolor)
            elif line["kind"] == "json":
                key = line["key"]
                val = line["val"]
                vcolor = line.get("val_color", FG)
                prefix = f'  "{key}": '
                draw.text((PAD_X, y), prefix, font=mono, fill=CYAN)
                kw = draw.textbbox((0, 0), prefix, font=mono)[2]
                draw.text((PAD_X + kw, y), f"{val},", font=mono, fill=vcolor)
            else:
                draw.text((PAD_X, y), line["text"], font=mono, fill=line["color"])
            y += LINE_HEIGHT

    if "caption" in frame and not frame.get("hero"):
        _draw_caption(draw, frame["caption"])
    _draw_watermark(draw)
    return img


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    frames_img: list[Image.Image] = []
    durations: list[int] = []
    for f in FRAMES:
        frames_img.append(render_frame(f))
        durations.append(f.get("duration", 2000))

    # Save frame 0 PNG for sanity check
    frames_img[0].save(FRAME0_PNG, "PNG", optimize=True)

    # Quantize each frame to an adaptive palette for compact GIF
    pal_frames = [fr.convert("P", palette=Image.ADAPTIVE, colors=128) for fr in frames_img]
    pal_frames[0].save(
        GIF_PATH,
        save_all=True,
        append_images=pal_frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
        disposal=2,
    )

    size = GIF_PATH.stat().st_size
    total_ms = sum(durations)
    with Image.open(GIF_PATH) as g:
        n = getattr(g, "n_frames", 1)
        dims = g.size
    print(f"GIF:       {GIF_PATH}")
    print(f"size:      {size} bytes ({size/1024/1024:.2f} MB)")
    print(f"dims:      {dims[0]}x{dims[1]}")
    print(f"n_frames:  {n}")
    print(f"duration:  {total_ms} ms total ({total_ms/1000:.1f} s)")
    print(f"frame0:    {FRAME0_PNG}  ({FRAME0_PNG.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
