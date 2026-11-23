import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Pause, Play, SkipBack, SkipForward } from "lucide-react";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent, LegendComponent, MarkAreaComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { InvestigationDossier } from "@/types/investigation";
import { buildReplayTimeline } from "@/lib/investigationReplay";
import { formatCpuPercent } from "@/lib/format";

echarts.use([LineChart, GridComponent, TooltipComponent, LegendComponent, MarkAreaComponent, CanvasRenderer]);

export function IncidentReplay({ dossier }: { dossier: InvestigationDossier }) {
  const { t, i18n } = useTranslation();
  const points = useMemo(() => buildReplayTimeline(dossier), [dossier]);
  const chartRef = useRef<HTMLDivElement>(null);
  const echartRef = useRef<echarts.ECharts | null>(null);
  const [idx, setIdx] = useState(0);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    setIdx(0);
    setPlaying(false);
  }, [dossier.id]);

  useEffect(() => {
    if (!chartRef.current) return;
    const chart = echarts.init(chartRef.current, undefined, { renderer: "canvas" });
    echartRef.current = chart;
    const burstArea = dossier.attribution.burst_hit_times.length
      ? [
          {
            xAxis: dossier.attribution.activity_interval.start,
          },
          {
            xAxis: dossier.attribution.activity_interval.end,
          },
        ]
      : undefined;

    chart.setOption({
      backgroundColor: "transparent",
      animation: false,
      grid: { left: 48, right: 16, top: 44, bottom: 36 },
      tooltip: {
        trigger: "axis",
        valueFormatter: (v: number) =>
          typeof v === "number" && Number.isFinite(v)
            ? `${v.toLocaleString(i18n.language, { minimumFractionDigits: 0, maximumFractionDigits: 2 })} %`
            : "—",
      },
      legend: {
        data: [
          t("investigations.chartGlobalCpu"),
          t("investigations.chartCoreCpu"),
          t("investigations.chartMachineCpu"),
        ],
        textStyle: { color: "#A69BB4" },
      },
      xAxis: {
        type: "time",
        axisLine: { lineStyle: { color: "#4a3d5c" } },
        axisLabel: {
          color: "#A69BB4",
          formatter: (v: number) =>
            new Date(v).toLocaleTimeString(i18n.language, { hour: "2-digit", minute: "2-digit", second: "2-digit" }),
        },
      },
      yAxis: {
        type: "value",
        min: 0,
        axisLabel: { color: "#A69BB4" },
        splitLine: { lineStyle: { color: "#2a2038" } },
      },
      series: [
        {
          name: t("investigations.chartGlobalCpu"),
          type: "line",
          showSymbol: false,
          connectNulls: false,
          data: points.map((p) => (p.globalCpu == null ? [p.t, null] : [p.t, p.globalCpu])),
          lineStyle: { color: "#A855F7", width: 2 },
        },
        {
          name: t("investigations.chartCoreCpu"),
          type: "line",
          showSymbol: false,
          connectNulls: false,
          data: points.map((p) =>
            p.processCoreCpu == null ? [p.t, null] : [p.t, p.processCoreCpu],
          ),
          lineStyle: { color: "#22D3EE", width: 2 },
        },
        {
          name: t("investigations.chartMachineCpu"),
          type: "line",
          showSymbol: false,
          connectNulls: false,
          data: points.map((p) =>
            p.processMachineCpu == null ? [p.t, null] : [p.t, p.processMachineCpu],
          ),
          lineStyle: { color: "#EC4899", width: 1.5, type: "dashed" },
        },
        {
          type: "line",
          markArea: burstArea
            ? {
                silent: true,
                itemStyle: { color: "rgba(168,85,247,0.12)" },
                data: [burstArea],
              }
            : undefined,
          data: [],
        },
      ],
    });
    return () => {
      chart.dispose();
      echartRef.current = null;
    };
  }, [points, dossier, t, i18n.language]);

  useEffect(() => {
    const chart = echartRef.current;
    const p = points[idx];
    if (!chart || !p) return;
    chart.dispatchAction({
      type: "showTip",
      seriesIndex: 0,
      dataIndex: idx,
    });
  }, [idx, points]);

  useEffect(() => {
    if (!playing || points.length < 2) return;
    const p0 = points[idx];
    const p1 = points[idx + 1];
    const delay = p1 ? Math.min(2000, Math.max(400, p1.t - p0.t)) : 800;
    const timer = window.setTimeout(() => {
      setIdx((i) => (i + 1 >= points.length ? 0 : i + 1));
    }, delay);
    return () => window.clearTimeout(timer);
  }, [playing, idx, points]);

  const current = points[idx];

  return (
    <section className="glass-panel space-y-4 p-4">
      <h3 className="font-display text-base">{t("investigations.replayTitle")}</h3>
      {!points.length ? (
        <p className="text-sm text-[var(--color-text-secondary)]">{t("investigations.replayEmpty")}</p>
      ) : (
        <>
          <div ref={chartRef} className="h-64 w-full" role="img" aria-label={t("investigations.replayTitle")} />
          <input
            type="range"
            min={0}
            max={Math.max(0, points.length - 1)}
            value={idx}
            className="w-full accent-[var(--color-violet)]"
            onChange={(e) => {
              setPlaying(false);
              setIdx(Number(e.target.value));
            }}
          />
          <div className="flex flex-wrap items-center gap-2">
            <button type="button" className="btn-ghost" onClick={() => setIdx((i) => Math.max(0, i - 1))}>
              <SkipBack size={16} />
            </button>
            <button
              type="button"
              className="btn-ghost"
              onClick={() => setPlaying((p) => !p)}
              aria-pressed={playing}
            >
              {playing ? <Pause size={16} /> : <Play size={16} />}
            </button>
            <button
              type="button"
              className="btn-ghost"
              onClick={() => setIdx((i) => Math.min(points.length - 1, i + 1))}
            >
              <SkipForward size={16} />
            </button>
            <span className="text-xs text-[var(--color-text-secondary)]">
              {new Date(current.at).toLocaleString(i18n.language)}
            </span>
          </div>
          {current ? (
            <dl className="grid gap-2 text-sm md:grid-cols-2">
              <div>
                <dt className="text-[var(--color-text-secondary)]">{t("investigations.metricGlobal")}</dt>
                <dd>{formatCpuPercent(current.globalCpu, i18n.language)}</dd>
              </div>
              <div>
                <dt className="text-[var(--color-text-secondary)]">{t("investigations.metricCore")}</dt>
                <dd>{formatProcessCpu(current.processState, current.processCoreCpu, t)}</dd>
              </div>
              <div>
                <dt className="text-[var(--color-text-secondary)]">{t("investigations.metricMachine")}</dt>
                <dd>
                  {current.processMachineCpu == null && current.processState !== "available"
                    ? formatProcessCpu(current.processState, null, t)
                    : formatCpuPercent(current.processMachineCpu, i18n.language, 2)}
                </dd>
              </div>
              <div>
                <dt className="text-[var(--color-text-secondary)]">{t("investigations.processState")}</dt>
                <dd>{t(`investigations.cpuState.${current.processState}`)}</dd>
              </div>
            </dl>
          ) : null}
        </>
      )}
    </section>
  );
}

function formatProcessCpu(
  state: string,
  value: number | null,
  t: (k: string) => string,
): string {
  if (state === "exited") return t("investigations.cpuState.exited");
  if (state === "not_tracked") return t("investigations.cpuState.not_tracked");
  if (state === "unavailable") return t("investigations.cpuState.unavailable");
  if (value == null) return "—";
  return `${value.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 1 })} %`;
}