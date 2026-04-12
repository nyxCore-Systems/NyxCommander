# Writing Plugins

A Nyx.Commander plugin is a folder containing a `plugin.json` manifest and one entrypoint file. No build toolchain required for viewer plugins; column/action/panel plugins are executables in any language.

---

## plugin.json — Manifest

All fields:

```json
{
  "id": "com.example.my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "One-line description shown in Plugin Manager",
  "author": "Your Name",
  "category": "viewer",
  "fileExtensions": ["png", "jpg"],
  "entrypoint": "viewer.html",
  "minAppVersion": "0.1.0",
  "enabled": true
}
```

**Required fields:** `id`, `name`, `version`, `category`, `entrypoint`

**Category-specific extra fields:**

| Category | Extra field | Type | Meaning |
|----------|------------|------|---------|
| `column` | `columnName` | string | Header label in the panel |
| `column` | `columnWidth` | number | Column width in pixels (default 72) |
| `action` | `keybinding` | string | Trigger key e.g. `"ctrl+shift+z"` |
| `action` | `menuLabel` | string | Label shown in Plugin Manager |
| `panel` | `protocol` | string | URI prefix e.g. `"s3://"` |

**`fileExtensions`** — lowercase, without the dot. Leave empty (`[]`) for column/action plugins that apply to all files.

---

## Viewer Plugins

A viewer plugin is a self-contained HTML file. Nyx renders it in a sandboxed `<iframe>` and communicates via `postMessage`.

### Sandbox constraints

The iframe runs with `sandbox="allow-scripts"`. This means:
- No Tauri IPC — the plugin cannot call Nyx commands.
- No `localStorage`, no cookies, no popups, no top-level navigation.
- The plugin **can** fetch the file's content using the `fileUrl` passed in the `load` message.

### Receiving the load message

```javascript
window.addEventListener('message', function(e) {
  const msg = e.data;
  if (msg.type !== 'load') return;

  // msg.filePath  — absolute path on disk, e.g. "/Users/me/photo.png"
  // msg.fileUrl   — Tauri asset URL to fetch the file content
  // msg.mimeType  — guessed MIME type, e.g. "image/png"
  // msg.theme     — { bg, fg, accent } — current Nyx colour theme
});
```

### Sending status back

```javascript
// Update the status bar in the viewer title bar
window.parent.postMessage({ type: 'status', text: '1024×768 · PNG' }, '*');

// Report an error (shown in red in the title bar)
window.parent.postMessage({ type: 'error', message: 'Decode failed' }, '*');
```

### Minimal example

```html
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  body { margin: 0; background: #0a0e27; display: flex;
         align-items: center; justify-content: center; height: 100vh; }
  img  { max-width: 100%; max-height: 100%; object-fit: contain; }
</style>
</head>
<body>
<img id="img" alt="">
<script>
  window.addEventListener('message', function(e) {
    const msg = e.data;
    if (msg.type !== 'load') return;
    if (msg.theme) document.body.style.background = msg.theme.bg;
    const img = document.getElementById('img');
    img.onload = () => window.parent.postMessage({
      type: 'status', text: img.naturalWidth + '×' + img.naturalHeight
    }, '*');
    img.src = msg.fileUrl;
  });
</script>
</body>
</html>
```

**`plugin.json` for this example:**

```json
{
  "id": "com.example.my-image-viewer",
  "name": "My Image Viewer",
  "version": "1.0.0",
  "author": "you",
  "description": "Shows images",
  "category": "viewer",
  "fileExtensions": ["png", "jpg", "gif"],
  "entrypoint": "viewer.html",
  "enabled": true
}
```

---

## Column / Action / Panel Plugins

These are executables. Nyx spawns them as a subprocess and communicates via **newline-delimited JSON** on stdin/stdout.

The executable can be written in any language: Python, shell, Rust, Go, Node — anything available on the user's `PATH`.

### Protocol

Each request is one JSON line on stdin. Each response is one JSON line on stdout. Flush after every response.

