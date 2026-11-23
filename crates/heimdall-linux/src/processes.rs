use heimdall_core::{finite_f32, ProcessFootprint};

/// Fila de la única pasada de procesos de este tick. No incluye argumentos de comando.
#[derive(Debug, Clone)]
pub struct ProcDetail {
    pub pid: u32,
    pub name: String,
    /// Porcentaje de un núcleo. None si no hay delta o el valor no es finito.
    pub cpu_core_percent: Option<f32>,
    /// Porcentaje de la capacidad total de la máquina.
    pub cpu_machine_percent: Option<f32>,
    pub memory_bytes: u64,
    pub start_time_ticks: u64,
    pub identity_confirmed: bool,
    /// Entra en el conjunto histórico acotado (top de CPU o de memoria).
    pub follow: bool,
}

pub struct RawProc {
    pub pid: u32,
    pub name: String,
    pub cpu_core_percent: Option<f32>,
    pub cpu_machine_percent: Option<f32>,
    pub memory_bytes: u64,
}

/// `raw` es el porcentaje de sysinfo: 100 = un núcleo lógico. La escala de máquina es `raw / núcleos`.
/// La primera pasada (`primed == false`) no tiene delta: se publica como ausente, no como cero.
pub fn finite_cpu(raw: f32, primed: bool, logical_cores: u32) -> (Option<f32>, Option<f32>) {
    if !primed || logical_cores == 0 {
        return (None, None);
    }
    let Some(core) = finite_f32(raw) else {
        return (None, None);
    };
    let Some(machine) = finite_f32(core / logical_cores as f32) else {
        return (None, None);
    };
    (Some(core), Some(machine))
}

pub fn footprint_from(detail: &ProcDetail, boot_id: &str) -> ProcessFootprint {
    ProcessFootprint {
        pid: detail.pid,
        name: detail.name.clone(),
        cpu_percent: detail.cpu_core_percent.filter(|v| v.is_finite()),
        cpu_machine_percent: detail.cpu_machine_percent.filter(|v| v.is_finite()),
        memory_bytes: detail.memory_bytes,
        boot_id: boot_id.to_string(),
        start_time_ticks: if detail.identity_confirmed {
            detail.start_time_ticks
        } else {
            0
        },
    }
}

pub fn rank_cpu(rows: &[RawProc], limit: usize) -> Vec<u32> {
    let mut ranked: Vec<&RawProc> = rows.iter().collect();
    ranked.sort_by(|a, b| cmp_cpu_desc(a, b));
    ranked.into_iter().take(limit).map(|p| p.pid).collect()
}

pub fn rank_memory(rows: &[RawProc], limit: usize) -> Vec<u32> {
    let mut ranked: Vec<&RawProc> = rows.iter().collect();
    ranked.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    ranked.into_iter().take(limit).map(|p| p.pid).collect()
}

fn cmp_cpu_desc(a: &RawProc, b: &RawProc) -> std::cmp::Ordering {
    match (a.cpu_core_percent, b.cpu_core_percent) {
        (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_cpu_is_absent_not_zero() {
        assert_eq!(finite_cpu(f32::NAN, true, 12), (None, None));
        assert_eq!(finite_cpu(f32::INFINITY, true, 12), (None, None));
        assert_eq!(finite_cpu(0.0, false, 12), (None, None));
    }

    #[test]
    fn machine_percent_is_core_percent_divided_by_logical_cpus() {
        let (core, machine) = finite_cpu(100.0, true, 12);
        assert_eq!(core, Some(100.0));
        let machine = machine.unwrap();
        assert!((machine - (100.0 / 12.0)).abs() < 0.001);
    }

    #[test]
    fn real_zero_stays_zero_after_the_first_delta() {
        assert_eq!(finite_cpu(0.0, true, 8), (Some(0.0), Some(0.0)));
    }
}
