import { EmulatorException } from "@specialist/core-api";

export function describeError(error: unknown): string {
  if (error instanceof EmulatorException) {
    return error.error.kind === "invalidAddress"
      ? `Некорректный адрес: ${error.error.address}`
      : `Диапазон выходит за пределы памяти: адрес ${error.error.address}, длина ${error.error.length}`;
  }
  return error instanceof Error ? error.message : String(error);
}
