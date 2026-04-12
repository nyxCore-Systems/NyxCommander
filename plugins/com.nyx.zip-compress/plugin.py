#!/usr/bin/env python3
"""
Nyx.Commander action plugin: Zip Compress
Compresses selected files and folders into a zip archive placed in the
target directory (the opposite panel's current path).

Keybinding: Ctrl+Shift+Z
Archive name: <first-item-name>_<YYYYMMDD_HHMMSS>.zip
            or  archive_<timestamp>.zip when multiple items are selected.

Protocol (newline-delimited JSON-RPC on stdin/stdout):
  In:  {"id": N, "method": "run_action", "params": {"files": [...], "target_dir": "..."}}
  Out: {"id": N, "result": {"message": "Zipped 3 items → archive.zip (1.2 MB)"}}
"""

import sys
import json
import os
import zipfile
import datetime


def make_archive_name(files: list[str]) -> str:
    ts = datetime.datetime.now().strftime('%Y%m%d_%H%M%S')
    if len(files) == 1:
        base = os.path.basename(files[0].rstrip('/'))
        # Strip extension for directories-as-sources; keep stem for files
        stem = base.rsplit('.', 1)[0] if '.' in base and not os.path.isdir(files[0]) else base
        return f'{stem}_{ts}.zip'
    return f'archive_{ts}.zip'


def add_path(zf: zipfile.ZipFile, src: str, arc_root: str) -> int:
    """Recursively add src into the zip under arc_root. Returns file count."""
    count = 0
    if os.path.isdir(src):
        dir_name = os.path.basename(src.rstrip('/'))
        for dirpath, dirnames, filenames in os.walk(src):
            # Compute archive path relative to src's parent
            rel = os.path.relpath(dirpath, os.path.dirname(src))
            arc_dir = os.path.join(arc_root, rel)
            for fname in filenames:
                full = os.path.join(dirpath, fname)
                arc_path = os.path.join(arc_dir, fname)
                zf.write(full, arc_path)
                count += 1
    else:
        arc_path = os.path.join(arc_root, os.path.basename(src))
        zf.write(src, arc_path)
        count = 1
    return count


def fmt_bytes(n: int) -> str:
    if n < 1024:
        return f'{n} B'
    if n < 1024 ** 2:
        return f'{n / 1024:.1f} KB'
    if n < 1024 ** 3:
        return f'{n / 1024 ** 2:.1f} MB'
    return f'{n / 1024 ** 3:.2f} GB'


def run_action(params: dict) -> dict:
    files = params.get('files', [])
    target_dir = params.get('target_dir', '')

    if not files:
        return {'message': 'No files selected.'}
    if not target_dir:
        return {'message': 'No target directory specified.'}
    if not os.path.isdir(target_dir):
        return {'message': f'Target directory does not exist: {target_dir}'}

    archive_name = make_archive_name(files)
    archive_path = os.path.join(target_dir, archive_name)

    # Use a flat arc_root: files go directly into the archive root, not nested
    # under a random directory name — matches what users expect from TC's packer.
    total_files = 0
    try:
        with zipfile.ZipFile(archive_path, 'w', zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
            for src in files:
                if not os.path.exists(src):
                    continue
                total_files += add_path(zf, src, '')
    except Exception as e:
        # Clean up partial archive
        try:
            os.remove(archive_path)
        except OSError:
            pass
        return {'message': f'Error: {e}'}

    size = os.path.getsize(archive_path)
    item_word = 'item' if len(files) == 1 else 'items'
    return {
        'message': (
            f'Zipped {len(files)} {item_word} ({total_files} file{"s" if total_files != 1 else ""}) '
            f'→ {archive_name} ({fmt_bytes(size)})'
        )
    }


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError:
            continue

        req_id = req.get('id')
        method = req.get('method', '')
        params = req.get('params', {})

        if method == 'run_action':
            result = run_action(params)
            sys.stdout.write(json.dumps({'id': req_id, 'result': result}) + '\n')
        else:
            sys.stdout.write(json.dumps({
                'id': req_id,
                'error': f'unknown method: {method}',
            }) + '\n')

        sys.stdout.flush()


if __name__ == '__main__':
    main()
