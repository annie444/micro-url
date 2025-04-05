type LocalStorageKey = 'hasDismissedWelcomeToast'

export class LocalStorageHelper {
  static setItem(key: LocalStorageKey, value: string) {
    localStorage.setItem(key, value)
  }

  static getItem(key: LocalStorageKey) {
    return localStorage.getItem(key)
  }

  static removeItem(key: LocalStorageKey) {
    localStorage.removeItem(key)
  }

  static clear() {
    localStorage.clear()
  }
}
