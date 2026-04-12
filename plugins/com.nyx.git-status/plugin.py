#!/usr/bin/env python3
"""
Nyx.Commander column plugin: Git Status
JSON-RPC over stdin/stdout (newline-delimited JSON).

Protocol:
  In:  {"id": N, "method": "get_columns", "params": {"files": [{"path": "...", ...}]}}
  Out: {"id": N, "result": [{"path": "...", "value": "✓ clean"}]}
"""

import sys
import json
import subprocess
import os

GIT_LABELS = {
    'M': '~ modified',
    'A': '+ added',
    'D': '- deleted',
    'R': '→ renamed',
    'C': '⎘ copied',
    'U': '! conflict',
    '?': '? untracked',
    '!': '  ignored',
}


def git_status_for_file(path: str) -> str:
    dirp = os.path.dirname(path)
    name = os.path.basename(path)
    if not dirp or not name:
        return ''
    try:
        result = subprocess.run(
            ['git', '-C', dirp, 'status', '--porcelain', '--', name],
            capture_output=True, text=True, timeout=3,
        )
        out = result.stdout.strip()
        if not out:
            # File is tracked and clean, or outside a git repo (non-zero exit)
            if result.returncode == 0:
                return '✓'
            return ''
        code = out[:2].strip()
        if code:
            return GIT_LABELS.get(code[0], code)
        return '✓'
    except Exception:
        return ''


def handle_get_columns(params: dict) -> list:
    files = params.get('files', [])
    results = []
    for f in files:
        path = f.get('path', '')
        value = git_status_for_file(path) if path else ''
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
