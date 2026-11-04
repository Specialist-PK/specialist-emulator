import type { SpecialistKey, SpecialistKeyEvent } from "@specialist/core-api";

const SPECIAL_KEYS: Readonly<Record<string, SpecialistKey>> = {
  Enter: "cr",
  NumpadEnter: "cr",
  Backspace: "bsp",
  Period: "period",
  Equal: "colon",
  Minus: "minus",
  F12: "clear",
  PageUp: "clear",
  PageDown: "lf",
  Slash: "slash",
  Backslash: "backslash",
  ArrowRight: "right",
  Numpad6: "right",
  Comma: "comma",
  ArrowLeft: "left",
  Numpad4: "left",
  Space: "space",
  Escape: "esc",
  Tab: "tab",
  ArrowDown: "down",
  Numpad2: "down",
  ArrowUp: "up",
  Numpad8: "up",
  Home: "home",
  Quote: "caret",
  Backquote: "at",
  Semicolon: "semicolon",
  BracketLeft: "lbracket",
  BracketRight: "rbracket",
  ShiftLeft: "shift",
  ShiftRight: "shift",
  ControlLeft: "rpt",
  ControlRight: "rpt",
  Insert: "lang",
  Numpad0: "lang",
};

export function specialistKeyForCode(code: string): SpecialistKey | undefined {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase() as SpecialistKey;
  if (/^Digit\d$/.test(code)) return code.slice(5) as SpecialistKey;
  if (/^F(?:[1-9]|10|11)$/.test(code)) return code.toLowerCase() as SpecialistKey;
  return SPECIAL_KEYS[code];
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (typeof target !== "object" || target === null || !("closest" in target)) return false;
  const closest = target.closest;
  if (typeof closest !== "function") return false;
  return closest.call(target, "input, textarea, select, [contenteditable='true']") !== null;
}

interface KeyboardBindingOptions {
  target: Window;
  emit: (event: SpecialistKeyEvent) => Promise<unknown>;
  onError?: (error: unknown) => void;
}

export function bindKeyboard(options: KeyboardBindingOptions): () => void {
  const pressed = new Set<SpecialistKey>();
  let pending: Promise<unknown> = Promise.resolve();

  function emit(key: SpecialistKey, isPressed: boolean) {
    pending = pending
      .then(() => options.emit({ key, pressed: isPressed }))
      .catch((error: unknown) => options.onError?.(error));
  }

  function releaseAll() {
    for (const key of pressed) emit(key, false);
    pressed.clear();
  }

  function keydown(event: KeyboardEvent) {
    if (isEditableTarget(event.target)) return;
    const key = specialistKeyForCode(event.code);
    if (key === undefined) return;
    event.preventDefault();
    if (pressed.has(key)) return;
    pressed.add(key);
    emit(key, true);
  }

  function keyup(event: KeyboardEvent) {
    const key = specialistKeyForCode(event.code);
    if (key === undefined || (!pressed.has(key) && isEditableTarget(event.target))) return;
    event.preventDefault();
    if (!pressed.delete(key)) return;
    emit(key, false);
  }

  function blur() {
    releaseAll();
  }

  options.target.addEventListener("keydown", keydown);
  options.target.addEventListener("keyup", keyup);
  options.target.addEventListener("blur", blur);
  return () => {
    options.target.removeEventListener("keydown", keydown);
    options.target.removeEventListener("keyup", keyup);
    options.target.removeEventListener("blur", blur);
    releaseAll();
  };
}
