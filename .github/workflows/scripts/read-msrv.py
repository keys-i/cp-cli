import os
import tomllib
from pathlib import Path

manifest = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))
package = manifest.get("package", {})
workspace_package = manifest.get("workspace", {}).get("package", {})
msrv = package.get("rust-version") or workspace_package.get("rust-version") or ""

with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
    print(f"msrv={msrv}", file=output)
