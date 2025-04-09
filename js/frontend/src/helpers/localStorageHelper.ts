type LocalStorageKey = "hasDismissedWelcomeToast" | "token";

export function setItem(key: LocalStorageKey, value: string) {
  localStorage.setItem(key, value);
}

export function getItem(key: LocalStorageKey) {
  return localStorage.getItem(key);
}

export function removeItem(key: LocalStorageKey) {
  localStorage.removeItem(key);
}

export function clear() {
  localStorage.clear();
}
