"""Draw an original geometric sync emblem with the product's tactical palette."""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
image = Image.new("RGBA", (256, 256), (23, 27, 26, 255))
d = ImageDraw.Draw(image)
d.rounded_rectangle((10, 10, 246, 246), radius=16, fill="#202723", outline="#39463e", width=3)
d.polygon([(128, 40), (204, 84), (204, 172), (128, 216), (52, 172), (52, 84)], fill="#171b1a", outline="#326d52", width=5)
# Two independent opposing arrows form a synchronization loop.
d.line([(79, 105), (111, 73), (169, 73)], fill="#f2bd74", width=12, joint="curve")
d.polygon([(169, 56), (193, 73), (169, 90)], fill="#f2bd74")
d.line([(177, 151), (145, 183), (87, 183)], fill="#8fbda4", width=12, joint="curve")
d.polygon([(87, 166), (63, 183), (87, 200)], fill="#8fbda4")
d.polygon([(128, 95), (152, 141), (104, 141)], outline="#f4f4f0", width=6)
d.rectangle((113, 154, 143, 158), fill="#f4f4f0")
image.save(ROOT / "icon.png", optimize=True)
print("Generated original 256x256 icon.png")
