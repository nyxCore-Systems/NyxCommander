export interface AtlasFrame {
  frame: { x: number; y: number; w: number; h: number };
  rotated: boolean;
  trimmed: boolean;
  spriteSourceSize: { x: number; y: number; w: number; h: number };
  sourceSize: { w: number; h: number };
}

export interface AtlasData {
  frames: Record<string, AtlasFrame>;
  meta: {
    image: string;
    format: string;
    size: { w: number; h: number };
    scale: string;
  };
}

let _atlas: AtlasData | null = null;

export async function loadAtlas(): Promise<AtlasData> {
  if (_atlas) return _atlas;
  const res = await fetch('/ui_atlas.json');
  _atlas = await res.json();
  return _atlas!;
}

export function getFrameStyle(
  atlas: AtlasData,
  name: string,
  scale: number = 1
): string {
  const frame = atlas.frames[name];
  if (!frame) return 'display: none';
  const { x, y, w, h } = frame.frame;
  const sw = w * scale;
  const sh = h * scale;
  const atlasW = atlas.meta.size.w * scale;
  const atlasH = atlas.meta.size.h * scale;
  return [
    `width: ${sw}px`,
    `height: ${sh}px`,
    `background-image: url('/ui_atlas.png')`,
    `background-position: ${-x * scale}px ${-y * scale}px`,
    `background-repeat: no-repeat`,
    `background-size: ${atlasW}px ${atlasH}px`,
    `image-rendering: pixelated`,
    `image-rendering: crisp-edges`,
    `display: inline-block`,
    `flex-shrink: 0`,
  ].join('; ');
}
