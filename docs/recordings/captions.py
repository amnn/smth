# Copyright (c) Ashok Menon
# SPDX-License-Identifier: Apache-2.0

"""Turn tape comments into frame-synchronized captions below the recorded terminal."""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


CAPTION_HEIGHT = 84
CAPTION_PREFIX = "# caption: "
FONT_SIZE = 32


def prepare(tape, directory):
    """Replace caption comments with screenshots in a temporary copy of the tape."""
    directory.mkdir(parents=True)
    cues = []
    lines = []
    for line in tape.read_text().splitlines():
        lines.append(line)
        if not line.startswith(CAPTION_PREFIX):
            continue
        text = line.removeprefix(CAPTION_PREFIX)
        marker = directory / f"cue-{len(cues):03}.png"
        cues.append((marker, "" if text == "off" else text))
        lines.append(f'Screenshot "{marker}"')
    prepared = directory / tape.name
    prepared.write_text("\n".join(lines) + "\n")
    return prepared, cues


def capture(ffmpeg, args):
    """Save VHS's screenshot frame indices, then delegate unchanged to real FFmpeg."""
    output = Path(args[-1]) if args else Path()
    if re.fullmatch(r"cue-\d+\.png", output.name):
        # VHS renders each Screenshot from numbered text/cursor frame files. Keeping
        # that index avoids guessing elapsed time or matching repeated screen images.
        inputs = [args[i + 1] for i, arg in enumerate(args) if arg == "-i"]
        (frame,) = [
            int(match[1])
            for path in inputs
            if (match := re.fullmatch(r"frame-text-(\d+)\.png", Path(path).name))
        ]
        output.with_suffix(".json").write_text(json.dumps({"frame": frame}))
    elif output.suffix == ".gif":
        output.with_suffix(".timing.json").write_text(
            json.dumps(
                {
                    "framerate": int(args[args.index("-r") + 1]),
                    "start_frame": int(args[args.index("-start_number") + 1]),
                }
            )
        )
    os.execv(ffmpeg, [ffmpeg, *args])


def render(raw, cues, output, theme, ffmpeg):
    """Append a caption track to the GIF without cropping or covering terminal content."""
    (family,) = re.findall(
        r'^Set FontFamily "([^"]+)"$', theme.read_text(), re.MULTILINE
    )
    matched = subprocess.check_output(
        [
            "fc-match",
            "-f",
            "%{family}\n%{file}\n%{index}\n",
            f"{family}:style=Bold",
        ],
        text=True,
    ).splitlines()
    name, path, index = matched
    if family not in name.split(","):
        raise RuntimeError(f"Caption font unavailable: {family} (matched {name})")
    font = ImageFont.truetype(path, FONT_SIZE, index=int(index))
    with Image.open(raw) as first:
        width = first.width
        background = first.convert("RGB").getpixel((0, 0))
    timing = json.loads(raw.with_suffix(".timing.json").read_text())
    rate = timing["framerate"]
    entries = [(0, "")]
    for marker, text in cues:
        frame = json.loads(marker.with_suffix(".json").read_text())["frame"]
        timestamp = (frame - timing["start_frame"]) / rate
        if timestamp <= entries[-1][0]:
            raise RuntimeError(
                f"Caption cues must occupy distinct, increasing frames: {marker}"
            )
        entries.append((timestamp, text))
    probe = json.loads(
        subprocess.check_output(
            [
                "ffprobe",
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "json",
                raw,
            ],
            text=True,
        )
    )
    duration = float(probe["format"]["duration"])
    if entries[-1][0] >= duration:
        raise RuntimeError("Last caption is outside the recording")

    manifest = ["ffconcat version 1.0"]
    for i, (start, text) in enumerate(entries):
        image = Image.new("RGB", (width, CAPTION_HEIGHT), background)
        draw = ImageDraw.Draw(image)
        length = font.getlength(text)
        if length > width - 48:
            raise RuntimeError(f"Caption is too wide: {text}")
        bounds = font.getbbox(text)
        draw.text(
            (
                (width - length) / 2,
                (CAPTION_HEIGHT - bounds[3] + bounds[1]) / 2 - bounds[1],
            ),
            text,
            font=font,
            fill="#141414",
        )
        label = raw.parent / f"label-{i:03}.png"
        image.save(label)
        end = entries[i + 1][0] if i + 1 < len(entries) else duration
        manifest.extend(
            [
                f"file '{label.name}'",
                f"option framerate {rate}",
                f"duration {end - start:.6f}",
            ]
        )
    # The concat demuxer needs a final file to honor the last duration.
    manifest.extend([f"file '{label.name}'", f"option framerate {rate}"])
    track = raw.parent / "captions.ffconcat"
    track.write_text("\n".join(manifest) + "\n")
    subprocess.run(
        [
            ffmpeg,
            "-y",
            "-v",
            "error",
            "-i",
            raw,
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            track,
            "-filter_complex",
            f"[0:v]fps={rate}[terminal];"
            "[terminal][1:v]vstack=inputs=2:shortest=1,split[a][b];"
            "[a]palettegen[p];[b][p]paletteuse[out]",
            "-map",
            "[out]",
            "-loop",
            "0",
            output,
        ],
        check=True,
    )


if __name__ == "__main__":
    capture(sys.argv[1], sys.argv[2:])
