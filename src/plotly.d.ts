declare module 'plotly.js-dist-min' {
  interface PlotlyData {
    x?: number[];
    y?: number[];
    mode?: string;
    name?: string;
    [key: string]: unknown;
  }

  interface PlotlyLayout {
    title?: string | { text?: string; [key: string]: unknown };
    xaxis?: { title?: string; [key: string]: unknown };
    yaxis?: { type?: string; title?: string; rangemode?: string; [key: string]: unknown };
    [key: string]: unknown;
  }

  interface PlotlyConfig {
    displayModeBar?: boolean;
    [key: string]: unknown;
  }

  interface Plotly {
    newPlot(
      root: string | HTMLElement,
      data: PlotlyData[],
      layout?: PlotlyLayout,
      config?: PlotlyConfig
    ): Promise<void>;
  }
  const Plotly: Plotly;
  export default Plotly;
}

