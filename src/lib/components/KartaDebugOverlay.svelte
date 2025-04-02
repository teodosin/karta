<script lang="ts">
	import { viewTransform } from '$lib/karta/KartaStore';
	import { onMount, onDestroy, tick } from 'svelte';
	
	// Create local copies to trigger reactivity
	let current = { ...viewTransform.current };
	let target = { ...viewTransform.target };
	
	// Format the values
	$: currentScale = current.scale.toFixed(3);
	$: currentTx = current.posX.toFixed(3);
	$: currentTy = current.posY.toFixed(3);
	
	$: targetScale = target.scale.toFixed(3);
	$: targetTx = target.posX.toFixed(3);
	$: targetTy = target.posY.toFixed(3);
	
	// Set up an interval to update the local copies
	let intervalId: any;
	
	function updateValues() {
	  current = { ...viewTransform.current };
	  target = { ...viewTransform.target };
	}
	
	onMount(() => {
	  // Set initial values
	  updateValues();
	  
	  // Update periodically
	  intervalId = setInterval(updateValues, 16);
	});
	
	onDestroy(() => {
	  if (intervalId) clearInterval(intervalId);
	});
  </script>
  
  <div class="fixed top-2.5 right-2.5 z-[1000] p-2 bg-black/50 text-white rounded text-xs font-mono pointer-events-none">
	<div class="font-bold">Target:</div>
	<div>Scale: {targetScale}</div>
	<div>Tx: {targetTx}</div>
	<div>Ty: {targetTy}</div>
	<hr class="my-1 border-gray-400">
	<div class="font-bold">Current ($):</div>
	<div>Scale: {currentScale}</div>
	<div>Tx: {currentTx}</div>
	<div>Ty: {currentTy}</div>
  </div>