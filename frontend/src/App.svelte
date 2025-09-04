<script lang="ts">
  import { onMount } from "svelte";

  // --- Configuration ---
  const numRows = 1000;
  const numCols = 100;
  const rowHeight = 30; // px
  const colWidth = 150; // px
  const rowNumberWidth = 50; // px
  const colHeaderHeight = 30; // px

  // --- State (Svelte 5 runes) ---
  let gridData = $state<string[][] | null>(new Array());
  let columnLabels = $state<string[]>([]);
  let activeCell = $state<[number, number] | null>(null);

  let viewportEl = $state<HTMLElement | null>(null);
  let viewportWidth = $state(0);
  let viewportHeight = $state(0);
  let scrollTop = $state(0);
  let scrollLeft = $state(0);

  let socket: WebSocket | null = null;

  // --- Helpers ---
  function generateColumnLabels(count: number): string[] {
    const labels: string[] = [];
    for (let i = 0; i < count; i++) {
      let label = "";
      let temp = i;
      while (temp >= 0) {
        label = String.fromCharCode((temp % 26) + 65) + label;
        temp = Math.floor(temp / 26) - 1;
      }
      labels.push(label);
    }
    return labels;
  }

  onMount(() => {
    columnLabels = generateColumnLabels(numCols);
    gridData = Array.from({ length: numRows }, () =>
      Array<string>(numCols).fill(""),
    );

    socket = new WebSocket("ws://localhost:7050");
    socket.onmessage = (e) =>
      console.log("Received message from server:", e.data);
    socket.onopen = () => console.log("WebSocket connection established.");
    return () => socket?.close();
  });

  // --- Events ---
  function handleGridBlur(e: FocusEvent) {
    const target = e.currentTarget as HTMLElement;
    const next = e.relatedTarget as Node | null;
    if (!target.contains(next)) activeCell = null;
  }

  function handleCellBlur(row: number, col: number) {
    console.log(
      `Cell (${row + 1}, ${columnLabels[col]}) lost focus. Value:`,
      gridData![row][col],
    );
    socket?.send(
      JSON.stringify({
        row: row + 1,
        col: columnLabels[col],
        value: gridData![row][col],
      }),
    );
  }

  // --- Scroll math (account for header/row gutters) ---
  // How far into the *grid area* (excluding headers) we've scrolled:
  const scrollDX = $derived(Math.max(0, scrollLeft - rowNumberWidth));
  const scrollDY = $derived(Math.max(0, scrollTop - colHeaderHeight));

  // Viewport pixels actually available to show cells (excluding sticky gutters):
  const innerW = $derived(Math.max(0, viewportWidth - rowNumberWidth));
  const innerH = $derived(Math.max(0, viewportHeight - colHeaderHeight));

  // Virtualization windows:
  const startCol = $derived(Math.max(0, Math.floor(scrollDX / colWidth)));
  const endCol = $derived(
    Math.min(numCols, startCol + Math.ceil(innerW / colWidth) + 1),
  );

  const startRow = $derived(Math.max(0, Math.floor(scrollDY / rowHeight)));
  const endRow = $derived(
    Math.min(numRows, startRow + Math.ceil(innerH / rowHeight) + 1),
  );

  const visibleCols = $derived(
    Array.from({ length: endCol - startCol }, (_, i) => startCol + i),
  );
  const visibleRows = $derived(
    Array.from({ length: endRow - startRow }, (_, i) => startRow + i),
  );
</script>

<div
  class="w-full h-[85vh] p-4 bg-background text-foreground rounded-lg border flex flex-col"
