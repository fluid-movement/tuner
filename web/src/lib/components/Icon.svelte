<script lang="ts">
	interface Props {
		name: 'tune' | 'chords' | 'close';
		size?: number;
	}
	let { name, size = 22 }: Props = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="1.8"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#if name === 'tune'}
		<!-- Tuning fork. -->
		<path d="M8 3v7a4 4 0 0 0 8 0V3" />
		<path d="M12 14v7" />
	{:else if name === 'chords'}
		<!-- A chord box: nut, frets, strings and a dot. -->
		<rect x="5" y="3" width="14" height="18" rx="2" />
		<path d="M5 8h14M5 13h14M9.7 3v18M14.3 3v18" />
		<circle cx="14.3" cy="10.5" r="1.4" fill="currentColor" stroke="none" />
	{:else}
		<path d="M6 6l12 12M18 6L6 18" />
	{/if}
</svg>
