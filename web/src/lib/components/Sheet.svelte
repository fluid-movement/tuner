<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import Icon from './Icon.svelte';

	interface Props {
		/** Heading, also the dialog's name. */
		title: string;
		/** Closes the sheet (close button, Escape, backdrop). */
		onclose: () => void;
		/** Shown at the right of the title bar, before the close button. */
		actions?: Snippet;
		children: Snippet;
		testid?: string;
	}
	let { title, onclose, actions, children, testid }: Props = $props();

	const id = `sheet-${Math.random().toString(36).slice(2, 8)}`;
	let closeButton = $state<HTMLButtonElement>();
	/** Downward drag of the sheet by its top bar, px (swipe down to close). */
	let drag = $state(0);
	let dragFrom = $state<{ y: number; id: number } | null>(null);
	/** Dragged this far (or flicked), the sheet closes. */
	const CLOSE_PX = 96;

	function grab(e: PointerEvent) {
		if ((e.target as HTMLElement).closest('button, select, input')) return;
		dragFrom = { y: e.clientY, id: e.pointerId };
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}
	function move(e: PointerEvent) {
		if (dragFrom?.id === e.pointerId) drag = Math.max(0, e.clientY - dragFrom.y);
	}
	function release(e: PointerEvent) {
		if (dragFrom?.id !== e.pointerId) return;
		dragFrom = null;
		if (drag > CLOSE_PX) onclose();
		else drag = 0;
	}
	const reduced =
		typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;

	$effect(() => {
		closeButton?.focus({ preventScroll: true });
		// The page behind doesn't scroll while the sheet is open.
		const overflow = document.body.style.overflow;
		document.body.style.overflow = 'hidden';
		return () => {
			document.body.style.overflow = overflow;
		};
	});
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<div class="layer">
	<button
		type="button"
		class="backdrop"
		aria-label="Close"
		tabindex="-1"
		onclick={onclose}
		transition:fade={{ duration: reduced ? 0 : 180 }}
	></button>
	<div
		class="sheet"
		role="dialog"
		aria-modal="true"
		aria-labelledby={id}
		data-testid={testid}
		style:transform={drag ? `translateY(${drag}px)` : null}
		class:dragging={dragFrom !== null}
		transition:fly={{ y: reduced ? 0 : 480, duration: reduced ? 0 : 260, easing: cubicOut }}
	>
		<div
			class="grip"
			role="presentation"
			onpointerdown={grab}
			onpointermove={move}
			onpointerup={release}
			onpointercancel={release}
		>
			<div class="handle" aria-hidden="true"></div>
			<div class="bar">
				<h2 {id}>{title}</h2>
				<div class="actions">
					{@render actions?.()}
					<button
						type="button"
						class="close"
						aria-label="Close {title}"
						bind:this={closeButton}
						onclick={onclose}
					>
						<Icon name="close" />
					</button>
				</div>
			</div>
		</div>
		<div class="content">
			{@render children()}
		</div>
	</div>
</div>

<style>
	.layer {
		position: fixed;
		inset: 0;
		z-index: 10;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		align-items: center;
	}
	.backdrop {
		position: absolute;
		inset: 0;
		border: 0;
		padding: 0;
		background: rgb(0 0 0 / 0.35);
		cursor: default;
	}
	.sheet {
		position: relative;
		width: 100%;
		max-width: 560px;
		max-height: calc(100dvh - max(24px, env(safe-area-inset-top)));
		display: flex;
		flex-direction: column;
		background: var(--bg);
		border-radius: 20px 20px 0 0;
		box-shadow: 0 -8px 32px rgb(0 0 0 / 0.18);
	}
	.grip {
		/* The top bar drags the sheet down; the content still scrolls. */
		touch-action: none;
		cursor: grab;
	}
	.sheet:not(.dragging) {
		transition: transform 200ms ease-out;
	}
	.handle {
		width: 40px;
		height: 5px;
		margin: 8px auto 0;
		border-radius: 999px;
		background: var(--line);
	}
	.bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 4px 12px 8px 20px;
	}
	h2 {
		margin: 0;
		font-size: 1.15rem;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.close {
		display: grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border: 0;
		border-radius: 999px;
		background: var(--line);
		color: var(--text);
		cursor: pointer;
	}
	.content {
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 4px 16px max(20px, env(safe-area-inset-bottom));
		display: grid;
		gap: 14px;
	}
</style>
