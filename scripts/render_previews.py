"""Create deterministic browser previews from the plugin's exact native-lowered UI tree."""
from pathlib import Path
import html
import json
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "docs" / "validation" / "previews"
BROWSER = Path(r"C:\Program Files\Google\Chrome\Application\chrome.exe")


def render(node):
    tag = node["tag"]
    if tag == "scroll-area":
        tag = "div"
    attributes = []
    styles = dict(node["styles"])
    if node["tag"] == "scroll-area":
        styles["overflow"] = "auto"
    if tag == "progress":
        styles["accent-color"] = "#326d52"
    styles.setdefault("overflow-wrap", "anywhere")
    attributes.append('style="' + html.escape(';'.join(k+':'+v for k, v in styles.items()), quote=True) + '"')
    for key, value in node["props"].items():
        if key in {"value", "max", "min", "type", "placeholder", "role", "aria-label", "aria-checked", "data-layout"}:
            attributes.append(html.escape(key) + '="' + html.escape(value, quote=True) + '"')
    content = html.escape(node.get("text") or "") + ''.join(render(c) for c in node["children"])
    if tag == "input":
        return '<input ' + ' '.join(attributes) + '>'
    return '<'+tag+' '+' '.join(attributes)+'>'+content+'</'+tag+'>'


for source in sorted(DIRECTORY.glob("*.json")):
    payload = json.loads(source.read_text(encoding="utf-8"))
    page = '<!doctype html><html lang="zh-CN"><meta charset="UTF-8"><title>UI 树尺寸预览</title><style>*{box-sizing:border-box}html,body{margin:0;padding:0;background:#171b1a;color:#f4f4f0;font-family:Arial,"Microsoft YaHei",sans-serif}button,input{font:inherit}button{cursor:pointer;text-align:left}</style><body>'+render(payload["tree"])
    destination = source.with_suffix(".html")
    destination.write_text(page, encoding="utf-8")
    if BROWSER.exists():
        subprocess.run([str(BROWSER), "--headless=new", "--disable-gpu", "--hide-scrollbars", "--no-first-run", "--no-default-browser-check", "--force-device-scale-factor=1", "--user-data-dir="+str(ROOT/".tools"/"preview-browser"), "--window-size="+str(payload["width"])+","+str(payload["height"]), "--screenshot="+str(source.with_suffix(".png")), destination.as_uri()], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print("Preview: "+source.stem)
