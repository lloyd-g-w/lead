<script lang="ts">
	import clsx from 'clsx';

	let {
		width = '80px',
		height = '30px',
		setColWidth = () => {},
		setRowHeight = () => {},
		val,
		active,
		direction = 'col', // New prop: 'col' for right-side drag, 'row' for bottom-side
		resizeable = true
	}: {
		width?: string;
		height?: string;
		setColWidth?: (width: string) => void;
		setRowHeight?: (height: string) => void;
		val: string;
		active: boolean;
		resizeable?: boolean;
		direction?: 'col' | 'row' | 'blank';
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
	class={clsx('placeholder group relative bg-background p-1', {
		active,
		col: direction === 'col',
		row: direction === 'row',
		blank: direction === 'blank'
	})}
>
	<span class="pointer-events-none flex h-full w-full items-center justify-center select-none">
		{val}
	</span>

	{#if resizeable}
		<div
			role="separator"
			aria-label="Resize handle"
			onmousedown={handleMouseDown}
			class={clsx('resizer', {
				'resizer-col': direction === 'col',
				'resizer-row': direction === 'row'
			})}
		></div>
	{/if}
</div>

<style>
	.placeholder {
		font-size: 14px;
		border: 1px solid var(--input);
		background-color: var(--color-background);
	}

	.placeholder.blank {
		border: 1px solid var(--input);
	}

	.placeholder.blank,
	.placeholder.row {
		border-left: none;
	}

	.placeholder.blank,
	.placeholder.col {
		border-top: none;
	}

	.active {
		background-color: color-mix(in oklab, var(--color-primary) 80%, var(--color-background) 80%);
		font-weight: bold;
		/* border: 1px solid var(--color-primary); */
	}

	/* --- Resizer Styles --- */
	.resizer {
		position: absolute;
		/* Subtle visual cue, becomes more visible on hover */
		background-color: var(--color-primary);
		opacity: 0;
		transition: opacity 0.1s ease-in-out;
		z-index: 60;
	}

	/* Style for vertical (column) resizing */
	.resizer-col {
		cursor: col-resize;
		top: 0;
		right: -5px;
		width: 9px; /* Larger grab area */
		height: 100%;
	}

	.resizer-row {
		cursor: row-resize;
		bottom: -5px;
		left: 0;
		height: 9px; /* Larger grab area */
		width: 100%;
	}

	/* Make the handle visible when hovering over the component */
	.resizer:hover,
	.group:hover > .resizer {
		opacity: 0.5;
	}
</style>
