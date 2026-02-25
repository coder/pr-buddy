#!/usr/bin/env python3
"""Generate PR Buddy Tauri app and tray icons.

This script is intentionally idempotent. Re-running it overwrites icon assets with the
same deterministic output.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"

INDIGO_TOP = (79, 70, 229, 255)  # #4f46e5
INDIGO_BOTTOM = (124, 58, 237, 255)  # #7c3aed
WHITE = (255, 255, 255, 255)


def _lerp(a: int, b: int, t: float) -> int:
    return int(round(a + (b - a) * t))


def create_gradient(size: int) -> Image.Image:
    """Create a vertical indigo gradient with subtle lighting."""
    column = Image.new("RGBA", (1, size))
    draw = ImageDraw.Draw(column)
    for y in range(size):
        t = y / max(1, size - 1)
        color = tuple(_lerp(INDIGO_TOP[i], INDIGO_BOTTOM[i], t) for i in range(4))
        draw.point((0, y), fill=color)

    gradient = column.resize((size, size), resample=Image.Resampling.BICUBIC)

    # Add soft highlight near top-left for depth.
    highlight = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    hdraw = ImageDraw.Draw(highlight)
    hdraw.ellipse(
        (
            int(size * 0.18),
            int(size * 0.06),
            int(size * 0.92),
            int(size * 0.78),
        ),
        fill=(255, 255, 255, 72),
    )
    highlight = highlight.filter(ImageFilter.GaussianBlur(radius=size * 0.08))

    return Image.alpha_composite(gradient, highlight)


def create_base_icon(size: int = 1024) -> Image.Image:
    """Create the core icon image: rounded indigo square with white bell glyph."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    gradient = create_gradient(size)

    # Rounded square background shape.
    bg_mask = Image.new("L", (size, size), 0)
    mdraw = ImageDraw.Draw(bg_mask)
    inset = int(size * 0.08)
    radius = int(size * 0.22)
    mdraw.rounded_rectangle(
        (inset, inset, size - inset, size - inset), radius=radius, fill=255
    )
    canvas.paste(gradient, (0, 0), bg_mask)

    # Bell glyph, drawn with primitives so it remains crisp when downsampled.
    draw = ImageDraw.Draw(canvas)

    # Bell handle
    draw.rounded_rectangle(
        (
            int(size * 0.46),
            int(size * 0.18),
            int(size * 0.54),
            int(size * 0.27),
        ),
        radius=int(size * 0.025),
        fill=WHITE,
    )

    # Dome + body
    draw.pieslice(
        (
            int(size * 0.27),
            int(size * 0.22),
            int(size * 0.73),
            int(size * 0.72),
        ),
        180,
        360,
        fill=WHITE,
    )
    draw.rounded_rectangle(
        (
            int(size * 0.31),
            int(size * 0.45),
            int(size * 0.69),
            int(size * 0.74),
        ),
        radius=int(size * 0.11),
        fill=WHITE,
    )

    # Bottom lip
    draw.rounded_rectangle(
        (
            int(size * 0.24),
            int(size * 0.67),
            int(size * 0.76),
            int(size * 0.76),
        ),
        radius=int(size * 0.04),
        fill=WHITE,
    )

    # Clapper
    draw.ellipse(
        (
            int(size * 0.43),
            int(size * 0.73),
            int(size * 0.57),
            int(size * 0.88),
        ),
        fill=WHITE,
    )

    return canvas


def save_png(image: Image.Image, path: Path, size: int) -> None:
    resized = image.resize((size, size), resample=Image.Resampling.LANCZOS)
    resized.save(path, format="PNG", optimize=True)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    base = create_base_icon(1024)

    # PNG app + tray icons
    save_png(base, ICONS_DIR / "32x32.png", 32)
    save_png(base, ICONS_DIR / "128x128.png", 128)
    save_png(base, ICONS_DIR / "128x128@2x.png", 256)
    save_png(base, ICONS_DIR / "tray-default.png", 32)

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
