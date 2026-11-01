import { ButtonId } from "../types";
import { translate, useT, type Key } from "../../../i18n";

interface MouseDiagramProps {
  selected: ButtonId | null;
  detected: ButtonId | null;
  enabled: boolean;
  scanning?: boolean;
  onSelect: (id: ButtonId) => void;
}

const BODY_PATH =
  "M144.3,18.2 C135.0,18.5 125.6,19.5 116.5,21.4 C107.4,23.3 98.1,25.9 89.7,29.7 C81.2,33.5 72.8,38.2 65.9,44.1 C59.0,50.1 52.7,57.4 48.3,65.3 C43.8,73.3 41.0,82.5 39.2,91.8 C37.4,101.1 37.4,111.2 37.3,121.1 C37.2,131.0 38.0,141.1 38.5,151.2 C38.9,161.3 39.4,171.5 39.8,181.7 C40.2,191.9 40.6,202.2 40.7,212.4 C40.9,222.5 41.1,232.7 40.8,242.7 C40.4,252.7 39.7,262.6 38.9,272.4 C38.0,282.3 36.5,292.0 35.7,301.8 C35.0,311.6 34.1,321.5 34.4,331.2 C34.6,341.0 35.5,350.8 37.3,360.2 C39.1,369.6 41.7,378.9 45.2,387.7 C48.8,396.4 53.1,405.0 58.4,412.8 C63.8,420.6 70.2,428.0 77.2,434.3 C84.2,440.5 92.4,446.0 100.6,450.1 C108.9,454.2 118.0,457.2 127.0,459.1 C135.9,460.9 145.3,461.5 154.4,461.0 C163.5,460.5 172.8,458.9 181.6,456.1 C190.3,453.3 199.1,449.2 206.9,444.1 C214.7,439.1 222.2,432.6 228.5,425.6 C234.8,418.7 240.3,410.6 244.9,402.3 C249.4,394.1 252.9,385.2 255.7,376.1 C258.5,367.0 260.4,357.5 261.5,347.9 C262.6,338.3 262.7,328.4 262.5,318.6 C262.2,308.8 261.0,299.0 260.0,289.2 C259.0,279.4 257.5,269.7 256.6,259.9 C255.8,250.0 255.0,240.0 254.8,230.0 C254.5,220.0 254.7,209.9 255.1,199.9 C255.4,189.9 256.2,179.9 256.9,170.0 C257.7,160.0 258.8,150.1 259.4,140.2 C259.9,130.3 260.8,120.3 260.4,110.6 C259.9,100.9 259.1,90.9 256.5,82.0 C253.9,73.1 249.9,64.3 244.6,57.0 C239.2,49.7 232.0,43.3 224.5,38.1 C217.0,32.9 208.1,29.1 199.3,26.0 C190.6,22.9 181.2,20.9 172.1,19.6 C162.9,18.3 153.5,17.9 144.3,18.2 Z";

const LEFT_ZONE = "M 143.7,-5.4 L 143.7,189.0 L 76.5,187.7 C 65.2,183.5 55.5,168.8 54.3,152.0 L 53.0,57.6 L 53.0,-5.4 Z";
const RIGHT_ZONE = "M 154.2,-5.4 L 154.2,189.0 L 222.6,187.7 C 237.3,181.4 245.7,168.8 247.0,152.0 L 247.0,57.6 L 247.0,-5.4 Z";

const DETAIL_LINES = [
  "M 53.9,57.6 L 54.3,152.0 C 55.5,168.8 65.2,183.5 76.5,187.7 L 143.7,189.0",
  "M 247.0,57.6 L 247.0,152.0 C 245.7,168.8 237.3,181.4 222.6,187.7 L 154.2,189.0",
  "M 143.7,16.4 L 143.7,60.9",
  "M 143.7,127.7 L 143.7,189.0",
  "M 154.2,16.4 L 154.2,60.9",
  "M 154.2,127.7 L 154.2,189.0",
];

