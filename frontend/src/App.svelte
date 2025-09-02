<script lang="ts">
  import { Input } from "$lib/components/ui/input";
  import { onMount } from "svelte";

  // --- Configuration ---
  const numRows = 1000; // Increased to show performance
  const numCols = 100; // Increased to show performance
  const rowHeight = 30; // px
  const colWidth = 150; // px
  const rowNumberWidth = 50; // px
  const colHeaderHeight = 30; // px

  // --- State ---
  let gridData: string[][];
  let columnLabels: string[] = [];
  let activeCell: [number, number] | null = null;
  let viewportElement: HTMLElement;
  let viewportWidth = 0;
  let viewportHeight = 0;
  let scrollTop = 0;
  let scrollLeft = 0;

  // --- Helper Functions ---
  /**
   * Generates Excel-style column labels (A, B, ..., Z, AA, AB, ...).
   */
  function generateColumnLabels(count: number): string[] {
    const labels = [];
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
    gridData = Array(numRows)
      .fill(null)
      .map(() => Array(numCols).fill(""));
  });

  // --- Event Handlers ---
  function handleGridBlur(e: FocusEvent) {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
      activeCell = null;
    }
  }

  // --- Reactive Calculations for Virtualization ---
  // $: is a Svelte feature that re-runs code when its dependencies change.

  // Calculate which rows/cols are visible based on scroll position
  $: startRow = Math.max(0, Math.floor(scrollTop / rowHeight));
  $: endRow = Math.min(
    numRows,
    startRow + Math.ceil(viewportHeight / rowHeight) + 1,
  );
  $: startCol = Math.max(0, Math.floor(scrollLeft / colWidth));
  $: endCol = Math.min(
    numCols,
    startCol + Math.ceil(viewportWidth / colWidth) + 1,
  );

  // Create arrays of only the visible items to render
  $: visibleRows = Array(endRow - startRow)
    .fill(0)
    .map((_, i) => startRow + i);
  $: visibleCols = Array(endCol - startCol)
    .fill(0)
    .map((_, i) => startCol + i);
</script>

<div
  class="w-full h-[85vh] p-4 bg-background text-foreground rounded-lg border flex flex-col"
>
  <div
    class="relative grid-container"
    on:focusout={handleGridBlur}
    style="
        --row-height: {rowHeight}px;
        --col-width: {colWidth}px;
        --row-number-width: {rowNumberWidth}px;
        --col-header-height: {colHeaderHeight}px;
    "
  >
    <!-- The scrollable viewport -->
    <div
      class="viewport"
      bind:this={viewportElement}
      bind:clientWidth={viewportWidth}
      bind:clientHeight={viewportHeight}
      on:scroll={(e) => {
        scrollTop = e.currentTarget.scrollTop;
        scrollLeft = e.currentTarget.scrollLeft;
      }}
    >
      <!-- Sizer div: creates the full scrollable area -->
      <div
        class="total-sizer"
        style:width="{numCols * colWidth}px"
        style:height="{numRows * rowHeight}px"
      />

      {#if gridData}
        <!-- Render Window: contains only the visible cells -->
        <div
          class="render-window"
          style:transform="translate({scrollLeft}px, {scrollTop}px)"
        >
          <!-- Top-left corner -->
          <div
            class="top-left-corner"
            class:active-header-corner={activeCell !== null}
          />

          <!-- Visible Column Headers -->
          <div
            class="col-headers-container"
            style:transform="translateX(-{scrollLeft % colWidth}px)"
          >
            {#each visibleCols as j (j)}
              <div
                class="header-cell"
                style:left="{j * colWidth}px"
                class:active-header={activeCell !== null && activeCell[1] === j}
              >
                {columnLabels[j]}
              </div>
            {/each}
          </div>

          <!-- Visible Row Headers -->
          <div
            class="row-headers-container"
            style:transform="translateY(-{scrollTop % rowHeight}px)"
          >
            {#each visibleRows as i (i)}
              <div
                class="row-number-cell"
                style:top="{i * rowHeight}px"
                class:active-header={activeCell !== null && activeCell[0] === i}
              >
                {i + 1}
              </div>
            {/each}
          </div>

          <!-- Visible Grid Cells -->
          <div
            class="cells-container"
            style:transform="translate(-{scrollLeft % colWidth}px, -{scrollTop %
              rowHeight}px)"
          >
            {#each visibleRows as i (i)}
              {#each visibleCols as j (j)}
                <div
                  class="grid-cell"
                  style:top="{i * rowHeight}px"
                  style:left="{j * colWidth}px"
                >
                  <Input
                    type="text"
                    bind:value={gridData[i][j]}
                    class="cell-input"
                    on:focus={() => (activeCell = [i, j])}
                  />
                  <!-- Active cell indicator with fill handle -->
                  {#if activeCell && activeCell[0] === i && activeCell[1] === j}
                    <div class="active-cell-indicator">
                      <div class="fill-handle" />
                    </div>
                  {/if}
                </div>
              {/each}
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .grid-container {
    flex-grow: 1;
    position: relative;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      Helvetica, Arial, sans-serif;
    border: 1px solid hsl(var(--border));
  }
  .viewport {
    width: 100%;
    height: 100%;
    overflow: auto;
    position: relative;
  }
  .total-sizer {
    position: relative;
    pointer-events: none;
  }
  .render-window {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    pointer-events: none; /* Pass clicks through to elements below */
  }

  .top-left-corner,
  .col-headers-container,
  .row-headers-container,
  .cells-container {
    position: absolute;
    top: 0;
    left: 0;
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
    top: 0;
    left: var(--row-number-width);
    height: var(--col-header-height);
    z-index: 3;
  }
  .row-headers-container {
    top: var(--col-header-height);
    left: 0;
    width: var(--row-number-width);
    z-index: 2;
  }
  .cells-container {
    top: var(--col-header-height);
    left: var(--row-number-width);
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
    pointer-events: auto;
  }

  .cell-input {
    width: 100%;
    height: 100%;
    padding: 0 0.5rem;
    background-color: transparent;
    border: none;
    border-radius: 0;
    font-size: 0.875rem;
    text-align: left;
    outline: none;
    box-shadow: none;
  }
  .cell-input:focus {
    box-shadow: none; /* Removes shadcn default focus ring */
  }

  .active-cell-indicator {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border: 2px solid hsl(var(--primary));
    pointer-events: none;
    z-index: 5;
  }
  .fill-handle {
    position: absolute;
    bottom: -3px;
    right: -3px;
    width: 5px;
    height: 5px;
    background: hsl(var(--primary));
    cursor: crosshair;
  }

</style>
