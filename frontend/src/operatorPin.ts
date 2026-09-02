/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-02 17:58:31 Europe/London
REASON FOR CREATION: Define the four-cell operator PIN boundary and build its HTTP Authorization header without storing the secret.
==========================================
*/

export const OPERATOR_PIN_LENGTH = 4

export type OperatorPinCells = [string, string, string, string]

export function createEmptyPinCells(): OperatorPinCells {
  return ['', '', '', '']
}

export function pinFromCells(cells: readonly string[]): string {
  if (
    cells.length !== OPERATOR_PIN_LENGTH ||
    cells.some((cell) => Array.from(cell).length !== 1)
  ) {
    return ''
  }
  return cells.join('')
}

export function authorizationHeaders(pin: string): Record<string, string> {
  return Array.from(pin).length === OPERATOR_PIN_LENGTH
    ? { authorization: `Bearer ${pin}` }
    : {}
}

export function pinCellsFromText(text: string): OperatorPinCells {
  const characters = Array.from(text).slice(0, OPERATOR_PIN_LENGTH)
  return [
    characters[0] ?? '',
    characters[1] ?? '',
    characters[2] ?? '',
    characters[3] ?? '',
  ]
}

export function replacePinCell(
  cells: readonly string[],
  index: number,
  value: string,
): OperatorPinCells {
  const next: OperatorPinCells = [
    Array.from(cells[0] ?? '').at(-1) ?? '',
    Array.from(cells[1] ?? '').at(-1) ?? '',
    Array.from(cells[2] ?? '').at(-1) ?? '',
    Array.from(cells[3] ?? '').at(-1) ?? '',
  ]
  if (index >= 0 && index < OPERATOR_PIN_LENGTH) {
    next[index] = Array.from(value).at(-1) ?? ''
  }
  return next
}
