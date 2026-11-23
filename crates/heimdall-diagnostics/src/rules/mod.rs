//! Escalas de CPU:
//! - `cpu_saturation_percent`, load y el texto de CPU global: 0–100 de la máquina.
//! - `process_cpu_highlight` y `RelatedProcess::cpu_percent`: porcentaje de un núcleo
//!   (100 = un núcleo lógico). `cpu_machine_percent` es informativo y no dispara reglas.
//! El resto de reglas no umbralizan la CPU de un proceso.

mod cpu_saturation;
mod gpu_driver;
mod gpu_vram_pressure;
mod io_contention;
mod memory_pressure;
mod storage_pressure;
mod thermal_cpu;
mod thermal_gpu;

pub use cpu_saturation::evaluate_cpu_saturation;
pub use gpu_driver::evaluate_gpu_driver_journal;
pub use gpu_vram_pressure::evaluate_gpu_vram_pressure;
pub use io_contention::evaluate_io_contention;
pub use memory_pressure::evaluate_memory_pressure;
pub use storage_pressure::evaluate_storage_pressure;
pub use thermal_cpu::evaluate_cpu_thermal;
pub use thermal_gpu::evaluate_gpu_thermal;