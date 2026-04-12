#!/usr/bin/env python3
"""
Nyx.Commander column plugin: Folder Size
Shows total disk usage for directories, and confirms file sizes.

Uses `du -sk` on macOS/Linux for directories — fast because the OS caches
inode data. Falls back to os.path.getsize() for files (already known, but
shown here so the column is consistent for all entry types).

Protocol (newline-delimited JSON-RPC on stdin/stdout):
  In:  {"id": N, "method": "get_columns", "params": {"files": [...]}}
  Out: {"id": N, "result": [{"path": "...", "value": "42.3 MB"}]}
"""

import sys
import json
import os
import subprocess


def fmt_bytes(n: int) -> str:
    """Human-readable byte count, max 5 chars wide."""
    if n < 0:
        return ''
    if n < 1024:
        return f'{n} B'
    if n < 1024 ** 2:
        return f'{n / 1024:.1f} K'
    if n < 1024 ** 3:
        return f'{n / 1024 ** 2:.1f} M'
    if n < 1024 ** 4:
        return f'{n / 1024 ** 3:.1f} G'
    return f'{n / 1024 ** 4:.1f} T'


def dir_size(path: str) -> str:
    """Run `du -sk` to get total disk usage of a directory."""
    try:
        result = subprocess.run(
            ['du', '-sk', path],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode != 0:
            return ''
        # du output: "<kblocks>\t<path>\n"
        raw = result.stdout.split('\t')[0].strip()
        if not raw.isdigit():
            return ''
        return fmt_bytes(int(raw) * 1024)
    except Exception:
        return ''


def file_size(path: str, reported_size: int) -> str:
    """Return size for a regular file. Prefer the already-known value."""
    if reported_size > 0:
        return fmt_bytes(reported_size)
    try:
        return fmt_bytes(os.path.getsize(path))
    except OSError:
        return ''


def handle_get_columns(params: dict) -> list:
    files = params.get('files', [])
    results = []
    for f in files:
        path = f.get('path', '')
        is_dir = f.get('is_dir', False)
        name = f.get('name', '')
        size = f.get('size', 0)

        if not path or name == '..':
            results.append({'path': path, 'value': ''})
            continue

        if is_dir:
            value = dir_size(path)
        else:
            value = file_size(path, size)

        results.append({'path': path, 'value': value})
    return results


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

        if method == 'get_columns':
            result = handle_get_columns(params)
            sys.stdout.write(json.dumps({'id': req_id, 'result': result}) + '\n')
        else:
            sys.stdout.write(json.dumps({
                'id': req_id,
                'error': f'unknown method: {method}',
            }) + '\n')

        sys.stdout.flush()


if __name__ == '__main__':
    main()
