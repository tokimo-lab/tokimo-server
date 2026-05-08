type Props = {
  /** value in 0..1 */
  value: number;
  size?: number;
  stroke?: number;
  trackColor?: string;
  fillColor?: string;
};

export function ActivityRing({
  value,
  size = 180,
  stroke = 18,
  trackColor = "rgba(139,92,246,0.15)",
  fillColor = "#8b5cf6",
}: Props) {
  const clamped = Math.min(Math.max(value, 0), 1);
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const dash = clamped * circumference;
  const center = size / 2;

  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      role="img"
      aria-label={`${Math.round(clamped * 100)}%`}
    >
      <defs>
        <linearGradient id="ringGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#a78bfa" />
          <stop offset="100%" stopColor={fillColor} />
        </linearGradient>
      </defs>
      <circle
        cx={center}
        cy={center}
        r={radius}
        stroke={trackColor}
        strokeWidth={stroke}
        fill="none"
      />
      <circle
        cx={center}
        cy={center}
        r={radius}
        stroke="url(#ringGradient)"
        strokeWidth={stroke}
        strokeLinecap="round"
        fill="none"
        strokeDasharray={`${dash} ${circumference - dash}`}
        transform={`rotate(-90 ${center} ${center})`}
        style={{
          transition: "stroke-dasharray 600ms cubic-bezier(0.4,0,0.2,1)",
        }}
      />
    </svg>
  );
}
