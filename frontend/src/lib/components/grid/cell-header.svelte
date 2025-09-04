<script lang="ts">
	import clsx from 'clsx';

	let {
		width = '80px',
		height = '30px',
		setColWidth = () => {},
		setRowHeight = () => {},
		val,
		active,
		direction = 'col' // New prop: 'col' for right-side drag, 'row' for bottom-side
	}: {
		width?: string;
		height?: string;
		setColWidth?: (width: string) => void;
		setRowHeight?: (height: string) => void;
		val: string;
		active: boolean;
		direction?: 'col' | 'row';
	} = $props();

	// --- Drag Logic ---
	const handleMouseDown = (startEvent: MouseEvent) => {
		// Prevent text selection during drag
		startEvent.preventDefault();

		const target = startEvent.currentTarget as HTMLElement;
		const parent = target.parentElement!;

		// Store the initial position and size
		const startX = startEvent.clientX;
		const startY = startEvent.clientY;
		const startWidth = parent.offsetWidth;
		const startHeight = parent.offsetHeight;

		const handleMouseMove = (moveEvent: MouseEvent) => {
			if (direction === 'col') {
				const dx = moveEvent.clientX - startX;
				// Enforce a minimum width of 40px
				setColWidth(`${Math.max(40, startWidth + dx)}px`);
			} else {
				const dy = moveEvent.clientY - startY;
				// Enforce a minimum height of 20px
				setRowHeight(`${Math.max(30, startHeight + dy)}px`);
			}
		};

		const handleMouseUp = () => {
			// Cleanup: remove the global listeners
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
		};

		// Add global listeners to track mouse movement anywhere on the page
		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
	};
</script>

<div
	style:width
	style:height
	class={clsx('placeholder group relative bg-background p-1 dark:bg-input/30', { active })}
>
	<span class="pointer-events-none flex h-full w-full items-center justify-center select-none">
		{val}
	</span>

	<div
		role="separator"
		aria-label="Resize handle"
		onmousedown={handleMouseDown}
		class={clsx('resizer', {
			'resizer-col': direction === 'col',
			'resizer-row': direction === 'row'
		})}
	/>
</div>

<style>
	.placeholder {
		font-size: 14px;
		border: 1px solid var(--input);
		overflow: hidden;
	}

	.active {
		border: 1px solid var(--color-primary);
		background-color: color-mix(in oklab, var(--color-primary) 40%, transparent);
	}

	/* --- Resizer Styles --- */
	.resizer {
		position: absolute;
		/* Make it easier to grab */
		z-index: 10;
		/* Subtle visual cue, becomes more visible on hover */
		background-color: transparent;
		transition: background-color 0.2s ease-in-out;
	}

	/* Style for vertical (column) resizing */
	.resizer-col {
		cursor: col-resize;
		top: 0;
		right: 0;
		width: 8px; /* Larger grab area */
		height: 100%;
	}

	/* Style for horizontal (row) resizing */
	.resizer-row {
		cursor: row-resize;
		bottom: 0;
		left: 0;
		height: 8px; /* Larger grab area */
		width: 100%;
	}

	/* Make the handle visible when hovering over the component */
	.group:hover > .resizer {
		background-color: var(--color-primary);
		opacity: 0.5;
	}
</style>
