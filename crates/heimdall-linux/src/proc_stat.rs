/// Identidad de arranque del kernel y starttime de un proceso.
///
/// `boot_id` identifica el arranque del sistema. El `session_id` del sampler es otra cosa:
/// cambia cada vez que Heimdall crea un `LinuxSampler`, aunque el kernel no se haya reiniciado.

pub fn read_boot_id() -> String {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Campo 22 de `/proc/[pid]/stat`: ticks desde el arranque (resolución de `CLK_TCK`, 10 ms en este equipo).
pub fn read_start_ticks(pid: u32) -> Option<u64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_start_ticks(&text)
}

pub fn parse_start_ticks(stat: &str) -> Option<u64> {
    let rest = stat.rsplit_once(')')?.1;
    let mut fields = rest.split_whitespace();
    // Tras el comm, el campo 3 es el primero. starttime es el campo 22 → índice 19.
    fields.nth(19)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_start_ticks;

    #[test]
    fn reads_starttime_even_when_comm_has_spaces() {
        let line = "42 (my proc) S 1 1 1 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 99991 0 0 0";
        assert_eq!(parse_start_ticks(line), Some(99991));
    }

    #[test]
    fn rejects_a_short_stat_line() {
        assert_eq!(parse_start_ticks("1 (init) S 0"), None);
    }
}
