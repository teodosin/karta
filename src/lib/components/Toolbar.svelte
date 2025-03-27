<script lang="ts">
    import { currentMode } from '$lib/karta/KartaStore';

    function setMode(mode: 'move' | 'connect') {
        currentMode.set(mode);
    }
</script>

<div class="absolute top-2 left-1/2 transform -translate-x-1/2 z-50 flex gap-2 p-2 bg-white/80 backdrop-blur-sm rounded shadow">
     <button
        class="px-3 py-1 text-sm rounded transition-colors { $currentMode === 'move' ? 'bg-indigo-600 text-white' : 'bg-gray-200 hover:bg-gray-300'}"
        on:click={() => setMode("move")}
    >
        Move
    </button>
    <button
        class="px-3 py-1 text-sm rounded transition-colors { $currentMode === 'connect' ? 'bg-indigo-600 text-white' : 'bg-gray-200 hover:bg-gray-300'}"
        on:click={() => setMode("connect")}
    >
        Connect
    </button>
    <!-- Add Reset View/All later -->
</div>