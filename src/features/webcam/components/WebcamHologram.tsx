// Decorative hologram webcam, shown when there is no camera signal.
export function WebcamHologram() {
  return (
    <svg
      className="dd-cam-holo"
      viewBox="90 220 1080 840"
      role="img"
      aria-label="Webcam hologram"
    >
      <defs>
        <pattern id="dd-cam-dots" width="20" height="20" patternUnits="userSpaceOnUse">
          <circle className="dd-mouse-dot" cx="10" cy="10" r="3.2" />
        </pattern>
        <pattern id="dd-cam-dots-fine" width="10" height="10" patternUnits="userSpaceOnUse">
          <circle className="dd-mouse-dot dd-mouse-dot--dense" cx="5" cy="5" r="2.6" />
        </pattern>
        <clipPath id="dd-cam-body-clip">
          <rect x="110" y="250" width="1040" height="440" rx="170" />
        </clipPath>
      </defs>

      {/* body */}
      <rect className="dd-cam-holo-fill" x="110" y="250" width="1040" height="440" rx="170" />
      <g clipPath="url(#dd-cam-body-clip)">
        <rect className="dd-cam-holo-edge" x="110" y="250" width="70" height="440" />
        <rect className="dd-cam-holo-edge" x="1080" y="250" width="70" height="440" />
        <rect className="dd-cam-holo-edge" x="110" y="250" width="1040" height="40" />
      </g>
      <rect className="dd-cam-holo-line" x="110" y="250" width="1040" height="440" rx="170" />
      <rect className="dd-cam-holo-line" x="165" y="285" width="930" height="370" rx="130" />
      <line className="dd-cam-holo-line" x1="110" y1="479" x2="1150" y2="479" />

      {/* status slots */}
      <rect className="dd-cam-holo-line" x="345" y="472" width="44" height="14" rx="7" />
      <rect className="dd-cam-holo-line" x="865" y="472" width="44" height="14" rx="7" />

      {/* lens */}
      <circle className="dd-cam-holo-line" cx="630" cy="470" r="165" />
      <circle className="dd-cam-holo-line" cx="630" cy="470" r="145" />
      <circle className="dd-cam-holo-fill" cx="630" cy="470" r="115" />
      <circle className="dd-cam-holo-line" cx="630" cy="470" r="115" />
      <circle className="dd-cam-holo-line" cx="630" cy="470" r="90" />
      <circle className="dd-cam-holo-dense" cx="630" cy="470" r="72" />
      <circle className="dd-cam-holo-line" cx="630" cy="470" r="72" />
      <ellipse className="dd-cam-holo-line" cx="630" cy="470" rx="34" ry="72" />
      <line className="dd-cam-holo-line" x1="558" y1="470" x2="702" y2="470" />
      <circle className="dd-cam-holo-glint" cx="601" cy="444" r="7" />

      {/* neck */}
      <rect className="dd-cam-holo-fill" x="518" y="690" width="220" height="95" />
      <rect className="dd-cam-holo-line" x="518" y="690" width="220" height="95" rx="10" />
      <line className="dd-cam-holo-line" x1="540" y1="690" x2="540" y2="770" />
      <line className="dd-cam-holo-line" x1="716" y1="690" x2="716" y2="770" />

      {/* base */}
      <rect className="dd-cam-holo-fill" x="390" y="905" width="478" height="130" rx="42" />
      <rect className="dd-cam-holo-line" x="390" y="905" width="478" height="130" rx="42" />
      <rect className="dd-cam-holo-fill" x="373" y="745" width="513" height="160" rx="38" />
      <rect className="dd-cam-holo-line" x="373" y="745" width="513" height="160" rx="38" />
      <line className="dd-cam-holo-line" x1="373" y1="800" x2="886" y2="800" />
    </svg>
  );
}