"""The desktop app's window icon: prompt-icon-64.png as raw RGBA (64 × 64 × 4 bytes),
which tao takes without a PNG decoder. Run after render.mjs:
python3 images/brand/make-icon-rgba.py"""
import pathlib

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent.parent / "apps/lumacore-app/assets/icon-64.rgba"
image = Image.open(HERE / "prompt-icon-64.png").convert("RGBA")
assert image.size == (64, 64), image.size
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_bytes(image.tobytes())
print(f"{OUT} ({OUT.stat().st_size} bytes)")
