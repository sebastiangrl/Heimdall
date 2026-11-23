//! Registro de acciones permitidas (sin ejecución automática).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianActionKind {
    RecommendCloseApps,
    RecommendFreeDisk,
    RecommendReviewThermal,
    RecommendDriverLogs,
    OpenBlackbox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardianAction {
    pub kind: GuardianActionKind,
    pub label: String,
    pub requires_user_approval: bool,
}

/// Solo acciones informativas por ahora — sin shell ni systemctl.
pub fn catalog() -> Vec<GuardianAction> {
    vec![
        GuardianAction {
            kind: GuardianActionKind::RecommendCloseApps,
            label: "Revisar aplicaciones que consumen recursos".into(),
            requires_user_approval: true,
        },
        GuardianAction {
            kind: GuardianActionKind::RecommendFreeDisk,
            label: "Liberar espacio en disco".into(),
            requires_user_approval: true,
        },
        GuardianAction {
            kind: GuardianActionKind::RecommendReviewThermal,
            label: "Revisar ventilación y carga térmica".into(),
            requires_user_approval: true,
        },
        GuardianAction {
            kind: GuardianActionKind::RecommendDriverLogs,
            label: "Revisar registros del controlador gráfico".into(),
            requires_user_approval: true,
        },
        GuardianAction {
            kind: GuardianActionKind::OpenBlackbox,
            label: "Abrir Caja negra".into(),
            requires_user_approval: false,
        },
    ]
}