/**
 * Calculates bounds-aware position for menus to prevent them from appearing outside the viewport
 */
export function calculateBoundsAwarePosition(
	targetX: number,
	targetY: number,
	menuElement: HTMLElement
): { x: number; y: number } {
	const rect = menuElement.getBoundingClientRect();
	const viewportWidth = window.innerWidth;
	const viewportHeight = window.innerHeight;
	
	let x = targetX;
	let y = targetY;

	// Minimum offset from click point to avoid covering it
	const minOffset = 10;

	console.log('menuPositioning Debug:', {
		target: { x: targetX, y: targetY },
		menuSize: { width: rect.width, height: rect.height },
		viewport: { width: viewportWidth, height: viewportHeight },
		wouldExceedRight: x + rect.width > viewportWidth,
		wouldExceedBottom: y + rect.height > viewportHeight
	});

	// Check right boundary
	if (x + rect.width > viewportWidth) {
		x = targetX - rect.width - minOffset; // Position to the left of click
		console.log('Adjusted X to left:', x);
	} else {
		x = targetX + minOffset; // Default: slightly right of click
		console.log('Keeping X to right:', x);
	}

	// Check bottom boundary
	if (y + rect.height > viewportHeight) {
		y = targetY - rect.height - minOffset; // Position above click
		console.log('Adjusted Y to above:', y);
	} else {
		y = targetY + minOffset; // Default: slightly below click
		console.log('Keeping Y below:', y);
	}

	// Ensure we don't go off the left edge
	x = Math.max(5, x);
	
	// Ensure we don't go off the top edge
	y = Math.max(5, y);

	console.log('Final position:', { x, y });

	return { x, y };
}