const LABEL_KEYS: Record<ButtonId, Key> = {
  left: "btn.left",
  right: "btn.right",
  middle: "btn.middle",
  back: "btn.back",
  forward: "btn.forward",
};

export function MouseDiagram({
  selected,
  detected,
  enabled,
  scanning = false,
  onSelect,
}: MouseDiagramProps) {
  useT();
  const zoneClass = (id: ButtonId): string => {
    const classes = ["dd-zone"];
    if (id === selected) classes.push("dd-zone--selected");
    if (id === detected) classes.push("dd-zone--detected");
    if (id === "left" || id === "right") classes.push("dd-zone--locked");
    return classes.join(" ");
  };

  const zoneProps = (id: ButtonId) => ({
    className: zoneClass(id),
    role: "button" as const,
    "aria-label": translate(LABEL_KEYS[id]),
    "data-zone": id,
    onClick: () => onSelect(id),
  });

  return (
    <svg
      className={enabled ? "dd-mouse" : "dd-mouse dd-mouse--off"}
      viewBox="0 0 300 480"
      role="img"
      aria-label={translate("mouse.diagram")}
    >
      <defs>
        <clipPath id="dd-body-clip">
          <path d={BODY_PATH} />
        </clipPath>

        <pattern id="dd-dots" width="10" height="10" patternUnits="userSpaceOnUse">
          <circle className="dd-mouse-dot" cx="5" cy="5" r="0.9" />
        </pattern>

        <pattern id="dd-dots-dense" width="3.4" height="3.4" patternUnits="userSpaceOnUse">
          <circle className="dd-mouse-dot dd-mouse-dot--dense" cx="1.7" cy="1.7" r="0.8" />
        </pattern>

        <pattern id="dd-dots-lit" width="3" height="3" patternUnits="userSpaceOnUse">
          <circle className="dd-mouse-dot dd-mouse-dot--lit" cx="1.5" cy="1.5" r="1.05" />
        </pattern>

        <linearGradient id="dd-scan-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style={{ stopColor: "var(--holo)" }} stopOpacity="0" />
          <stop offset="0.5" style={{ stopColor: "var(--holo-2)" }} stopOpacity="0.55" />
          <stop offset="1" style={{ stopColor: "var(--holo)" }} stopOpacity="0" />
        </linearGradient>
      </defs>

      <g className="dd-holo">
        <path className="dd-holo-tint" d={BODY_PATH} />
        <path className="dd-holo-dots" d={BODY_PATH} />
        <path className="dd-holo-rim" d={BODY_PATH} clipPath="url(#dd-body-clip)" />
        <path className="dd-holo-outline" d={BODY_PATH} />
        <rect className="dd-holo-detail" x="41.3" y="157.5" width="10.1" height="55" rx="5.0" />
        <rect className="dd-holo-detail" x="41.3" y="220" width="10.1" height="55" rx="5.0" />
        {DETAIL_LINES.map((d) => (
          <path key={d} className="dd-holo-detail" d={d} />
        ))}
        <rect className="dd-holo-detail" x="131.5" y="60.9" width="34.4" height="66.8" rx="17.2" />
      </g>

      <path clipPath="url(#dd-body-clip)" d={LEFT_ZONE} {...zoneProps("left")} />
      <path clipPath="url(#dd-body-clip)" d={RIGHT_ZONE} {...zoneProps("right")} />

      <rect x="137.8" y="63.0" width="22.7" height="60.5" rx="11.3" {...zoneProps("middle")} />

      <rect x="41.3" y="157.5" width="10.1" height="55" rx="5.0" {...zoneProps("back")} />
      <rect x="41.3" y="220" width="10.1" height="55" rx="5.0" {...zoneProps("forward")} />

      {scanning && (
        <g clipPath="url(#dd-body-clip)" className="dd-scan-clip">
          <rect className="dd-scan" x="0" y="0" width="300" height="16" fill="url(#dd-scan-grad)" />
        </g>
      )}
    </svg>
  );
}