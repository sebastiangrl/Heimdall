import { useEffect, useRef } from "react";
import * as echarts from "echarts";
import { useTranslation } from "react-i18next";

export interface TelemetryFrame {
  observed_at: string;
  cpu_percent?: number | null;
  mem_available_bytes?: number | null;
  gpu_percent?: number | null;
}

interface Props {
  pre: TelemetryFrame[];
  post: TelemetryFrame[];
  incidentAt: string;
}

export function IncidentTimeline({ pre, post, incidentAt }: Props) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const chart = echarts.init(el, undefined, { renderer: "canvas" });
    const frames = [...pre, ...post];
    const times = frames.map((f) => new Date(f.observed_at).toLocaleTimeString());
    const cpu = frames.map((f) => f.cpu_percent ?? null);
    const memGb = frames.map((f) =>
      f.mem_available_bytes != null ? f.mem_available_bytes / (1024 ** 3) : null,
    );
    const gpu = frames.map((f) => f.gpu_percent ?? null);
    const markIdx = Math.max(0, pre.length - 1);

    chart.setOption({
      backgroundColor: "transparent",
      grid: { left: 48, right: 24, top: 32, bottom: 36 },
      tooltip: { trigger: "axis" },
      legend: {
        data: [t("blackbox.chartCpu"), t("blackbox.chartMemAvail"), t("blackbox.chartGpu")],
        textStyle: { color: "#a69bb4" },
      },
      xAxis: {
        type: "category",
        data: times,
        axisLabel: { color: "#a69bb4", fontSize: 10 },
      },
      yAxis: [
        { type: "value", name: "%", axisLabel: { color: "#a69bb4" }, splitLine: { lineStyle: { color: "rgba(255,255,255,0.06)" } } },
        { type: "value", name: "GB", axisLabel: { color: "#a69bb4" }, splitLine: { show: false } },
      ],
      series: [
        {
          name: t("blackbox.chartCpu"),
          type: "line",
          data: cpu,
          smooth: true,
          showSymbol: false,
          lineStyle: { color: "#A855F7", width: 2 },
          markLine: {
            symbol: "none",
            label: { formatter: t("blackbox.incidentMarker"), color: "#EC4899" },
            lineStyle: { color: "#EC4899", type: "dashed" },
            data: markIdx >= 0 ? [{ xAxis: times[markIdx] ?? incidentAt }] : [],
          },
        },
        {
          name: t("blackbox.chartMemAvail"),
          type: "line",
          yAxisIndex: 1,
          data: memGb,
          smooth: true,
          showSymbol: false,
          lineStyle: { color: "#EC4899", width: 2 },
        },
        {
          name: t("blackbox.chartGpu"),
          type: "line",
          data: gpu,
          smooth: true,
          showSymbol: false,
          lineStyle: { color: "#61d6a5", width: 2 },
        },
      ],
    });
    const resize = () => chart.resize();
    window.addEventListener("resize", resize);
    return () => {
      window.removeEventListener("resize", resize);
      chart.dispose();
    };
  }, [pre, post, incidentAt, t]);

  return <div ref={ref} className="h-56 w-full md:h-64" />;
}