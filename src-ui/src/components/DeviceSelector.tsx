import React, { useState, useRef, useEffect, useCallback } from "react";
import { Headphones, Check, ChevronDown } from "lucide-react";

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
}

interface DeviceSelectorProps {
  devices: AudioDeviceInfo[];
  activeDeviceId: string;
  onSelectDevice: (deviceId: string) => void;
  disabled?: boolean;
}

export const DeviceSelector: React.FC<DeviceSelectorProps> = ({
  devices,
  activeDeviceId,
  onSelectDevice,
  disabled = false,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const activeDevice = devices.find((d) => d.id === activeDeviceId) ||
    devices.find((d) => d.is_default) ||
    devices[0] || {
      id: "default",
      name: "Default Audio Device",
      is_default: true,
    };

  // Close when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  const handleSelect = useCallback(
    (id: string) => {
      onSelectDevice(id);
      setIsOpen(false);
    },
    [onSelectDevice]
  );

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled || devices.length === 0) return;

    if (!isOpen) {
      if (e.key === "Enter" || e.key === " " || e.key === "ArrowDown") {
        e.preventDefault();
        setIsOpen(true);
        const idx = devices.findIndex((d) => d.id === activeDevice.id);
        setFocusedIndex(idx >= 0 ? idx : 0);
      }
      return;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      setIsOpen(false);
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusedIndex((prev) => (prev + 1) % devices.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusedIndex((prev) => (prev - 1 + devices.length) % devices.length);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (focusedIndex >= 0 && focusedIndex < devices.length) {
        handleSelect(devices[focusedIndex].id);
      }
    }
  };

  return (
    <div
      ref={containerRef}
      className="relative inline-block text-left"
      onKeyDown={handleKeyDown}
    >
      {/* Trigger Button */}
      <button
        type="button"
        disabled={disabled}
        onClick={() => !disabled && setIsOpen(!isOpen)}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-label={`Select audio playback device. Current device: ${activeDevice.name}`}
        title={`Active WASAPI Output: ${activeDevice.name}`}
        className={`flex items-center gap-2 bg-surface-card border px-2.5 py-1 rounded-md text-xs transition-colors cursor-pointer select-none ${
          isOpen
            ? "border-accent-cyan ring-1 ring-accent-cyan/30 text-text-primary"
            : "border-border-subtle hover:border-border-default text-text-secondary hover:text-text-primary"
        } ${disabled ? "opacity-50 cursor-not-allowed" : ""}`}
      >
        <span
          className="w-1.5 h-1.5 rounded-full bg-status-online shadow-[0_0_6px_#22c55e] shrink-0"
          aria-hidden="true"
        />
        <Headphones className="w-3.5 h-3.5 text-text-muted shrink-0" />
        <span className="max-w-[130px] truncate font-medium text-text-primary">
          {activeDevice.name}
        </span>
        <ChevronDown
          className={`w-3 h-3 text-text-muted transition-transform shrink-0 ${
            isOpen ? "rotate-180 text-accent-cyan" : ""
          }`}
        />
      </button>

      {/* Popover Dropdown */}
      {isOpen && (
        <ul
          ref={listRef}
          role="listbox"
          aria-label="Audio Output Devices"
          className="absolute left-0 mt-1.5 min-w-[220px] max-w-[280px] bg-surface-card border border-border-default rounded-lg shadow-2xl py-1.5 px-1 z-50 backdrop-blur-md focus:outline-none animate-in fade-in zoom-in-95 duration-100"
        >
          <li className="px-2 py-1 text-[10px] font-mono uppercase tracking-wider text-text-muted border-b border-border-subtle mb-1">
            Playback Endpoints ({devices.length})
          </li>
          {devices.map((device, index) => {
            const isSelected = device.id === activeDevice.id;
            const isFocused = index === focusedIndex;

            return (
              <li
                key={device.id}
                role="option"
                aria-selected={isSelected}
                onClick={() => handleSelect(device.id)}
                onMouseEnter={() => setFocusedIndex(index)}
                className={`flex items-center justify-between gap-2 px-2.5 py-1.5 rounded text-xs cursor-pointer select-none transition-colors ${
                  isFocused
                    ? "bg-surface-elevated text-text-primary"
                    : "text-text-secondary hover:text-text-primary"
                } ${isSelected ? "text-accent-cyan font-medium" : ""}`}
              >
                <div className="flex items-center gap-2 truncate">
                  <span className="w-3.5 h-3.5 flex items-center justify-center shrink-0">
                    {isSelected && (
                      <Check className="w-3.5 h-3.5 text-accent-cyan" />
                    )}
                  </span>
                  <span className="truncate" title={device.name}>
                    {device.name}
                  </span>
                </div>
                {device.is_default && (
                  <span className="text-[9px] font-mono px-1 py-0.2 bg-border-subtle text-text-muted rounded shrink-0">
                    DEFAULT
                  </span>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
};
