import { useEffect, useRef, type ReactNode } from "react";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
}

export function Modal({ open, onClose, title, children }: ModalProps) {
  const ref = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (open && !el.open) el.showModal();
    else if (!open && el.open) el.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      className="backdrop:bg-black/60 backdrop:backdrop-blur-sm bg-surface-1 rounded-2xl border border-border shadow-2xl p-0 w-[calc(100%-2rem)] max-w-md animate-slide-up"
    >
      <div className="flex items-center justify-between px-5 py-4 border-b border-border">
        <h2 className="text-base font-semibold text-fg">{title}</h2>
        <button
          onClick={onClose}
          className="w-8 h-8 flex items-center justify-center rounded-lg text-fg-muted hover:text-fg hover:bg-surface-2 transition-colors"
          aria-label="Close"
        >
          ✕
        </button>
      </div>
      <div className="overflow-y-auto p-5 max-h-[70vh]">{children}</div>
    </dialog>
  );
}
