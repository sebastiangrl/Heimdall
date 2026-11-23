import type { TelemetrySnapshot } from "@/types/telemetry";

export interface StorageMountGroup {
  totalBytes: number;
  usedBytes: number;
  /** Mount points sharing the same capacity snapshot. */
  mountPoints: string[];
  /** Distinct device labels from the collector (e.g. btrfs subvolumes). */
  deviceNames: string[];
}

/**
 * Merge btrfs subvolumes / bind mounts that report identical used+total
 * (same underlying filesystem capacity in one tick).
 */
export function groupStorageMounts(
  storage: TelemetrySnapshot["storage"],
): StorageMountGroup[] {
  const map = new Map<string, StorageMountGroup>();

  for (const d of storage) {
    const total = d.total_bytes.value;
    const used = d.used_bytes.value;
    if (total == null || used == null || !Number.isFinite(total) || !Number.isFinite(used)) {
      continue;
    }
    const key = `${total}:${used}`;
    const mount = d.mount_point?.trim() || d.name;
    const existing = map.get(key);
    if (existing) {
      if (mount && !existing.mountPoints.includes(mount)) {
        existing.mountPoints.push(mount);
      }
      if (d.name && !existing.deviceNames.includes(d.name)) {
        existing.deviceNames.push(d.name);
      }
    } else {
      map.set(key, {
        totalBytes: total,
        usedBytes: used,
        mountPoints: mount ? [mount] : [],
        deviceNames: d.name ? [d.name] : [],
      });
    }
  }

  return [...map.values()].sort((a, b) => a.mountPoints[0]?.localeCompare(b.mountPoints[0] ?? "") ?? 0);
}