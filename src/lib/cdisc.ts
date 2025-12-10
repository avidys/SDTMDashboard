export type DMRow = { USUBJID: string; ARM?: string; SEX?: string; AGE?: string };
export type PCRow = {
  USUBJID: string;
  PCTPT?: string;     // nominal time label
  PCTPTNUM?: string;  // numeric nominal time
  PCDTC?: string;     // actual collection datetime (ISO)
  PCSTRESN?: string;  // concentration numeric
  PCSTRESU?: string;  // units
};

export function parseCSV(text: string): Record<string, string>[] {
  const [head, ...rows] = text.split(/\r?\n/).filter(Boolean);
  const cols = head.split(",").map((s) => s.replace(/(^"|"$)/g, ""));
  return rows.map(r => {
    const vals = r.match(/(".*?"|[^,]+)(?=,|$)/g)?.map(s=>s.replace(/^"(.*)"$/,'$1')) ?? [];
    const o: Record<string,string> = {};
    cols.forEach((c,i)=> o[c.trim().toUpperCase()] = (vals[i] ?? "").trim());
    return o;
  });
}

export function dmSubjects(dm: Record<string,string>[]): DMRow[] {
  return dm.map(r => ({ USUBJID: r.USUBJID, ARM: r.ARM, SEX: r.SEX, AGE: r.AGE }));
}

export function pcForSubject(pc: Record<string,string>[], usubjid: string): PCRow[] {
  return pc.filter(r => r.USUBJID === usubjid)
           .map(r => r as PCRow)
           .filter(r => r.PCSTRESN && !Number.isNaN(Number(r.PCSTRESN)));
}

export function timeFromPC(row: PCRow): number | null {
  // Prefer numeric nominal time if present, else try to parse from PCTPT like "0.5 h"
  if (row.PCTPTNUM && !Number.isNaN(Number(row.PCTPTNUM))) return Number(row.PCTPTNUM);
  if (row.PCTPT) {
    const m = row.PCTPT.match(/([\d.]+)/);
    if (m) return Number(m[1]);
  }
  return null;
}