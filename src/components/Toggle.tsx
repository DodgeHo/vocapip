interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
}
export function Toggle({ checked, onChange, label }: ToggleProps) {
  return (
    <button
      className={"toggle " + (checked ? "toggle-on" : "")}
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
    >
      <span />
    </button>
  );
}
