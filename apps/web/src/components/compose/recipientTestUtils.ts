/** The serialised value of a recipient field in tests: its chips (comma separated) then the typed text. */
export function recipientValue(input: HTMLElement): string {
  const root = input.closest('.compose-rf');
  const chips = Array.from(root?.querySelectorAll<HTMLElement>('li[data-recipient]') ?? []).map(item => item.dataset.recipient ?? '');
  const typed = (input as HTMLInputElement).value;
  return (chips.length > 0 ? `${chips.join(', ')}, ` : '') + typed;
}

/** The finished recipients of a field (by its input or group), comma separated, no trailing separator. */
export function recipientList(input: HTMLElement): string {
  return recipientValue(input).replace(/,\s*$/u, '');
}
