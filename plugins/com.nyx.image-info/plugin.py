#!/usr/bin/env python3
"""
Nyx.Commander column plugin: Image Info
Shows pixel dimensions (W×H) for image files by reading their binary headers.

Supports: PNG, JPEG, GIF87a/89a, BMP, WebP (VP8/VP8L/VP8X), ICO, TIFF
No external dependencies — only stdlib `struct`.

Protocol (newline-delimited JSON-RPC on stdin/stdout):
  In:  {"id": N, "method": "get_columns", "params": {"files": [...]}}
  Out: {"id": N, "result": [{"path": "...", "value": "1920×1080"}]}
"""

import sys
import json
import struct
import os


# How many bytes to read for header parsing (enough for any format below)
HEADER_BYTES = 512


def read_header(path: str) -> bytes:
    try:
        with open(path, 'rb') as f:
            return f.read(HEADER_BYTES)
    except OSError:
        return b''


def dims_png(h: bytes):
    # PNG: 8-byte sig + 4-byte length + 4-byte "IHDR" + 4W + 4H
    if len(h) < 24 or h[:8] != b'\x89PNG\r\n\x1a\n':
        return None
    w, ht = struct.unpack('>II', h[16:24])
    return w, ht


def dims_jpeg(h: bytes):
    # JPEG: scan for SOF markers (0xFF C0..C3, C5..C7, C9..CB, CD..CF)
    SOF = {0xC0, 0xC1, 0xC2, 0xC3, 0xC5, 0xC6, 0xC7,
           0xC9, 0xCA, 0xCB, 0xCD, 0xCE, 0xCF}
    if len(h) < 4 or h[0] != 0xFF or h[1] != 0xD8:
        return None
    i = 2
    while i + 3 < len(h):
        if h[i] != 0xFF:
            break
        marker = h[i + 1]
        if marker in SOF and i + 9 < len(h):
            ht, w = struct.unpack('>HH', h[i + 5:i + 9])
            return w, ht
        if i + 3 < len(h):
            length = struct.unpack('>H', h[i + 2:i + 4])[0]
            i += 2 + length
        else:
            break
    return None


def dims_gif(h: bytes):
    # GIF: 6-byte sig ("GIF87a" or "GIF89a") + 2W + 2H (little-endian)
    if len(h) < 10 or h[:3] != b'GIF':
        return None
    w, ht = struct.unpack('<HH', h[6:10])
    return w, ht


def dims_bmp(h: bytes):
    # BMP: 2-byte "BM" + 4 file size + 4 reserved + 4 offset + 4 header size + 4W + 4H
    if len(h) < 26 or h[:2] != b'BM':
        return None
    w, ht = struct.unpack('<ii', h[18:26])
    return abs(w), abs(ht)   # height may be negative (top-down)


def dims_webp(h: bytes):
    # RIFF....WEBP + chunk type
    if len(h) < 30 or h[:4] != b'RIFF' or h[8:12] != b'WEBP':
        return None
    chunk = h[12:16]
    if chunk == b'VP8 ' and len(h) >= 30:
        # Lossy: skip 10 bytes of VP8 bitstream header, then 14-bit W and H
        if h[23:26] == b'\x9d\x01\x2a' and len(h) >= 30:
            raw_w, raw_h = struct.unpack('<HH', h[26:30])
            return raw_w & 0x3FFF, raw_h & 0x3FFF
    elif chunk == b'VP8L' and len(h) >= 25:
        # Lossless: 1 signature byte then packed 14-bit W-1 and H-1
        bits = struct.unpack('<I', h[21:25])[0]
        w = (bits & 0x3FFF) + 1
        ht = ((bits >> 14) & 0x3FFF) + 1
        return w, ht
    elif chunk == b'VP8X' and len(h) >= 30:
        # Extended: 24-bit W-1 and H-1 at bytes 24 and 27
        w = struct.unpack('<I', h[24:27] + b'\x00')[0] + 1
        ht = struct.unpack('<I', h[27:30] + b'\x00')[0] + 1
        return w, ht
    return None


def dims_ico(h: bytes):
    # ICO: 6-byte header + per-image 16-byte entries. First entry at byte 6.
    # Bytes 6-7: width, height (0 = 256)
    if len(h) < 8 or h[:4] != b'\x00\x00\x01\x00':
        return None
    w = h[6] or 256
    ht = h[7] or 256
    return w, ht


def dims_tiff(h: bytes):
    # TIFF: 2-byte byte order ("II" LE or "MM" BE) + 2-byte magic (42)
    if len(h) < 8:
        return None
    if h[:2] == b'II':
        endian = '<'
    elif h[:2] == b'MM':
        endian = '>'
    else:
        return None
    magic = struct.unpack(endian + 'H', h[2:4])[0]
    if magic != 42:
        return None
    # IFD offset
    ifd_off = struct.unpack(endian + 'I', h[4:8])[0]
    # We only have HEADER_BYTES — if IFD is beyond that we can't read it
    if ifd_off + 2 > len(h):
        return None
    count = struct.unpack(endian + 'H', h[ifd_off:ifd_off + 2])[0]
    w = ht = None
    pos = ifd_off + 2
    for _ in range(count):
        if pos + 12 > len(h):
            break
        tag, typ, cnt = struct.unpack(endian + 'HHI', h[pos:pos + 8])
        val_off = pos + 8
        # Tag 256 = ImageWidth, 257 = ImageLength
        if tag in (256, 257):
            if typ == 3:   # SHORT
                val = struct.unpack(endian + 'H', h[val_off:val_off + 2])[0]
            elif typ == 4:  # LONG
                val = struct.unpack(endian + 'I', h[val_off:val_off + 4])[0]
            else:
                val = None
            if val is not None:
                if tag == 256:
                    w = val
                else:
                    ht = val
        pos += 12
        if w is not None and ht is not None:
            break
    if w and ht:
        return w, ht
    return None


PARSERS = [dims_png, dims_jpeg, dims_gif, dims_bmp, dims_webp, dims_ico, dims_tiff]


def image_dims(path: str) -> str:
    h = read_header(path)
    if not h:
        return ''
    for parser in PARSERS:
        result = parser(h)
        if result:
            w, ht = result
            return f'{w}\u00d7{ht}'   # "W×H"
    return ''


def handle_get_columns(params: dict) -> list:
    files = params.get('files', [])
    results = []
    for f in files:
        path = f.get('path', '')
        is_dir = f.get('is_dir', False)
        ext = f.get('extension', '').lower()

        if not path or is_dir:
            results.append({'path': path, 'value': ''})
            continue

        # Only attempt parsing for known image extensions
        IMAGE_EXTS = {'png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'ico', 'tiff', 'tif'}
        if ext not in IMAGE_EXTS:
            results.append({'path': path, 'value': ''})
            continue

        results.append({'path': path, 'value': image_dims(path)})
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
