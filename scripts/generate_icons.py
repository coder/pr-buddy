#!/usr/bin/env python3
"""Generate PR Buddy Tauri app and tray icons from SVG sources.

This script is intentionally idempotent. Re-running it overwrites icon assets with the
same deterministic output.

Design: Filled bell with GitHub mark cutout (negative space).
- App icon: indigo rounded rect, white bell, GitHub mark reveals background.
- Tray icon: white bell with GitHub cutout on transparent (macOS template).

SVG sources live alongside this script:
  scripts/icon-app.svg   — app icon
  scripts/icon-tray.svg  — tray template icon

Requires: cairosvg, Pillow
"""

from __future__ import annotations

import io
from pathlib import Path

import cairosvg
from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"
SCRIPTS_DIR = ROOT / "scripts"


def render_svg(svg_path: Path, size: int) -> Image.Image:
    """Render an SVG file to an RGBA PIL Image at the given pixel size."""
    png_data = cairosvg.svg2png(
        url=str(svg_path),
        output_width=size,
        output_height=size,
    )
    return Image.open(io.BytesIO(png_data)).convert("RGBA")


def create_tray_icon(size: int) -> Image.Image:
    """White bell with GitHub mark punched out on transparent background.

    cairosvg doesn't handle SVG masks well, so the cutout is done via
    Pillow alpha compositing: render the bell and GitHub mark separately,
    then subtract the mark's alpha from the bell's alpha.
    """
    bell = render_svg(SCRIPTS_DIR / "icon-tray.svg", size)
    github = render_svg(SCRIPTS_DIR / "icon-github.svg", size)

    bell_a = bell.split()[3]
    github_a = github.split()[3]
    result_a = ImageChops.subtract(bell_a, github_a)

    result = bell.copy()
    result.putalpha(result_a)
    return result


def save_png(image: Image.Image, path: Path, size: int) -> None:
    resized = image.resize((size, size), resample=Image.Resampling.LANCZOS)
    resized.save(path, format="PNG", optimize=True)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    app_svg = SCRIPTS_DIR / "icon-app.svg"

    # Render at high resolution for quality downscaling
    app = render_svg(app_svg, 1024)
    tray = create_tray_icon(256)

    # PNG app icons
    save_png(app, ICONS_DIR / "32x32.png", 32)
    save_png(app, ICONS_DIR / "128x128.png", 128)
    save_png(app, ICONS_DIR / "128x128@2x.png", 256)
    save_png(app, ICONS_DIR / "icon.png", 512)

    # Tray icon (transparent background, white bell+cutout — macOS template image)
    save_png(tray, ICONS_DIR / "tray-default.png", 32)

    # Windows ICO (multi-resolution)
    app.save(
        ICONS_DIR / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

    # macOS ICNS — used by Finder, Dock, and Notification Center
    app.save(ICONS_DIR / "icon.icns", format="ICNS")

    # Keep directory clean once real assets exist.
    gitkeep = ICONS_DIR / ".gitkeep"
    if gitkeep.exists():
        gitkeep.unlink()

    print(f"Generated icons in {ICONS_DIR}")


if __name__ == "__main__":
    main()
