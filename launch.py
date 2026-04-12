#!/usr/bin/env python3
"""
launch.py — build and/or run Nyx.Commander

Usage:
  python launch.py          # dev mode (hot-reload)
  python launch.py --build  # release build
  python launch.py --run    # run last release build without rebuilding
"""

import argparse
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
BINARY = os.path.join(ROOT, "src-tauri", "target", "release", "nyx-commander")
APP_BUNDLE = os.path.join(ROOT, "src-tauri", "target", "release", "bundle", "macos", "Nyx.Commander.app")


def run(cmd: list[str], **kwargs):
    print(f"+ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=ROOT, **kwargs)
    if result.returncode != 0:
        sys.exit(result.returncode)


def dev():
    run(["pnpm", "tauri", "dev"])


def build():
    run(["pnpm", "tauri", "build"])


def launch_release():
    if os.path.isdir(APP_BUNDLE):
        run(["open", APP_BUNDLE])
    elif os.path.isfile(BINARY):
        run([BINARY])
    else:
        print("No release build found. Run with --build first.")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(description="Build and launch Nyx.Commander")
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--build", action="store_true", help="Release build (then launch)")
    group.add_argument("--run", action="store_true", help="Launch last release build without rebuilding")
    args = parser.parse_args()

    if args.build:
        build()
        launch_release()
    elif args.run:
        launch_release()
    else:
        dev()


if __name__ == "__main__":
    main()
