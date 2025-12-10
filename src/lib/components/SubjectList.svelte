<script lang="ts">
  import type { DMRow } from '@/lib/cdisc';

  export type ColumnConfig = {
    key: string;
    label: string;
    trimCommon?: boolean; // Whether to trim common prefix/suffix for this column
  };

  interface Props {
    subjects: DMRow[];
    selected: string[];
    panelWidth: number;
    showCheckbox?: boolean;
    trimCommonChars?: boolean;
    showHeader?: boolean;
    columns?: ColumnConfig[];
  }

  let { 
    subjects, 
    selected = $bindable([]), 
    panelWidth = $bindable(20), 
    showCheckbox = false,
    trimCommonChars = true,
    showHeader = false,
    columns
  }: Props = $props();

  let isResizing = $state(false);
  let startX = 0;
  let startWidth = 0;

  // Default columns if not specified
  let defaultColumns: ColumnConfig[] = [
    { key: 'USUBJID', label: 'SUBJID', trimCommon: true },
    { key: 'SEX', label: 'SEX', trimCommon: false },
    { key: 'AGE', label: 'AGE', trimCommon: false },
    { key: 'ARM', label: 'Cohort', trimCommon: true }
  ];

  let displayColumns = $derived(columns || defaultColumns);

  // Helper function to find common prefix
  function findCommonPrefix(strings: string[]): string {
    if (strings.length === 0) return '';
    if (strings.length === 1) return '';
    
    let prefix = strings[0];
    for (let i = 1; i < strings.length; i++) {
      while (strings[i].indexOf(prefix) !== 0) {
        prefix = prefix.slice(0, -1);
        if (prefix === '') return '';
      }
    }
    return prefix;
  }

  // Helper function to find common suffix
  function findCommonSuffix(strings: string[]): string {
    if (strings.length === 0) return '';
    if (strings.length === 1) return '';
    
    let suffix = strings[0];
    for (let i = 1; i < strings.length; i++) {
      while (!strings[i].endsWith(suffix)) {
        suffix = suffix.slice(1);
        if (suffix === '') return '';
      }
    }
    return suffix;
  }

  // Trim common characters from a value
  function trimCommon(value: string, allValues: string[]): string {
    if (!trimCommonChars || allValues.length <= 1) return value;
    
    const commonPrefix = findCommonPrefix(allValues);
    const commonSuffix = findCommonSuffix(allValues);
    
    let trimmed = value;
    if (commonPrefix && trimmed.startsWith(commonPrefix)) {
      trimmed = trimmed.slice(commonPrefix.length);
    }
    if (commonSuffix && trimmed.endsWith(commonSuffix)) {
      trimmed = trimmed.slice(0, -commonSuffix.length);
    }
    
    return trimmed || value; // Return original if trimming removes everything
  }

  // Get display value for a column
  function getDisplayValue(subject: DMRow, column: ColumnConfig): string {
    const value = (subject as Record<string, string | undefined>)[column.key] ?? '—';
    if (value === '—' || !column.trimCommon) return value;
    
    const allValues = subjects
      .map(s => (s as Record<string, string | undefined>)[column.key] ?? '')
      .filter(v => v && v !== '—');
    
    return trimCommon(value, allValues);
  }

  // Calculate column widths based on max character length
  function calculateColumnWidth(column: ColumnConfig): number {
    if (subjects.length === 0) return 6;
    
    const values = subjects.map(s => {
      const displayValue = getDisplayValue(s, column);
      return displayValue.length;
    });
    
    const maxLength = Math.max(...values, column.label.length);
    // Use actual character length + padding (0.6rem per character + 1rem padding)
    return Math.max(3, maxLength * 0.6 + 1);
  }

  let columnWidths = $derived.by(() => {
    const widths: Record<string, number> = {};
    for (const col of displayColumns) {
      widths[col.key] = calculateColumnWidth(col);
    }
    return widths;
  });

  // Determine which columns to show based on panel width
  let baseWidth = $derived(showCheckbox ? 2.5 : 0);
  let visibleColumns = $derived.by(() => {
    let currentWidth = baseWidth;
    const visible: ColumnConfig[] = [];
    
    for (const col of displayColumns) {
      const colWidth = columnWidths[col.key] || 6;
      if (panelWidth >= currentWidth + colWidth) {
        visible.push(col);
        currentWidth += colWidth;
      } else {
        break;
      }
    }
    
    return visible;
  });

  let styleString = $derived.by(() => {
    const styles = [`width: ${panelWidth}rem`];
    for (const [key, width] of Object.entries(columnWidths)) {
      styles.push(`--${key.toLowerCase()}-width: ${width}rem`);
    }
    return styles.join('; ');
  });


  function toggleSubject(id: string) {
    const index = selected.indexOf(id);
    if (index === -1) {
      selected = [...selected, id];
    } else {
      selected = selected.filter(s => s !== id);
    }
  }

  function isSelected(id: string): boolean {
    return selected.includes(id);
  }

  function handleMouseDown(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = panelWidth;
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const diff = (e.clientX - startX) / 16; // Convert pixels to rem (assuming 16px base)
    const newWidth = Math.max(5, Math.min(40, startWidth + diff)); // Min 5rem (just checkbox), max 40rem
    panelWidth = newWidth;
  }

  function handleMouseUp() {
    isResizing = false;
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }
</script>

