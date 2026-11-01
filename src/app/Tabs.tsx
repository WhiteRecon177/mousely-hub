export type TabId = "mouse" | "macros" | "webcam";

type TabsProps = {
  active: TabId;
  onChange: (tab: TabId) => void;
};

const TABS: { id: TabId; label: string }[] = [
  { id: "mouse", label: "Mouse" },
  { id: "macros", label: "Macros" },
  { id: "webcam", label: "Webcam" },
];

export function Tabs({ active, onChange }: TabsProps) {
  return (
    <nav className="tabs">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          className={tab.id === active ? "tab tab-active" : "tab"}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </nav>
  );
}
