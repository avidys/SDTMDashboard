<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';

  export type Column<T = Record<string, unknown>> = {
    key: string;
    label: string;
    width?: string | number;
    cellClass?: string | ((row: T, col: string) => string);
    format?: (value: unknown, row: T) => string;
  };

  interface Props<T = Record<string, unknown>> {
    data: T[];
    columns: Column<T>[];
    rowHeight?: number;
    overscan?: number;
    class?: string;
  }

  let {
    data,
    columns,
    rowHeight = 40,
    overscan = 10,
    class: className = ''
  }: Props = $props();

  let containerRef: HTMLDivElement | null = null;
  let headerRef: HTMLDivElement | null = null;
  let rowVirtualizer: ReturnType<typeof createVirtualizer<HTMLDivElement, Element>> | null = null;
  let dataKey = $state('');
  let measuredWidths = $state<number[]>([]);

  // Calculate column widths based on content or provided widths
  let columnWidths = $derived.by(() => {
    return columns.map((col, index) => {
      // If width is explicitly provided, use it
      if (typeof col.width === 'number') {
        return `${col.width}px`;
      } else if (typeof col.width === 'string') {
        return col.width;
      }
      // Use measured width if available
      if (measuredWidths[index] && measuredWidths[index] > 0) {
        return `${measuredWidths[index]}px`;
      }
      // Default fallback
      return '150px';
    });
  });

  let gridTemplate = $derived(columnWidths.join(' '));
  let totalTableWidth = $derived(columnWidths.reduce((sum, w) => {
    const num = parseFloat(w);
    return sum + (isNaN(num) ? 150 : num);
  }, 0));

  // Helper to safely access virtualizer
  function getVirtualizer() {
    return rowVirtualizer ? $rowVirtualizer : null;
  }

  // Create a key to track when data/columns change significantly
  let currentDataKey = $derived(`${data.length}-${columns.length}-${columns.map(c => c.key).join(',')}`);

  // Initialize or recreate virtualizer when container is ready or data changes
  $effect(() => {
    if (containerRef && data.length > 0 && columns.length > 0) {
      // If data key changed significantly, recreate virtualizer
      if (dataKey !== currentDataKey) {
        rowVirtualizer = createVirtualizer({
          count: data.length,
          getScrollElement: () => containerRef,
          estimateSize: () => rowHeight,
          overscan
        });
        dataKey = currentDataKey;
      } else if (!rowVirtualizer) {
        // Initial creation
        rowVirtualizer = createVirtualizer({
          count: data.length,
          getScrollElement: () => containerRef,
          estimateSize: () => rowHeight,
          overscan
        });
        dataKey = currentDataKey;
      }
    }
  });

  // Update virtualizer when data length changes (but columns stay the same)
  $effect(() => {
    const v = getVirtualizer();
    if (v && data.length > 0) {
      v.setOptions({
        count: data.length
      });
    }
  });

  // Get virtual items and total size
  let virtualRows = $derived.by(() => {
    const v = getVirtualizer();
    if (!v) return [];
    const items = v.getVirtualItems();
    // Force a measure update if we have items but they're not visible
    if (items.length === 0 && data.length > 0 && containerRef) {
      // Use a microtask to allow the DOM to update
      queueMicrotask(() => {
        const v2 = getVirtualizer();
        if (v2) {
          v2.measure();
        }
      });
    }
    return items;
  });

  let totalSize = $derived.by(() => {
    const v = getVirtualizer();
    if (v) {
      const size = v.getTotalSize();
      return size > 0 ? size : data.length * rowHeight;
    }
    // Fallback: estimate size based on data length
    return data.length * rowHeight;
  });

  // Force measure when container becomes available
  $effect(() => {
    if (containerRef && rowVirtualizer) {
      // Use a small delay to ensure container has dimensions
      const timeoutId = setTimeout(() => {
        const v = getVirtualizer();
        if (v) {
          v.measure();
        }
      }, 0);
      return () => clearTimeout(timeoutId);
    }
  });

  function formatValue(value: unknown): string {
    if (value === null || value === undefined) return '—';
    if (typeof value === 'string') return value;
    if (typeof value === 'number') return value.toString();
    return String(value);
  }

  function getCellClass(row: Record<string, unknown>, column: Column): string {
    if (typeof column.cellClass === 'function') {
      return column.cellClass(row, column.key);
    }
    return column.cellClass || '';
  }

  function getCellValue(row: Record<string, unknown>, column: Column): string {
    const value = row[column.key];
    if (column.format) {
      return column.format(value, row);
    }
    return formatValue(value);
  }

  // Measure column widths based on content
  function measureColumnWidths() {
    if (!containerRef || data.length === 0 || columns.length === 0) return;

    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) return;

    // Use a reasonable font size matching our CSS
    context.font = '0.875rem system-ui, -apple-system, sans-serif';
    const headerFont = '600 0.875rem system-ui, -apple-system, sans-serif';
    // Padding: 0.75rem left + 0.75rem right = 1.5rem = 24px (assuming 16px base)
    const padding = 24;
    const minWidth = 80;

    const widths: number[] = [];

    for (let i = 0; i < columns.length; i++) {
      const col = columns[i];
      let maxWidth = 0;

      // Measure header
      context.font = headerFont;
      const headerWidth = context.measureText(col.label).width;
      maxWidth = Math.max(maxWidth, headerWidth);

      // Measure sample of data cells (first 100 rows or all if less)
      context.font = '0.875rem system-ui, -apple-system, sans-serif';
      const sampleSize = Math.min(100, data.length);
      for (let j = 0; j < sampleSize; j++) {
        const row = data[j];
        const value = getCellValue(row, col);
        const textWidth = context.measureText(value).width;
        maxWidth = Math.max(maxWidth, textWidth);
      }

      // Add padding and ensure minimum width
      widths[i] = Math.max(minWidth, maxWidth + padding);
    }

    measuredWidths = widths;
  }

  // Measure widths when data or columns change
  $effect(() => {
    if (data.length > 0 && columns.length > 0) {
      // Use a small delay to ensure DOM is ready
      const timeoutId = setTimeout(() => {
        measureColumnWidths();
      }, 0);
      return () => clearTimeout(timeoutId);
    }
  });

  // Synchronize horizontal scrolling between header and body
  let isScrolling = $state(false);

  function handleHeaderScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    if (!isScrolling && containerRef && Math.abs(target.scrollLeft - containerRef.scrollLeft) > 1) {
      isScrolling = true;
      containerRef.scrollLeft = target.scrollLeft;
      requestAnimationFrame(() => {
        isScrolling = false;
      });
    }
  }

  function handleBodyScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    if (!isScrolling && headerRef && Math.abs(target.scrollLeft - headerRef.scrollLeft) > 1) {
      isScrolling = true;
      headerRef.scrollLeft = target.scrollLeft;
      requestAnimationFrame(() => {
        isScrolling = false;
      });
    }
  }
