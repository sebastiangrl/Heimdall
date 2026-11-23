import { useEffect, useState } from "react";
import { Shell, type NavKey } from "@/components/Shell";
import { BlackBoxPage } from "@/pages/BlackBoxPage";
import { IntelligencePage } from "@/pages/IntelligencePage";
import { HardwarePage } from "@/pages/HardwarePage";
import { OverviewPage } from "@/pages/OverviewPage";
import { ProcessesPage } from "@/pages/ProcessesPage";
import { ProfilesPage } from "@/pages/ProfilesPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { InvestigationsPage } from "@/pages/InvestigationsPage";
import { useWindowFrame } from "@/hooks/useWindowFrame";
import { useSettingsStore } from "@/store/settings";
import i18n from "@/i18n";

export default function App() {
  const reducedGlass = useSettingsStore((s) => s.reducedGlass);
  const locale = useSettingsStore((s) => s.locale);
  const [section, setSection] = useState<NavKey>("overview");
  useWindowFrame();

  useEffect(() => {
    document.body.classList.toggle("reduced-glass", reducedGlass);
  }, [reducedGlass]);

  useEffect(() => {
    if (i18n.language !== locale) {
      void i18n.changeLanguage(locale);
    }
  }, [locale]);

  return (
    <div className="app-shell">
      <Shell active={section} onNavigate={setSection}>
        {section === "overview" ? <OverviewPage /> : null}
        {section === "hardware" ? <HardwarePage /> : null}
        {section === "processes" ? <ProcessesPage /> : null}
        {section === "intelligence" ? <IntelligencePage /> : null}
        {section === "blackbox" ? <BlackBoxPage /> : null}
        {section === "investigations" ? <InvestigationsPage /> : null}
        {section === "profiles" ? <ProfilesPage /> : null}
        {section === "settings" ? <SettingsPage /> : null}
      </Shell>
    </div>
  );
}