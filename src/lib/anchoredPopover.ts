export type PopoverRectangle = {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
};

export type AnchoredPopoverPlacement = {
  left: number;
  top: number;
  width: number;
  maxHeight: number;
};

export function placeAnchoredPopover({
  anchor,
  boundary,
  contentHeight,
  contentBoundaryWidth,
  gap = 5,
  margin = 8,
  maxWidth = 360,
  maxHeight = 260,
}: {
  anchor: PopoverRectangle;
  boundary: PopoverRectangle;
  contentHeight: number;
  contentBoundaryWidth: number;
  gap?: number;
  margin?: number;
  maxWidth?: number;
  maxHeight?: number;
}): AnchoredPopoverPlacement {
  const availableWidth = Math.max(
    1,
    Math.min(boundary.width, contentBoundaryWidth) - margin * 2,
  );
  const width = Math.min(maxWidth, availableWidth);
  const minViewportLeft = boundary.left + margin;
  const maxViewportLeft = boundary.right - margin - width;
  const viewportLeft = Math.min(
    Math.max(anchor.left, minViewportLeft),
    maxViewportLeft,
  );

  const desiredHeight = Math.min(contentHeight, maxHeight);
  const spaceBelow = boundary.bottom - anchor.bottom - gap - margin;
  const spaceAbove = anchor.top - boundary.top - gap - margin;
  const placeBelow = spaceBelow >= desiredHeight || spaceBelow >= spaceAbove;
  const availableHeight = Math.max(1, placeBelow ? spaceBelow : spaceAbove);
  const constrainedHeight = Math.min(maxHeight, availableHeight);
  const renderedHeight = Math.min(desiredHeight, constrainedHeight);

  return {
    left: viewportLeft - anchor.left,
    top: placeBelow ? anchor.height + gap : -renderedHeight - gap,
    width,
    maxHeight: constrainedHeight,
  };
}
