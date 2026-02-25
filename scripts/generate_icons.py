#!/usr/bin/env python3
"""Generate PR Buddy Tauri app and tray icons.

This script is intentionally idempotent. Re-running it overwrites icon assets with the
same deterministic output.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"

APP_BACKGROUND = (240, 240, 245, 255)  # #f0f0f5
APP_STROKE = (51, 51, 69, 255)  # #333345
ACCENT_STROKE = (45, 45, 63, 255)  # #2d2d3f
TRAY_STROKE = (20, 20, 24, 255)  # near-black for macOS template rendering


def _stroke(size: int, ratio: float = 0.035) -> int:
    """Return a size-proportional stroke width."""
    return max(1, int(round(size * ratio)))


def draw_bell_outline(
    draw: ImageDraw.ImageDraw,
    size: int,
    stroke: int,
    color: tuple[int, int, int, int],
    bounds: tuple[float, float, float, float],
) -> None:
    """Draw an outlined notification bell using simple geometric primitives."""
    left, top, right, bottom = [int(size * v) for v in bounds]
    width = right - left
    height = bottom - top

    # Bell handle
    handle_w = int(width * 0.18)
    handle_h = int(height * 0.14)
    handle_x0 = left + (width - handle_w) // 2
    handle_y0 = top
    draw.rounded_rectangle(
        (handle_x0, handle_y0, handle_x0 + handle_w, handle_y0 + handle_h),
        radius=max(1, stroke // 2),
        outline=color,
        width=stroke,
    )

    # Dome and sides
    shoulder_y = top + int(height * 0.22)
    body_bottom = bottom - int(height * 0.18)
    dome_box = (left + int(width * 0.05), shoulder_y - int(height * 0.32), right - int(width * 0.05), shoulder_y + int(height * 0.58))
    draw.arc(dome_box, start=205, end=335, fill=color, width=stroke)

    side_inset = int(width * 0.14)
    draw.line((left + side_inset, shoulder_y + int(height * 0.12), left + side_inset, body_bottom), fill=color, width=stroke)
    draw.line((right - side_inset, shoulder_y + int(height * 0.12), right - side_inset, body_bottom), fill=color, width=stroke)

    # Bottom lip
    lip_box = (
        left + int(width * 0.06),
        body_bottom - int(height * 0.12),
        right - int(width * 0.06),
        body_bottom + int(height * 0.12),
    )
    draw.arc(lip_box, start=18, end=162, fill=color, width=stroke)

    # Clapper
    clapper_r = max(stroke, int(width * 0.07))
    clapper_x = left + width // 2
    clapper_y = body_bottom + int(height * 0.07)
    draw.ellipse(
        (
            clapper_x - clapper_r,
            clapper_y - clapper_r,
            clapper_x + clapper_r,
            clapper_y + clapper_r,
        ),
        outline=color,
        width=max(1, int(stroke * 0.8)),
    )


def draw_merge_accent(
    draw: ImageDraw.ImageDraw,
    size: int,
    stroke: int,
    color: tuple[int, int, int, int],
    anchor: tuple[float, float] = (0.65, 0.67),
) -> None:
    """Draw a subtle git-merge style Y accent near the bell's lower-right area."""
    ax, ay = anchor
    p1 = (int(size * (ax - 0.10)), int(size * (ay - 0.10)))
    p2 = (int(size * (ax - 0.10)), int(size * ay))
    merge = (int(size * ax), int(size * (ay - 0.04)))
    tail = (int(size * (ax + 0.11)), int(size * (ay + 0.10)))

    accent_stroke = max(1, int(round(stroke * 0.8)))
    draw.line((p1, merge), fill=color, width=accent_stroke)
    draw.line((p2, merge), fill=color, width=accent_stroke)
    draw.line((merge, tail), fill=color, width=accent_stroke)

    node_r = max(1, int(round(accent_stroke * 0.9)))
    draw.ellipse((p1[0] - node_r, p1[1] - node_r, p1[0] + node_r, p1[1] + node_r), fill=color)
    draw.ellipse((p2[0] - node_r, p2[1] - node_r, p2[0] + node_r, p2[1] + node_r), fill=color)


def create_base_icon(size: int = 1024) -> Image.Image:
    """Create app icon: neutral rounded square + outlined bell + merge accent."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    inset = int(size * 0.08)
    radius = int(size * 0.22)
    draw.rounded_rectangle((inset, inset, size - inset, size - inset), radius=radius, fill=APP_BACKGROUND)

    bell_stroke = _stroke(size, 0.035)
    draw_bell_outline(
        draw,
        size,
        stroke=bell_stroke,
        color=APP_STROKE,
        bounds=(0.24, 0.20, 0.76, 0.84),
    )
    draw_merge_accent(draw, size, stroke=bell_stroke, color=ACCENT_STROKE, anchor=(0.66, 0.70))

    return canvas


def create_tray_icon(size: int = 256) -> Image.Image:
    """Create tray icon: transparent background with dark outlined bell + merge accent."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    bell_stroke = _stroke(size, 0.042)
    draw_bell_outline(
        draw,
        size,
        stroke=bell_stroke,
        color=TRAY_STROKE,
        bounds=(0.16, 0.10, 0.84, 0.90),
    )
    draw_merge_accent(draw, size, stroke=bell_stroke, color=TRAY_STROKE, anchor=(0.68, 0.70))

    return canvas


def save_png(image: Image.Image, path: Path, size: int) -> None:
    resized = image.resize((size, size), resample=Image.Resampling.LANCZOS)
    resized.save(path, format="PNG", optimize=True)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    base = create_base_icon(1024)
    tray = create_tray_icon(256)

    # PNG app + tray icons
    save_png(base, ICONS_DIR / "32x32.png", 32)
    save_png(base, ICONS_DIR / "128x128.png", 128)
    save_png(base, ICONS_DIR / "128x128@2x.png", 256)
    save_png(tray, ICONS_DIR / "tray-default.png", 32)

    # Optional convenience PNG for docs/manual checks.
    save_png(base, ICONS_DIR / "icon.png", 512)

    # Windows ICO (multi-resolution)
    base.save(
        ICONS_DIR / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    # macOS ICNS
    base.save(ICONS_DIR / "icon.icns", format="ICNS")

    # Keep directory clean once real assets exist.
    gitkeep = ICONS_DIR / ".gitkeep"
    if gitkeep.exists():
        gitkeep.unlink()

    print(f"Generated icons in {ICONS_DIR}")


if __name__ == "__main__":
    main()