>
  <div
    class="grid-container relative"
    on:focusout={handleGridBlur}
    style="
      --row-height: {rowHeight}px;
      --col-width: {colWidth}px;
      --row-number-width: {rowNumberWidth}px;
      --col-header-height: {colHeaderHeight}px;
    "
  >
    <!-- Single, real scroll container -->
    <div
      class="viewport"
      bind:this={viewportEl}
      bind:clientWidth={viewportWidth}
      bind:clientHeight={viewportHeight}
      on:scroll={(e) => {
        const el = e.currentTarget as HTMLElement;
        scrollTop = el.scrollTop;
        scrollLeft = el.scrollLeft;
      }}
    >
      <!-- Sizer creates true scrollable area (includes gutters + grid) -->
      <div
        class="total-sizer"
        style:width={`${rowNumberWidth + numCols * colWidth}px`}
        style:height={`${colHeaderHeight + numRows * rowHeight}px`}
      />

      <!-- Top-left sticky corner -->
      <div
        class="top-left-corner"
        style="top: 0; left: 0;"
        class:active-header-corner={activeCell !== null}
      />

      <!-- Column headers (stick to top, scroll horizontally with grid) -->
      <div
        class="col-headers-container"
        style="
          top: 0;
          left: {rowNumberWidth}px;
          transform: translateX(-{scrollDX}px);
        "
      >
        {#each visibleCols as j (j)}
          <div
            class="header-cell"
            style:left={`${j * colWidth}px`}
            class:active-header={activeCell !== null && activeCell[1] === j}
          >
            {columnLabels[j]}
          </div>
        {/each}
      </div>

      <!-- Row headers (stick to left, scroll vertically with grid) -->
      <div
        class="row-headers-container"
        style="
          top: {colHeaderHeight}px;
          left: 0;
          transform: translateY(-{scrollDY}px);
        "
      >
        {#each visibleRows as i (i)}
          <div
            class="row-number-cell"
            style:top={`${i * rowHeight}px`}
            class:active-header={activeCell !== null && activeCell[0] === i}
          >
            {i + 1}
          </div>
        {/each}
      </div>

      <!-- Visible grid cells (overlay; move opposite the inner scroll) -->
      {#if gridData}
        <div
          class="cells-container"
          style="
            top: {colHeaderHeight}px;
            left: {rowNumberWidth}px;
            transform: translate(-{scrollDX}px, -{scrollDY}px);
          "
        >
          {#each visibleRows as i (i)}
            {#each visibleCols as j (j)}
              <div
                class="grid-cell"
                style:top={`${i * rowHeight}px`}
                style:left={`${j * colWidth}px`}
              >
                <input
                  type="text"
                  value={gridData[i][j]}
                  class="cell-input"
                  on:input={(e) =>
                    (gridData[i][j] = (
                      e.currentTarget as HTMLInputElement
                    ).value)}
                  on:focus={() => (activeCell = [i, j])}
                  on:blur={() => handleCellBlur(i, j)}
                />
                {#if activeCell && activeCell[0] === i && activeCell[1] === j}
                  <div class="active-cell-indicator">
                    <div class="fill-handle" />
                  </div>
                {/if}
              </div>
            {/each}
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .grid-container {
    flex-grow: 1;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      Helvetica, Arial, sans-serif;
    border: 1px solid hsl(var(--border));
  }

  .viewport {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: auto;
  }

  .total-sizer {
    /* occupies the scroll area; other layers are absolutely positioned on top */
    width: 100%;
    height: 100%;
  }

  /* Layered overlays inside the viewport */
  .top-left-corner,
  .col-headers-container,
  .row-headers-container,
  .cells-container {
    position: absolute;
    z-index: 1;
  }

  .top-left-corner {
    width: var(--row-number-width);
    height: var(--col-header-height);
    background-color: hsl(var(--muted));
    border-right: 1px solid hsl(var(--border));
    border-bottom: 1px solid hsl(var(--border));
    z-index: 4;
  }

  .active-header-corner {
    background-color: hsl(var(--accent));
  }

  .col-headers-container {
    height: var(--col-header-height);
    z-index: 3;
  }

  .row-headers-container {
    width: var(--row-number-width);
    z-index: 2;
  }

  .cells-container {
    z-index: 1;
  }

  .header-cell,
  .row-number-cell {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: hsl(var(--muted));
    font-weight: 500;
    font-size: 0.8rem;
    color: hsl(var(--muted-foreground));
    user-select: none;
    border-right: 1px solid hsl(var(--border));
    border-bottom: 1px solid hsl(var(--border));
    transition: background-color 100ms;
  }

  .header-cell {
    height: var(--col-header-height);
    width: var(--col-width);
  }
  .row-number-cell {
    width: var(--row-number-width);
    height: var(--row-height);
  }

  .active-header {
    background-color: hsl(var(--accent) / 0.8);
    color: hsl(var(--accent-foreground));
  }

  .grid-cell {
    position: absolute;
    width: var(--col-width);
    height: var(--row-height);
    border-right: 1px solid hsl(var(--border) / 0.7);
    border-bottom: 1px solid hsl(var(--border) / 0.7);
    background-color: hsl(var(--background));
  }

  .cell-input {
    appearance: none;
    -webkit-appearance: none;
    -moz-appearance: none;
    width: 100%;
    height: 100%;
    padding: 0 0.5rem;
    margin: 0;
    background-color: transparent;
    border-radius: 0;
    border: 1px solid black;
    font-size: 0.875rem;
    font-family: inherit;
    text-align: left;
    outline: none;
    box-shadow: none;
  }

  .cell-input:focus,
  .cell-input:focus-visible {
    border: 1px solid red;
  }

  .active-cell-indicator {
    position: absolute;
    top: -1px;
    left: -1px;
    width: calc(100% + 2px);
    height: calc(100% + 2px);
    border: 2px solid hsl(var(--primary));
    pointer-events: none;
    z-index: 5;
  }

  .fill-handle {
    position: absolute;
    bottom: -4px;
    right: -4px;
    width: 6px;
    height: 6px;
    background: hsl(var(--primary));
    border: 1px solid hsl(var(--primary-foreground));
    cursor: crosshair;
  }
</style>
