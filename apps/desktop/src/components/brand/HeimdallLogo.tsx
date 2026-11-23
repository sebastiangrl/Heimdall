export function HeimdallLogo({ compact = false }: { compact?: boolean }) {
  return (
    <div className={`flex items-center gap-3 ${compact ? "justify-center" : ""}`}>
      <svg
        width={compact ? 36 : 42}
        height={compact ? 36 : 42}
        viewBox="0 0 64 64"
        fill="none"
        aria-hidden
        className="shrink-0"
      >
        <defs>
          <radialGradient id="heimdall-core" cx="50%" cy="40%" r="55%">
            <stop offset="0%" stopColor="#EC4899" />
            <stop offset="55%" stopColor="#A855F7" />
            <stop offset="100%" stopColor="#7C3AED" />
          </radialGradient>
        </defs>
        <circle cx="32" cy="32" r="28" stroke="url(#heimdall-core)" strokeWidth="2" opacity="0.85" />
        <circle cx="32" cy="32" r="14" fill="url(#heimdall-core)" opacity="0.95" />
        <circle cx="32" cy="32" r="6" fill="#F5F0FA" opacity="0.9" />
      </svg>
      {!compact ? (
        <div>
          <p className="font-display text-lg font-semibold tracking-tight text-[var(--color-text-primary)]">
            Heimdall
          </p>
          <p className="text-[11px] leading-tight text-[var(--color-text-secondary)]">
            Lo ve todo. Protege lo que importa.
          </p>
        </div>
      ) : null}
    </div>
  );
}