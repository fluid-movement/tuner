<script lang="ts">
	import { minutesText, type WeekDay } from '#lib/practice/progress.ts';

	interface Props {
		days: WeekDay[];
		onpractice: () => void;
	}
	let { days, onpractice }: Props = $props();
	let minutes = $derived(minutesText(days.reduce((s, d) => s + d.seconds, 0)));
	let played = $derived(days.filter((d) => d.seconds > 0).length);
</script>

<div class="strip">
	<ol class="days" aria-label="Days played this week" data-testid="week">
		{#each days as d (d.day)}
			<li class:played={d.seconds > 0} class:today={d.today} title={d.day}>
				<span class="dot" aria-hidden="true"></span>
				<span class="label">{d.label}</span>
				<span class="sr">{d.seconds > 0 ? 'played' : 'not played'}</span>
			</li>
		{/each}
	</ol>
	<p class="sum">
		{#if played}{minutes} · {played} {played === 1 ? 'day' : 'days'}{:else}This week{/if}
	</p>
	<button type="button" class="practice" onclick={onpractice}>Practice</button>
</div>

<style>
	.strip {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 4px 4px 4px 8px;
	}
	.days {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		gap: 6px;
	}
	.days li {
		display: grid;
		justify-items: center;
		gap: 3px;
	}
	.dot {
		width: 10px;
		height: 10px;
		border-radius: 999px;
		border: 1.5px solid var(--line);
	}
	.played .dot {
		background: var(--accent);
		border-color: var(--accent);
	}
	.label {
		font-size: 0.65rem;
		color: var(--muted);
	}
	.today .label {
		color: var(--text);
		font-weight: 700;
	}
	.sr {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
	}
	.sum {
		flex: 1;
		margin: 0;
		color: var(--muted);
		font-size: 0.85rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.practice {
		min-height: 40px;
		padding: 0 16px;
		border: 1px solid var(--line);
		border-radius: 999px;
		background: var(--surface);
		color: var(--text);
		font-weight: 600;
		cursor: pointer;
	}
</style>
