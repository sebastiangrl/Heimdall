/** Alineado con `heimdall_telemetry::ProcessSample` (serde camelCase). */
export interface ProcessFootprint {
  pid: number;
  name: string;
  /** Porcentaje de un núcleo. null si la muestra aún no tiene delta. */
  cpu_percent: number | null;
  memory_bytes: number;
}

export interface ProcessSample {
  byCpu: ProcessFootprint[];
  byMemory: ProcessFootprint[];
}