</script>

<div class="virtualized-table-container {className}">
  <div 
    class="table-header-wrapper"
    bind:this={headerRef}
    onscroll={handleHeaderScroll}
  >
    <div class="table-header" style="grid-template-columns: {gridTemplate}; width: {totalTableWidth}px;">
      {#each columns as column}
        <div class="header-cell">{column.label}</div>
      {/each}
    </div>
  </div>
  <div
    class="table-body"
    bind:this={containerRef}
    onscroll={handleBodyScroll}
    role="grid"
    aria-rowcount={data.length}
    aria-colcount={columns.length}
  >
    <div class="virtual-spacer" style="height: {totalSize}px; width: {totalTableWidth}px;">
      {#if rowVirtualizer && virtualRows.length > 0}
        {#each virtualRows as virtualRow (virtualRow.key)}
          {@const row = data[virtualRow.index]}
          {#if row}
            <div
              class="virtual-row"
              class:odd={virtualRow.index % 2 === 1}
              style="grid-template-columns: {gridTemplate}; transform: translateY({virtualRow.start}px); width: {totalTableWidth}px;"
              role="row"
              aria-rowindex={virtualRow.index + 1}
            >
              {#each columns as column}
                <div
                  class="table-cell {getCellClass(row, column)}"
                  role="gridcell"
                >
                  {getCellValue(row, column)}
                </div>
              {/each}
            </div>
          {/if}
        {/each}
      {:else if data.length > 0 && columns.length > 0}
        <!-- Fallback: show first few rows if virtualization isn't working -->
        {#each data.slice(0, 20) as row, index}
          <div
            class="virtual-row"
            class:odd={index % 2 === 1}
            style="grid-template-columns: {gridTemplate}; transform: translateY({index * rowHeight}px); width: {totalTableWidth}px;"
            role="row"
            aria-rowindex={index + 1}
          >
            {#each columns as column}
              <div
                class="table-cell {getCellClass(row, column)}"
                role="gridcell"
              >
                {getCellValue(row, column)}
              </div>
            {/each}
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .virtualized-table-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .table-header-wrapper {
    overflow-x: auto;
    overflow-y: hidden;
    border-bottom: 2px solid #e5e7eb;
    flex-shrink: 0;
  }

  .table-header {
    display: grid;
    background: #f9fafb;
    position: sticky;
    top: 0;
    z-index: 10;
    min-width: fit-content;
  }

  .header-cell {
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    font-size: 0.875rem;
    color: #374151;
    border-right: 1px solid #e5e7eb;
  }

  .header-cell:last-child {
    border-right: none;
  }

  .table-body {
    flex: 1;
    overflow: auto;
    position: relative;
  }

  .virtual-spacer {
    position: relative;
    min-width: fit-content;
  }

  .virtual-row {
    display: grid;
    position: absolute;
    left: 0;
    border-bottom: 1px solid #f3f4f6;
    min-width: fit-content;
  }

  .virtual-row.odd {
    background: #fafafa;
  }

  .virtual-row:hover {
    background: #f3f4f6;
  }

  .table-cell {
    padding: 0.75rem;
    font-size: 0.875rem;
    color: #1f2937;
    border-right: 1px solid #f3f4f6;
    white-space: nowrap;
    overflow: visible;
  }

  .table-cell:last-child {
    border-right: none;
  }

  .table-cell.font-mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace;
  }
</style>

