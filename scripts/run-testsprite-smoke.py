from __future__ import annotations

import datetime as dt
import json
import os
import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
TESTS_DIR = ROOT / "testsprite_tests"
RESULTS_FILE = TESTS_DIR / "tmp" / "test_results.json"


def sanitize_title(title: str) -> str:
    return re.sub(r"[^a-zA-Z0-9._]", "", title.replace(" ", "_").replace("-", "_"))


def compact_error(proc: subprocess.CompletedProcess[str]) -> str:
    output = (proc.stdout or "") + (proc.stderr or "")
    output = output.strip()
    if not output:
        output = f"Exited with code {proc.returncode}."
    return output[-4000:]


def main() -> int:
    results = json.loads(RESULTS_FILE.read_text(encoding="utf-8"))
    env = os.environ.copy()
    env.setdefault("BASE_URL", env.get("TESTSPRITE_BASE_URL", "https://bbrainfuckk.github.io/qorx/"))
    env.setdefault("TESTSPRITE_BASE_URL", env["BASE_URL"])
    now = dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    failed = False

    for result in results:
        test_file = TESTS_DIR / f"{sanitize_title(result['title'])}.py"
        result["modified"] = now

        if not test_file.exists():
            result["testStatus"] = "FAILED"
            result["testError"] = f"Missing test file: {test_file.name}"
            failed = True
            print(f"FAILED {result['testId']} {result['title']}: missing file")
            continue

        proc = subprocess.run(
            [sys.executable, str(test_file)],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=120,
        )
        if proc.returncode == 0:
            result["testStatus"] = "PASSED"
            result["testError"] = ""
            print(f"PASSED {result['testId']} {result['title']}")
        else:
            result["testStatus"] = "FAILED"
            result["testError"] = compact_error(proc)
            failed = True
            print(f"FAILED {result['testId']} {result['title']}")
            if result["testError"]:
                print(result["testError"])

    RESULTS_FILE.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
