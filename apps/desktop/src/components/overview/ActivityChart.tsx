import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent, LegendComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";

echarts.use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer]);

const MAX_POINTS = 60;

export function ActivityChart({
  cpu,
  memoryPercent,
  hidden,
}: {
  cpu: number | null;
  memoryPercent: number | null;
  hidden?: boolean;
}) {
  const { t, i18n } = useTranslation();
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<echarts.ECharts | null>(null);
  const seriesRef = useRef<{ cpu: number[]; mem: number[]; ts: number[] }>({
    cpu: [],
    mem: [],
    ts: [],
  });

  useEffect(() => {
    if (!ref.current) return;
    const chart = echarts.init(ref.current, undefined, { renderer: "canvas" });
    chartRef.current = chart;

    chart.setOption({
      backgroundColor: "transparent",
      animation: !hidden,
      grid: { left: 44, right: 12, top: 40, bottom: 32 },
      tooltip: {
        trigger: "axis",
        valueFormatter: (v: unknown) => `${Number(v).toFixed(1)} %`,
      },
      legend: {
        data: [
          { name: t("chart.cpu"), icon: "circle", textStyle: { color: "#A855F7" } },
          { name: t("chart.memory"), icon: "circle", textStyle: { color: "#EC4899" } },
        ],
      },
      xAxis: {
        type: "time",
        splitNumber: 6,
        axisLine: { lineStyle: { color: "#4a3d5c" } },
        axisLabel: {
          color: "#A69BB4",
          hideOverlap: true,
          formatter: (v: number) =>
            new Date(v).toLocaleTimeString(i18n.language, {
              hour: "2-digit",
              minute: "2-digit",
              second: "2-digit",
            }),
        },
      },
      yAxis: {
        type: "value",
        min: 0,
        max: 100,
        splitLine: { lineStyle: { color: "#2a2038" } },
        axisLabel: { color: "#A69BB4", formatter: "{value}%" },
      },
      series: [
        {
          name: t("chart.cpu"),
          type: "line",
          smooth: 0.35,
          showSymbol: false,
          data: [],
          lineStyle: { color: "#A855F7", width: 2 },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: "rgba(168, 85, 247, 0.28)" },
              { offset: 1, color: "rgba(168, 85, 247, 0.02)" },
            ]),
          },
        },
        {
          name: t("chart.memory"),
          type: "line",
          smooth: 0.35,
          showSymbol: false,
          data: [],
          lineStyle: { color: "#EC4899", width: 2 },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: "rgba(236, 72, 153, 0.22)" },
              { offset: 1, color: "rgba(236, 72, 153, 0.02)" },
            ]),
          },
        },
      ],
    });

    const resize = () => chart.resize();
    window.addEventListener("resize", resize);
    return () => {
      window.removeEventListener("resize", resize);
      chart.dispose();
    };
  }, [hidden, i18n.language, t]);

  useEffect(() => {
    if (hidden || (cpu === null && memoryPercent === null)) return;
    const chart = chartRef.current;
    if (!chart) return;
    const s = seriesRef.current;
    const now = Date.now();
    s.ts.push(now);
    s.cpu.push(cpu ?? 0);
    s.mem.push(memoryPercent ?? 0);
    if (s.ts.length > MAX_POINTS) {
      s.ts.shift();
      s.cpu.shift();
      s.mem.shift();
    }
    const cpuData = s.ts.map((t, i) => [t, s.cpu[i]]);
    const memData = s.ts.map((t, i) => [t, s.mem[i]]);
    chart.setOption({
      series: [{ data: cpuData }, { data: memData }],
    });
  }, [cpu, memoryPercent, hidden]);

  return (
    <div className="glass-panel-strong p-4">
      <h3 className="font-display text-lg text-[var(--color-text-primary)]">{t("overview.activityTitle")}</h3>
      <p className="mb-2 text-xs text-[var(--color-text-secondary)]">{t("overview.activityHint")}</p>
      <div ref={ref} className="h-52 w-full" />
    </div>
  );
}