"""Capture final test evidence using a regular file, preserving Cargo's exit code."""
from pathlib import Path
import json
import re
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
log = ROOT / "docs" / "validation" / "tests.txt"
log.parent.mkdir(parents=True, exist_ok=True)
with log.open("wb") as output:
    result = subprocess.run(["node", str(ROOT / "scripts" / "cargo.mjs"), "test", "--locked", "--all-targets"], cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
text = log.read_text(encoding="utf-8", errors="replace").rstrip() + "\n"
log.write_bytes(text.encode("utf-8"))
results = [tuple(map(int, match)) for match in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed", text)]
summary = {"command": "node scripts/cargo.mjs test --locked --all-targets", "exitCode": result.returncode,
           "passed": sum(p for p, _ in results), "failed": sum(f for _, f in results), "suites": len(results)}
(ROOT / "docs" / "validation" / "tests.json").write_text(json.dumps(summary, indent=2)+"\n", encoding="utf-8")
print(json.dumps(summary))
raise SystemExit(result.returncode)