**Request structure:**

```json
{"id": 1, "method": "get_columns", "params": { ... }}
```

**Response structure:**

```json
{"id": 1, "result": { ... }}
```

The `id` field must be echoed back — Nyx matches responses to requests by id.

### Column plugin: `get_columns`

**Request params:**

```json
{
  "files": [
    {
      "path": "/Users/me/project/main.rs",
      "name": "main.rs",
      "is_dir": false,
      "size": 4096,
      "modified": 1700000000,
      "extension": "rs"
    }
  ]
}
```

**Response:**

```json
{
  "result": [
    { "path": "/Users/me/project/main.rs", "value": "✓" }
  ]
}
```

Return one entry per file. Missing files are shown as blank. The `value` is a plain string — keep it short (fits in `columnWidth` pixels).

**Timeout:** 5 seconds per call. Slow operations will time out and show a blank value.

### Action plugin: `run_action`

**Request params:**

```json
{
  "files": ["/Users/me/a.txt", "/Users/me/b.txt"],
  "target_dir": "/Users/me/dest/"
}
```

**Response:**

```json
{
  "result": { "status": "ok", "message": "Compressed 2 files → archive.zip" }
}
```

The `message` string is displayed to the user after the action completes.

### Panel plugin: `list_dir`

**Request params:**

```json
{ "uri": "s3://my-bucket/prefix/" }
```

**Response:**

```json
{
  "result": {
    "entries": [
      {
        "name": "file.txt",
        "path": "s3://my-bucket/prefix/file.txt",
        "is_dir": false,
        "size": 1234,
        "modified": 1700000000
      }
    ]
  }
}
```

### Python template

```python
#!/usr/bin/env python3
import sys, json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    req = json.loads(line)
    method = req.get('method', '')
    params = req.get('params', {})

    if method == 'get_columns':
        results = []
        for f in params.get('files', []):
            value = compute_value(f['path'])   # your logic here
            results.append({'path': f['path'], 'value': value})
        print(json.dumps({'id': req['id'], 'result': results}), flush=True)

    else:
        print(json.dumps({'id': req['id'], 'error': f'unknown method: {method}'}), flush=True)
```

### Shell template

```bash
#!/usr/bin/env bash
while IFS= read -r line; do
  id=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
  method=$(echo "$line" | python3 -c "import sys,json; print(json.load(sys.stdin)['method'])")

  if [ "$method" = "get_columns" ]; then
    # ... compute and emit result ...
    echo "{\"id\": $id, \"result\": []}"
  fi
done
```

---

## Packaging as `.nyx-plugin`

A `.nyx-plugin` file is a standard ZIP archive with all plugin files at the **root level** (no wrapping subdirectory). The archive must contain `plugin.json` as its first entry.

```bash
cd ~/.nyx/plugins/com.example.my-plugin
zip -r ~/Desktop/com.example.my-plugin.nyx-plugin .
```

The `id` field in `plugin.json` determines the install directory name — make sure it's unique (reverse-domain format recommended: `com.yourname.plugin-name`).

---

## Debugging Tips

**Viewer plugins:** Open DevTools in any browser, load your `viewer.html` directly, and test `postMessage` in the console:

```javascript
window.dispatchEvent(new MessageEvent('message', {
  data: { type: 'load', fileUrl: 'file:///path/to/test.png', mimeType: 'image/png',
          theme: { bg: '#0a0e27', fg: '#e0e0f0', accent: '#00d4ff' } }
}));
```

**Column/action/panel plugins:** Run the executable directly in a terminal and pipe JSON to stdin:

```bash
echo '{"id":1,"method":"get_columns","params":{"files":[{"path":"/tmp/test.txt","name":"test.txt","is_dir":false,"size":100,"modified":0,"extension":"txt"}]}}' | python3 plugin.py
```

**Hot reload:** Edit `plugin.json` or replace a file in the plugin folder — Nyx detects the change and reloads the registry within a second. No restart needed.
