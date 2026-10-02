#!/usr/bin/env python3
"""Generate GE4G's geometric brutalist launcher mark (requires Pillow)."""
from pathlib import Path
from PIL import Image, ImageDraw
ROOT = Path(__file__).resolve().parents[1]
def mark(size):
    im = Image.new('RGB', (1024, 1024), '#dcff3f')
    draw = ImageDraw.Draw(im)
    draw.rectangle((128, 160, 896, 896), fill='#151515')
    draw.rectangle((96, 96, 832, 832), fill='#f2efe5', outline='#151515', width=40)
    draw.line([(424, 300), (260, 300), (260, 610), (424, 610), (424, 458), (348, 458)], fill='#151515', width=60, joint='curve')
    draw.line([(660, 270), (540, 478), (735, 478)], fill='#151515', width=60)
    draw.line([(680, 342), (680, 640)], fill='#151515', width=60)
    return im.resize((size, size), Image.Resampling.LANCZOS)
def main():
    for path in (ROOT / 'client/android/app/src/main/res').glob('mipmap-*/ic_launcher.png'):
        with Image.open(path) as old: size = old.width
        mark(size).save(path)
    for path in (ROOT / 'client/ios/Runner/Assets.xcassets/AppIcon.appiconset').glob('*.png'):
        with Image.open(path) as old: size = old.width
        mark(size).save(path)
    mark(256).save(ROOT / 'client/windows/runner/resources/app_icon.ico', sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)])
    mark(256).save(ROOT / 'packaging/arch/ge4g.png')
if __name__ == '__main__': main()
