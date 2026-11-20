// Top-view diagrams of mice whose buttons PeriphOS can assign. Original
// schematic drawings (not manufacturer artwork). Coordinates are in a
// 1000 × 620 view box; the mouse points up, its left side is the thumb side.

export interface DiagramButton {
  /** Button index as reported by the driver. */
  index: number;
  /** Where the leader line ends on the mouse. */
  anchor: [number, number];
  side: "left" | "right";
  /** Vertical position of the label card (view-box units, card centre). */
  cardY: number;
}

export interface MouseDiagram {
  /** Mouse outline. */
  body: string;
  /** Recessed areas, drawn darker (the wheel channel). */
  wells: string[];
  /** Seams and details drawn as thin lines. */
  lines: string[];
  /** Physical buttons drawn on the body, highlighted with their label. */
  parts: { index: number; shape: string }[];
  buttons: DiagramButton[];
}

export const VIEW_WIDTH = 1000;
export const VIEW_HEIGHT = 620;
/** Width of a label card column, in view-box units. */
export const CARD_WIDTH = 270;

const g502x: MouseDiagram = {
  body: `M 440 70 C 455 52 485 48 498 50 L 502 50 C 520 48 556 52 576 66
    C 600 90 612 150 612 230 C 612 300 604 350 608 410 C 614 480 606 560 560 588
    C 530 606 470 608 440 590 C 410 572 404 540 396 510 C 386 480 360 470 348 440
    C 338 412 344 380 366 368 C 384 356 392 330 392 290 C 392 220 398 130 418 92
    C 424 82 430 74 440 70 Z`,
  wells: ["M 486 82 h 28 a 6 6 0 0 1 6 6 v 84 a 6 6 0 0 1 -6 6 h -28 a 6 6 0 0 1 -6 -6 v -84 a 6 6 0 0 1 6 -6 Z"],
  lines: [
    // Thumb rest edge.
    "M 368 368 C 392 400 402 450 400 508",
    // Logo plate.
    "M 476 460 a 24 24 0 1 0 48 0 a 24 24 0 1 0 -48 0",
  ],
  parts: [
    // Main buttons.
    { index: 0, shape: "M 442 74 C 456 58 474 54 480 54 L 480 300 C 450 304 418 300 396 294 C 396 220 402 130 422 96 C 428 86 434 78 442 74 Z" },
    { index: 1, shape: "M 520 54 C 540 54 558 58 572 70 C 596 94 606 150 606 230 L 605 292 C 580 300 548 302 520 300 Z" },
    // Wheel (click and tilt share it).
    { index: 2, shape: "M 491 92 h 18 a 4 4 0 0 1 4 4 v 68 a 4 4 0 0 1 -4 4 h -18 a 4 4 0 0 1 -4 -4 v -68 a 4 4 0 0 1 4 -4 Z" },
    // Button below the wheel.
    { index: 8, shape: "M 492 204 h 16 a 5 5 0 0 1 5 5 v 26 a 5 5 0 0 1 -5 5 h -16 a 5 5 0 0 1 -5 -5 v -26 a 5 5 0 0 1 5 -5 Z" },
    // DPI buttons beside the left button.
    { index: 9, shape: "M 420 118 c 6 -3 12 0 12 6 v 24 c 0 6 -6 8 -12 5 c -3 -2 -4 -4 -4 -7 v -21 c 0 -3 2 -6 4 -7 Z" },
    { index: 10, shape: "M 414 162 c 6 -3 12 0 12 6 v 24 c 0 6 -6 8 -12 5 c -3 -2 -4 -4 -4 -7 v -21 c 0 -3 2 -6 4 -7 Z" },
    // Forward and back on the thumb side, sniper at the front of the thumb rest.
    { index: 5, shape: "M 384 246 c 6 -2 11 1 11 6 v 30 c 0 5 -5 7 -10 5 c -3 -1 -5 -4 -5 -7 v -27 c 0 -3 2 -6 4 -7 Z" },
    { index: 3, shape: "M 384 296 c 6 -2 11 1 11 6 v 30 c 0 5 -5 7 -10 5 c -3 -1 -5 -4 -5 -7 v -27 c 0 -3 2 -6 4 -7 Z" },
    { index: 4, shape: "M 351 392 a 13 13 0 1 0 26 0 a 13 13 0 1 0 -26 0" },
  ],
  buttons: [
    { index: 6, anchor: [487, 128], side: "left", cardY: 70 },
    { index: 9, anchor: [424, 135], side: "left", cardY: 150 },
    { index: 10, anchor: [418, 179], side: "left", cardY: 230 },
    { index: 0, anchor: [450, 230], side: "left", cardY: 310 },
    { index: 5, anchor: [389, 268], side: "left", cardY: 390 },
    { index: 3, anchor: [389, 318], side: "left", cardY: 470 },
    { index: 4, anchor: [364, 392], side: "left", cardY: 550 },
    { index: 2, anchor: [500, 100], side: "right", cardY: 90 },
    { index: 7, anchor: [513, 128], side: "right", cardY: 190 },
    { index: 1, anchor: [565, 230], side: "right", cardY: 290 },
    { index: 8, anchor: [500, 222], side: "right", cardY: 390 },
  ],
};

/** The diagram for a device, when one exists for its model and button count. */
export function diagramFor(deviceName: string, buttonCount: number): MouseDiagram | null {
  if (/\bG502 X\b/i.test(deviceName) && buttonCount === 11) return g502x;
  return null;
}
