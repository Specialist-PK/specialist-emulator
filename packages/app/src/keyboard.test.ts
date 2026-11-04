import type { SpecialistKeyEvent } from "@specialist/core-api";
import { describe, expect, it } from "vitest";

import { bindKeyboard, specialistKeyForCode } from "./keyboard";

function keyboardEvent(type: string, code: string, target: EventTarget): KeyboardEvent {
  const event = new Event(type, { cancelable: true });
  Object.defineProperties(event, {
    code: { value: code },
    target: { value: target },
  });
  return event as KeyboardEvent;
}

function editableTarget(): EventTarget {
  return { closest: () => ({}) } as unknown as EventTarget;
}

function plainTarget(): EventTarget {
  return { closest: () => null } as unknown as EventTarget;
}

function setup() {
  const target = new EventTarget() as unknown as Window;
  const events: SpecialistKeyEvent[] = [];
  const release = bindKeyboard({
    target,
    emit: async (event) => {
      events.push(event);
    },
  });
  return { target, events, release };
}

async function drain() {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe("specialistKeyForCode", () => {
  it("maps the QWERTY code contract", () => {
    expect(specialistKeyForCode("KeyA")).toBe("a");
    expect(specialistKeyForCode("Digit0")).toBe("0");
    expect(specialistKeyForCode("F11")).toBe("f11");
    expect(specialistKeyForCode("F12")).toBe("clear");
    expect(specialistKeyForCode("Equal")).toBe("colon");
    expect(specialistKeyForCode("Quote")).toBe("caret");
    expect(specialistKeyForCode("Backquote")).toBe("at");
    expect(specialistKeyForCode("ControlLeft")).toBe("rpt");
    expect(specialistKeyForCode("PageDown")).toBe("lf");
    expect(specialistKeyForCode("ShiftRight")).toBe("shift");
    expect(specialistKeyForCode("Numpad0")).toBe("lang");
    expect(specialistKeyForCode("CapsLock")).toBeUndefined();
  });
});

describe("bindKeyboard", () => {
  it("serializes press and release and suppresses auto-repeat", async () => {
    const { target, events, release } = setup();
    const first = keyboardEvent("keydown", "KeyA", plainTarget());
    target.dispatchEvent(first);
    target.dispatchEvent(keyboardEvent("keydown", "KeyA", plainTarget()));
    target.dispatchEvent(keyboardEvent("keyup", "KeyA", plainTarget()));
    await drain();

    expect(first.defaultPrevented).toBe(true);
    expect(events).toEqual([
      { key: "a", pressed: true },
      { key: "a", pressed: false },
    ]);
    release();
  });

  it("releases every pressed key on blur and cleanup", async () => {
    const { target, events, release } = setup();
    target.dispatchEvent(keyboardEvent("keydown", "KeyA", plainTarget()));
    target.dispatchEvent(keyboardEvent("keydown", "ShiftLeft", plainTarget()));
    target.dispatchEvent(new Event("blur"));
    await drain();
    expect(events).toEqual([
      { key: "a", pressed: true },
      { key: "shift", pressed: true },
      { key: "a", pressed: false },
      { key: "shift", pressed: false },
    ]);

    target.dispatchEvent(keyboardEvent("keydown", "KeyQ", plainTarget()));
    release();
    await drain();
    expect(events.at(-1)).toEqual({ key: "q", pressed: false });
  });

  it("ignores editable fields but releases keys pressed earlier", async () => {
    const { target, events, release } = setup();
    const editable = keyboardEvent("keydown", "KeyB", editableTarget());
    target.dispatchEvent(editable);
    target.dispatchEvent(keyboardEvent("keydown", "KeyA", plainTarget()));
    target.dispatchEvent(keyboardEvent("keyup", "KeyA", editableTarget()));
    await drain();

    expect(editable.defaultPrevented).toBe(false);
    expect(events).toEqual([
      { key: "a", pressed: true },
      { key: "a", pressed: false },
    ]);
    release();
  });
});
