#!/usr/bin/env python3
"""Generate PR Buddy Tauri app and tray icons.

This script is intentionally idempotent. Re-running it overwrites icon assets with the
same deterministic output.

Design: Bold solid white bell on indigo rounded square. No thin strokes, no fine
details — readable at 16×16.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"

BG_COLOR = (91, 110, 245, 255)   # #5B6EF5 — solid indigo
WHITE = (255, 255, 255, 255)
BLACK = (30, 30, 30, 255)        # near-black for macOS tray template


def _draw_bell(draw: ImageDraw.ImageDraw, size: int, fill: tuple) -> None:
    """Draw a bold filled bell glyph centred in the canvas."""
    s = size

    # Bell handle (small rounded rect at top)
    draw.rounded_rectangle(
        (int(s * 0.44), int(s * 0.16), int(s * 0.56), int(s * 0.26)),
        radius=int(s * 0.03),
        fill=fill,
    )

    # Dome (top half-circle)
    draw.pieslice(
        (int(s * 0.25), int(s * 0.20), int(s * 0.75), int(s * 0.70)),
        180,
        360,
        fill=fill,
    )

    # Body (fills gap below dome to the lip)
    draw.rectangle(
        (int(s * 0.25), int(s * 0.45), int(s * 0.75), int(s * 0.68)),
        fill=fill,
    )

    # Bottom lip (wide bar)
    draw.rounded_rectangle(
        (int(s * 0.20), int(s * 0.65), int(s * 0.80), int(s * 0.75)),
        radius=int(s * 0.04),
        fill=fill,
    )

    # Clapper (ball at bottom)
    draw.ellipse(
        (int(s * 0.42), int(s * 0.73), int(s * 0.58), int(s * 0.87)),
        fill=fill,
    )


def create_app_icon(size: int = 1024) -> Image.Image:
    """Solid indigo rounded square with a white filled bell."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    inset = int(size * 0.08)
    radius = int(size * 0.22)
    draw.rounded_rectangle(
        (inset, inset, size - inset, size - inset),
        radius=radius,
        fill=BG_COLOR,
    )

    _draw_bell(draw, size, WHITE)
    return canvas


def create_tray_icon(size: int = 64) -> Image.Image:
    """Dark filled bell on transparent background (macOS template image)."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    _draw_bell(draw, size, BLACK)
    return canvas


def save_png(image: Image.Image, path: Path, size: int) -> None:
    resized = image.resize((size, size), resample=Image.Resampling.LANCZOS)
    resized.save(path, format="PNG", optimize=True)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    app = create_app_icon(1024)
    tray = create_tray_icon(64)

    # PNG app icons
    save_png(app, ICONS_DIR / "32x32.png", 32)
    save_png(app, ICONS_DIR / "128x128.png", 128)
    save_png(app, ICONS_DIR / "128x128@2x.png", 256)
    save_png(app, ICONS_DIR / "icon.png", 512)

    # Tray icon (transparent background, dark bell)
    save_png(tray, ICONS_DIR / "tray-default.png", 32)

    # Windows ICO (multi-resolution)
    app.save(
        ICONS_DIR / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    # macOS ICNS
    app.save(ICONS_DIR / "icon.icns", format="ICNS")

    # Keep directory clean once real assets exist.
    gitkeep = ICONS_DIR / ".gitkeep"
    if gitkeep.exists():
        gitkeep.unlink()

    print(f"Generated icons in {ICONS_DIR}")


if __name__ == "__main__":
    main()
