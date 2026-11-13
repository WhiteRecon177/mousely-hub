/**
 * Reads a localStorage item, migrating from a legacy key when the new key is absent.
 * Copies the legacy value to the new key and deletes the legacy key so future reads
 * use only the new key.
 */
export function getStorageItemWithMigration(key: string, legacyKey: string): string | null {
  try {
    const current = localStorage.getItem(key);
    if (current !== null) return current;
    const legacy = localStorage.getItem(legacyKey);
    if (legacy === null) return null;
    try {
      localStorage.setItem(key, legacy);
      localStorage.removeItem(legacyKey);
    } catch {
      // Ignore storage write/remove errors during migration; return legacy value anyway.
    }
    return legacy;
  } catch {
    return null;
  }
}
