import { EmulatorException } from "@specialist/core-api";

export function describeError(error: unknown): string {
  if (error instanceof EmulatorException) {
    switch (error.error.kind) {
      case "invalidAddress":
        return `Некорректный адрес: ${error.error.address}`;
      case "rangeOutOfBounds":
        return `Диапазон выходит за пределы памяти: адрес ${error.error.address}, длина ${error.error.length}`;
      case "memoryNotWritable":
        return `Диапазон недоступен для записи: адрес ${error.error.address}, длина ${error.error.length}`;
      case "invalidRomSize":
        return `Неверный размер Monitor 2 ROM: ожидалось ${error.error.expected}, получено ${error.error.actual}`;
      case "missingRom":
        return "Monitor 2 ROM не загружен";
      case "unknownKey":
        return `Неизвестная клавиша «Специалиста»: ${error.error.key}`;
      case "eventCycleInPast":
        return `Такт события клавиатуры уже прошёл: ${error.error.cycle}`;
    }
  }
  return error instanceof Error ? error.message : String(error);
}
