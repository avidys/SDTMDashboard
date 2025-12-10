<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';
    import { browser } from '$app/environment';
    import { parseCSV, dmSubjects, pcForSubject, timeFromPC, type DMRow, type PCRow } from '@/lib/cdisc';
    import SubjectList from '@/lib/components/SubjectList.svelte';
    import VirtualizedTable, { type Column } from '@/lib/components/VirtualizedTable.svelte';
    import type Plotly from 'plotly.js-dist-min';
  
    let PlotlyModule: typeof Plotly | null = null;
    let dm = $state<DMRow[]>([]);
    let pc = $state<PCRow[]>([]);
    let selected = $state<string[]>([]);
    let pcFiltered = $state<PCRow[]>([]);
    let units = $state('');
    let panelWidth = $state(20); // in rem
    let showRawPopup = $state(false);

    // Get all available columns from PC data and create column definitions
    let tableColumns = $derived.by(() => {
      if (pcFiltered.length === 0) return [];
      const allKeys = new Set<string>();
      for (const row of pcFiltered) {
        Object.keys(row).forEach(key => allKeys.add(key));
      }
      const sortedKeys = Array.from(allKeys).sort();
      return sortedKeys.map(key => ({
        key,
        label: key,
        cellClass: key === 'USUBJID' ? 'font-mono' : undefined,
        width: key === 'USUBJID' ? 150 : undefined
      })) as Column<Record<string, unknown>>[];
    });

    // Load Plotly only on the client side
    if (browser) {
      import('plotly.js-dist-min').then((mod) => {
        PlotlyModule = mod.default;
      });
    }
  
    async function pickCSV(kind: 'DM' | 'PC') {
      const path = await open({ multiple: false, filters:[{ name:'CSV', extensions:['csv'] }] }) as string | null;
      if (!path) return;
      const f = await invoke<{name:string;content:string}>('read_csv', { path });
      const rows = parseCSV(f.content);
      if (kind === 'DM') dm = dmSubjects(rows);
      if (kind === 'PC') {
        pc = rows as unknown as PCRow[];
        // try to detect concentration units
        units = pc.find(r=>r.PCSTRESU)?.PCSTRESU || '';
      }
      updateSelection();
    }

    async function pickXPT(kind: 'DM' | 'PC') {
      try {
        const path = await open({ multiple: false, filters:[{ name:'SAS XPT', extensions:['xpt', 'xport'] }] }) as string | null;
        if (!path) return;
        console.log('Loading XPT file:', path);
        const f = await invoke<{name:string;content:string}>('read_xpt', { path });
        console.log('Received file:', f.name, 'Content length:', f.content.length);
        console.log('First 500 chars:', f.content.substring(0, 500));
        const rows = parseCSV(f.content);
        console.log('Parsed rows:', rows.length);
        if (rows.length > 0) {
          console.log('First row keys:', Object.keys(rows[0]));
          console.log('First row sample:', Object.fromEntries(Object.entries(rows[0]).slice(0, 5)));
        }
        if (kind === 'DM') {
          dm = dmSubjects(rows);
          console.log('DM subjects:', dm.length);
        }
        if (kind === 'PC') {
          pc = rows as unknown as PCRow[];
          console.log('PC rows:', pc.length);
          // try to detect concentration units
          units = pc.find(r=>r.PCSTRESU)?.PCSTRESU || '';
        }
        updateSelection();
      } catch (error) {
        console.error('Error loading XPT file:', error);
        alert(`Error loading XPT file: ${error}`);
      }
    }
  
    function updateSelection() {
      if (selected.length === 0) {
        pcFiltered = [];
        return;
      }
      // Combine PC data for all selected subjects
      pcFiltered = selected.flatMap(id => pcForSubject(pc, id));
      queueMicrotask(drawPlots);
    }

    $effect(() => {
      updateSelection();
    });
  
    function toTable(rows: PCRow[]) {
      return rows
        .map(r => ({ Time: timeFromPC(r), PCTPT: r.PCTPT ?? '', Conc: Number(r.PCSTRESN), Unit: r.PCSTRESU ?? '' }))
        .filter(r => r.Time !== null)
        .sort((a,b) => (a.Time! - b.Time!));
    }
  
    async function drawPlots() {
      if (!browser || !PlotlyModule) return;
      
      if (pcFiltered.length === 0) {
        // Clear plots if no data
        await PlotlyModule.newPlot('plot-linear', [], { title: 'PK Curve (Linear)' }, { displayModeBar: false });
        await PlotlyModule.newPlot('plot-semilog', [], { title: 'PK Curve (Semi-log Y)' }, { displayModeBar: false });
        return;
      }

      // Group data by subject for multi-subject plotting
      const dataBySubject = new Map<string, typeof pcFiltered>();
      for (const row of pcFiltered) {
        if (!dataBySubject.has(row.USUBJID)) {
          dataBySubject.set(row.USUBJID, []);
        }
        dataBySubject.get(row.USUBJID)!.push(row);
      }

      // Create traces for each selected subject
      const traces = Array.from(dataBySubject.entries()).map(([usubjid, rows]) => {
        const tableData = toTable(rows);
        const x = tableData.map(d => d.Time!);
        const y = tableData.map(d => d.Conc);
        return { x, y, mode: 'lines+markers' as const, name: usubjid, type: 'scatter' as const };
      });

      // Linear
      await PlotlyModule.newPlot('plot-linear', traces, {
        title: 'PK Curve (Linear)',
        xaxis: { title: 'Time' },
        yaxis: { title: `Concentration ${units ? '('+units+')' : ''}` }
      }, { displayModeBar: false });

      // Semi-log (log y)
      await PlotlyModule.newPlot('plot-semilog', traces, {
        title: 'PK Curve (Semi-log Y)',
        xaxis: { title: 'Time' },
        yaxis: { type: 'log', title: `Concentration ${units ? '('+units+')' : ''}`, rangemode: 'tozero' }
      }, { displayModeBar: false });
    }
  </script>
  
  <SubjectList subjects={dm} bind:selected bind:panelWidth />
  
  <div class="main-content" style="margin-left: {panelWidth}rem;">
    <div class="p-6 space-y-4">
      <h1 class="text-2xl font-semibold">SDTM PK Viewer (DM + PC)</h1>
    
      <div class="flex gap-3">
        <button class="px-3 py-2 rounded bg-gray-200 hover:bg-gray-300" onclick={() => pickCSV('DM')}>Load DM (CSV)</button>
        <button class="px-3 py-2 rounded bg-gray-200 hover:bg-gray-300" onclick={() => pickXPT('DM')}>Load DM (XPT)</button>
        <button class="px-3 py-2 rounded bg-gray-200 hover:bg-gray-300" onclick={() => pickCSV('PC')}>Load PC (CSV)</button>
        <button class="px-3 py-2 rounded bg-gray-200 hover:bg-gray-300" onclick={() => pickXPT('PC')}>Load PC (XPT)</button>
      </div>
    
      <div class="space-y-4">
        <div class="flex items-center gap-3">
          <h2 class="font-semibold">
            PC Concentrations
            {#if selected.length === 1}
              for {selected[0]}
            {:else if selected.length > 1}
              ({selected.length} subjects)
            {/if}
          </h2>
          {#if pcFiltered.length > 0}
            <button 
              class="px-3 py-1 text-sm rounded bg-blue-500 text-white hover:bg-blue-600"
              onclick={() => showRawPopup = true}
            >
              Raw
            </button>
          {/if}
        </div>
        <div id="plot-linear" class="border rounded h-[260px]"></div>
        <div id="plot-semilog" class="border rounded h-[260px]"></div>
      </div>
    </div>
  </div>

  <!-- Raw Data Popup -->
  {#if showRawPopup}
    <div 
      class="popup-overlay" 
      role="dialog"
      aria-modal="true"
      aria-label="Raw PC Data"
      onclick={(e) => e.target === e.currentTarget && (showRawPopup = false)}
      onkeydown={(e) => e.key === 'Escape' && (showRawPopup = false)}
      tabindex="-1"
    >
      <div class="popup-content">
        <div class="popup-header">
          <h3 class="text-lg font-semibold">Raw PC Data</h3>
          <button 
            class="close-button"
            onclick={() => showRawPopup = false}
            aria-label="Close"
          >
            ×
          </button>
        </div>
        <div class="popup-body">
          {#if tableColumns.length > 0 && pcFiltered.length > 0}
            <VirtualizedTable
              data={pcFiltered as Record<string, unknown>[]}
              columns={tableColumns}
              rowHeight={40}
              overscan={10}
            />
          {:else}
            <div class="text-center text-gray-500 py-8">No data available</div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <style>
    .main-content {
      min-height: calc(100vh - 110px); /* Full height minus header and footer */
      transition: margin-left 0.1s ease-out;
    }

    .popup-overlay {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 1000;
    }

    .popup-content {
      background: white;
      border-radius: 8px;
      box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
      width: 90%;
      max-width: 1200px;
      max-height: 80vh;
      display: flex;
      flex-direction: column;
    }

    .popup-header {
      padding: 1rem 1.5rem;
      border-bottom: 1px solid #e5e7eb;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .close-button {
      background: none;
      border: none;
      font-size: 1.5rem;
      cursor: pointer;
      color: #6b7280;
      padding: 0;
      width: 2rem;
      height: 2rem;
      display: flex;
      align-items: center;
      justify-content: center;
      border-radius: 4px;
    }

    .close-button:hover {
      background: #f3f4f6;
      color: #1f2937;
    }

    .popup-body {
      padding: 0;
      flex: 1;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      min-height: 0;
    }

    .popup-body :global(.virtualized-table-container) {
      padding: 1.5rem;
      height: 100%;
    }
  </style>