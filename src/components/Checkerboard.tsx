import { CSSProperties } from "react";

/** A subtle checkerboard pattern that reads as "transparent" without
 * fighting for attention with the image sitting on top of it. Tile size
 * and contrast are tuned down from the harsh default Photoshop look. */
export function checkerboardStyle(): CSSProperties {
  const light = "#232323";
  const dark = "#1a1a1a";
  const tile = 16;
  return {
    backgroundImage: `
      linear-gradient(45deg, ${dark} 25%, transparent 25%),
      linear-gradient(-45deg, ${dark} 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, ${dark} 75%),
      linear-gradient(-45deg, transparent 75%, ${dark} 75%)
    `,
    backgroundSize: `${tile}px ${tile}px`,
    backgroundPosition: `0 0, 0 ${tile / 2}px, ${tile / 2}px -${tile / 2}px, -${tile / 2}px 0`,
    backgroundColor: light,
  };
}