<div 
  class="subject-list" 
  style={styleString}
>
  {#if showHeader}
    <div class="header">
      <h2 class="font-semibold">Subjects</h2>
      {#if selected.length > 0}
        <div class="text-sm text-gray-600">{selected.length} selected</div>
      {/if}
    </div>
  {/if}
  <div class="table-container">
    {#if subjects.length === 0}
      <div class="empty-state">No subjects loaded</div>
    {:else}
      <table class="subject-table">
        <thead>
          <tr>
            {#if showCheckbox}
              <th class="checkbox-col"></th>
            {/if}
            {#each visibleColumns as column}
              <th 
                class="text-left {column.key.toLowerCase()}-col"
                style="width: var(--{column.key.toLowerCase()}-width, 6rem);"
              >
                {column.label}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each subjects as subject}
            <tr
              class:selected={isSelected(subject.USUBJID)}
              onclick={() => toggleSubject(subject.USUBJID)}
            >
              {#if showCheckbox}
                <td class="checkbox-col">
                  <input
                    type="checkbox"
                    checked={isSelected(subject.USUBJID)}
                    onclick={(e) => e.stopPropagation()}
                    readonly
                    tabindex="-1"
                  />
                </td>
              {/if}
              {#each visibleColumns as column}
                <td 
                  class="{column.key === 'USUBJID' ? 'font-mono ' : ''}{column.key.toLowerCase()}-col"
                  style="width: var(--{column.key.toLowerCase()}-width, 6rem);"
                >
                  {getDisplayValue(subject, column)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
  <button class="resize-handle" aria-label="Resize panel" onmousedown={handleMouseDown} type="button"></button>
</div>

<style>
  .subject-list {
    position: fixed;
    left: 0;
    top: 60px; /* Header height */
    bottom: 50px; /* Footer height */
    background: #ffffff;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    z-index: 100;
  }

  .header {
    padding: 1rem;
    border-bottom: 1px solid #e5e7eb;
    background: #f9fafb;
  }

  .table-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .subject-list:not(:has(.header)) .table-container {
    border-top: 1px solid #e5e7eb;
  }

  .subject-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
    table-layout: fixed;
  }

  .subject-table thead {
    position: sticky;
    top: 0;
    background: #f9fafb;
    z-index: 10;
  }

  .subject-table th {
    padding: 0.5rem;
    font-weight: 600;
    text-align: left;
    border-bottom: 1px solid #e5e7eb;
    font-size: 0.75rem;
    text-transform: uppercase;
    color: #6b7280;
  }

  .subject-table td {
    padding: 0.5rem;
    border-bottom: 1px solid #f3f4f6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subject-table td.checkbox-col,
  .subject-table th.checkbox-col {
    width: 2.5rem;
  }

  .subject-table td.font-mono {
    font-family: ui-monospace, 'Courier New', monospace;
  }

  .subject-table tbody tr {
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .subject-table tbody tr:hover {
    background: #f9fafb;
  }

  .subject-table tbody tr.selected {
    background: #eef6ff;
  }

  .checkbox-col {
    width: 2.5rem;
    text-align: center;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: #9ca3af;
    font-size: 0.875rem;
  }

  input[type="checkbox"] {
    cursor: pointer;
    pointer-events: none;
  }

  .resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    z-index: 200;
  }

  .resize-handle:hover {
    background: #3b82f6;
  }

  .resize-handle:focus {
    outline: none;
  }

  .subject-list:has(.resize-handle:hover) {
    user-select: none;
  }
</style>

