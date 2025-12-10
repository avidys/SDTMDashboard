<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';

    let content = '';
    async function openFile() {
      const file = await open({ multiple: false });
      if (file) {
        content = await invoke('read_text_file', { path: file });
      }
    }
</script>
  
<button on:click={openFile}>Open file</button>
<pre>{content}</pre>