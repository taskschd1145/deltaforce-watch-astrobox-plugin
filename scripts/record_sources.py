"""Record raw source identity after final validation, excluding generated reports and tool caches."""
from pathlib import Path
import hashlib
import json
ROOT=Path(__file__).resolve().parents[1]
paths=[]
for directory in ("src","tests","scripts","examples",".github","wit","licenses"):
    paths.extend(p for p in (ROOT/directory).rglob("*") if p.is_file() and "__pycache__" not in p.parts)
for name in ("Cargo.toml","Cargo.lock","rust-toolchain.toml","manifest.json","index.txt","LICENSE","THIRD_PARTY_NOTICES.md","README.md","AGENTS.md",".gitignore",".gitattributes","icon.png"):
    paths.append(ROOT/name)
report={"phase":"post-validation raw-byte source identity","files":[{"path":p.relative_to(ROOT).as_posix(),"bytes":p.stat().st_size,"sha256":hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(paths)]}
target=ROOT/"docs"/"validation"/"source-hashes.json"
target.write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
print(f"Recorded {len(report['files'])} source files.")
