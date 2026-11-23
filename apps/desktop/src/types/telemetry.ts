export type SensorStatus =
  | "supported"
  | "unsupported"
  | "unavailable"
  | "permission_denied"
  | "stale";

export interface MetricValue<T = number> {
  status: SensorStatus;
  value?: T;
  unit?: string;
  observed_at: string;
  source?: string;
}

export interface TelemetrySnapshot {
  observed_at: string;
  session_id: string;
  cpu: {
    usage_percent_global: MetricValue<number>;
    temperature_c: MetricValue<number>;
    frequency_mhz_avg: MetricValue<number>;
  };
  memory: {
    used_bytes: MetricValue<number>;
    available_bytes: MetricValue<number>;
    total_bytes: MetricValue<number>;
    swap_used_bytes: MetricValue<number>;
  };
  gpu: {
    name?: string;
    active_provider?: string;
    hint_code?: string;
    utilization_percent: MetricValue<number>;
    temperature_c: MetricValue<number>;
    vram_used_bytes: MetricValue<number>;
    vram_total_bytes: MetricValue<number>;
    core_clock_mhz?: MetricValue<number>;
    mem_clock_mhz?: MetricValue<number>;
    power_watts?: MetricValue<number>;
    fan_speed_percent?: MetricValue<number>;
  };
  storage: {
    name: string;
    mount_point?: string;
    used_bytes: MetricValue<number>;
    total_bytes: MetricValue<number>;
  }[];
  system: {
    uptime_seconds: MetricValue<number>;
    load_avg_1: MetricValue<number>;
  };
  psi: {
    memory_some_avg10: MetricValue<number>;
  };
}

export interface HardwareSummary {
  hostname: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  cpu: { name: string; logical_cores: number };
  memory: { total_bytes: number };
  gpus: { name: string; vendor: string; driver_note?: string }[];
  discovered_at: string;
}