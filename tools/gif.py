#!/usr/bin/env python3
"""Turn a directory of `capture` frames into a caption-burned GIF.

Written in Pillow rather than as an ffmpeg/ImageMagick pipeline because neither binary is installed on
every machine this repo is built on, and the failure landed *after* the frames had been rendered — two
runs lost that way. Pillow is already present wherever the evidence plots are made.

    tools/gif.py <frames-dir> <out.gif> "<caption>"

Palette and dither are fixed here so two GIFs a week apart are comparable: a palette change between runs
would look like a visual difference that no code change caused.
"""

from __future__ import annotations

import pathlib
import sys

from PIL import Image, ImageDraw, ImageFont

# **256, the format's maximum.** At 128 the amber, teal and blue faction capsules compressed toward one
# another — three colours meant to be told apart at a glance, quantized to nearly the same shade. The
# file cost is small next to a demo whose whole subject is colour.
COLORS = 256
# The caption is burned in: a GIF that needs a README sentence to be understood is not readable on its own.
CAPTION_HEIGHT = 30
FONT_CANDIDATES = [
    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
]


def load_font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for path in FONT_CANDIDATES:
        if pathlib.Path(path).is_file():
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def main() -> int:
    if len(sys.argv) < 4:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    frames_dir = pathlib.Path(sys.argv[1])
    out_path = pathlib.Path(sys.argv[2])
    caption = sys.argv[3]

    paths = sorted(frames_dir.glob("frame*.png"))
    if not paths:
        print(f"gif.py: no frame*.png in {frames_dir}", file=sys.stderr)
        return 1

    width_env = int(sys.argv[4]) if len(sys.argv) > 4 else 720
    stride = int(sys.argv[5]) if len(sys.argv) > 5 else 1
    fps = int(sys.argv[6]) if len(sys.argv) > 6 else 30

    first = Image.open(paths[0]).convert("RGB")
    if first.width != width_env:
        height = round(first.height * width_env / first.width)
    else:
        height = first.height
    canvas_size = (width_env, height)

    font = load_font(15)
    frames: list[Image.Image] = []
    for path in paths[::stride]:
        image = Image.open(path).convert("RGB")
        if image.size != canvas_size:
            image = image.resize(canvas_size, Image.LANCZOS)
        strip = Image.new("RGB", (canvas_size[0], canvas_size[1] + CAPTION_HEIGHT), (12, 16, 22))
        strip.paste(image, (0, 0))
        draw = ImageDraw.Draw(strip)
        draw.text((10, canvas_size[1] + 7), caption, font=font, fill=(255, 255, 255))
        frames.append(strip)

    # **One palette, built from the frames themselves.** Each frame is first reduced to `COLORS` with an
    # adaptive palette, then every frame is re-mapped onto the *first* frame's palette. Sampling eight
    # frames into a vertical montage and quantizing that produced a palette dominated by grass — the
    # faction colours (amber, teal, blue) disappeared and every villager came out grey-green. Reducing a
    # full frame adaptively keeps the colours that are actually in it.
    palette = frames[0].quantize(colors=COLORS, method=Image.Quantize.MEDIANCUT)
    quantized = [frame.quantize(palette=palette, dither=Image.Dither.NONE) for frame in frames]
    quantized[0].save(
        out_path,
        save_all=True,
        append_images=quantized[1:],
        duration=round(1000 / fps * stride),
        loop=0,
        optimize=True,
        disposal=1,
    )
    size_mb = out_path.stat().st_size / (1024 * 1024)
    print(f"gif.py: {len(paths)} frames (every {stride}) -> {out_path} ({size_mb:.1f} MiB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
