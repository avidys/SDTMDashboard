<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="app-container">
	<header class="fixed-header">
		<h1>SDTM Dashboard</h1>
	</header>
	
	<main class="main-content">
		{@render children()}
	</main>
	
	<footer class="fixed-footer">
		<p>&copy; 2025 SDTM Dashboard by Avidys LLC. All rights reserved.</p>
	</footer>
</div>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		height: 100vh;
		overflow: hidden;
	}

	:global(html) {
		margin: 0;
		padding: 0;
		height: 100%;
	}

	.app-container {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.fixed-header {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		height: 60px;
		background-color: #1e1e1e;
		color: #ffffff;
		display: flex;
		align-items: center;
		padding: 0 20px;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
		z-index: 1000;
	}

	.fixed-header h1 {
		margin: 0;
		font-size: 1.5rem;
		font-weight: 600;
	}

	.main-content {
		flex: 1;
		margin-top: 60px;
		margin-bottom: 50px;
		overflow-y: auto;
		padding: 20px;
		background-color: #ffffff;
	}

	.fixed-footer {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		height: 50px;
		background-color: #1e1e1e;
		color: #ffffff;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 -2px 4px rgba(0, 0, 0, 0.1);
		z-index: 1000;
	}

	.fixed-footer p {
		margin: 0;
		font-size: 0.875rem;
	}
</style>
