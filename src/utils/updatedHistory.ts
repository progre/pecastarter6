export default function updatedHistory(
  newValue: string,
  history: readonly string[],
  limit: number
): readonly string[] {
  return [
    newValue,
    ...history
      .filter((x) => x !== newValue && x.trim() !== '')
      .filter((_, i) => i < limit - (newValue.trim() !== '' ? 1 : 0)),
  ];
}
