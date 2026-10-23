"""P2-08: compare pixels independently from workflow and semantic assertions."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import binascii
import struct
import zlib


def ppm(path):
    magic, geometry, maximum, pixels = path.read_bytes().split(b"\n", 3)
    width, height = map(int, geometry.split())
    assert magic == b"P6" and maximum == b"255" and len(pixels) == width * height * 3
    return width, height, pixels


def png(path, width, height, pixels):
    def chunk(kind, data):
        return struct.pack("!I", len(data)) + kind + data + struct.pack("!I", binascii.crc32(kind + data) & 0xffffffff)
    rows = b"".join(b"\0" + pixels[y*width*3:(y+1)*width*3] for y in range(height))
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack("!IIBBBBB", width, height, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))


def difference(a, b):
    assert len(a) == len(b) and len(a) % 3 == 0
    pixels = len(a) // 3
    value = int.from_bytes(a, "little") ^ int.from_bytes(b, "little")
    # One independent 24-bit lane per RGB pixel. Adding the low-23-bit mask
    # raises the lane's high bit iff any low bit is nonzero; OR retains bit 23.
    # No lane can overflow 24 bits, so neighboring lanes cannot affect the count.
    low = int.from_bytes(b"\xff\xff\x7f" * pixels, "little")
    high = int.from_bytes(b"\0\0\x80" * pixels, "little")
    return ((((value & low) + low) | value) & high).bit_count(), value.bit_count()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(custom, slint, output):
    custom, slint, output = [Path(p).resolve() for p in (custom, slint, output)]
    output.mkdir(parents=True, exist_ok=True)
    results = [read(p / "result.json") for p in (custom, slint)]
    assert all(r["status"] == "passed" and r["goldens_checked"] for r in results)
    assert read(custom / "traces.json") == read(slint / "traces.json")
    assert read(custom / "semantics.json") == read(slint / "semantics.json")
    captures = [r["id"] for r in results[0]["observations"]]
    assert captures == [r["id"] for r in results[1]["observations"]] and len(captures) == 150
    rows = []
    for name in captures:
        paths = [p / (name + ".ppm") for p in (custom, slint)]
        images = [ppm(p) for p in paths]
        assert images[0][:2] == images[1][:2]
        changed, bits = difference(images[0][2], images[1][2])
        pixels = images[0][0] * images[0][1]
        rows.append({"id": name, "pixels": pixels, "different_pixels": changed, "different_rgb_bits": bits, "custom_ppm_sha256": digest(paths[0]), "slint_ppm_sha256": digest(paths[1]), "interpretation": "exact differing RGB pixels/bits; descriptive, no pass tolerance or parity inferred"})
    with (output / "pixel-differences.csv").open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    names = ["1280x720-100-standard-initial", "1280x720-150-standard-search-keyboard", "1280x720-200-high-contrast-initial", "1920x1080-100-reduced-effects-file-scroll-select", "1920x1080-150-reduced-motion-effects-toggle", "1920x1080-200-high-contrast-reduced-composition-commit"]
    sheet = bytearray(960 * 6 * 270 * 3)
    for n, name in enumerate(names):
        scale = int(name.split("-")[1]) / 100
        for column, directory in enumerate((custom, slint)):
            width, height, pixels = ppm(directory / (name + ".ppm"))
            for y in range(270):
                sy = min(height-1, int((y+.5) * 640 / 480 * scale))
                for x in range(480):
                    sx = min(width-1, int((x+.5) * 640 / 480 * scale))
                    source = (sy * width + sx) * 3
                    dest = ((n * 270 + y) * 960 + column * 480 + x) * 3
                    sheet[dest:dest+3] = pixels[source:source+3]
    png(output / "comparison.png", 960, 1620, sheet)
    assert (output / "comparison.png").stat().st_size <= 256 * 1024
    evidence = {"status": "passed independent functional equivalence; image differences retained", "workflow_states_per_candidate": len(read(custom / "traces.json")), "semantic_captures_per_candidate": len(read(custom / "semantics.json")), "captures_per_candidate": 150, "text_cases_per_candidate": len(results[0]["text"]), "text_nodes_per_candidate": len(results[0]["text_nodes"]), "sources": [{"directory": str(p.relative_to(Path.cwd())).replace("\\", "/"), "result_sha256": digest(p / "result.json"), "traces_sha256": digest(p / "traces.json"), "semantics_sha256": digest(p / "semantics.json"), "negative": r["negative"], "text": r["text"], "text_nodes": r["text_nodes"]} for p, r in zip((custom, slint), results)], "contact_sheet_cases": names, "visual_review": "recorded separately in quality-review.md; contact sheet downsampled for review, full original rasters retained locally", "native_ime": False, "native_accessibility_consumer": False}
    (output / "quality.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print(f"150 image pairs, 300 identical workflow states and 150 identical semantic captures: {output}")


if __name__ == "__main__":
    # Analytic and exhaustive channel cases independently corroborate lane counting.
    values = bytes(range(256)) * 3
    expected = sum(values[n:n+3] != b"\0\0\0" for n in range(0, len(values), 3))
    assert difference(values, bytes(len(values)))[0] == expected
    assert difference(b"\0\0\0\0\0\x80\xff\xff\xff", bytes(9)) == (2, 25)
    main(*sys.argv[1:])
