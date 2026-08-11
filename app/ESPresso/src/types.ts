export type Profile = {
  deviceId: string;
  name: string;
  role: string;
  bio: string;
};

export type Contact = {
  id: number;
  deviceId: string;
  name: string;
  role: string;
  bio: string;
  savedAt: string;
};

export type Device = {
  id: number;
  name: string;
  host: string;
  port: number;
  source: "manual" | "auto";
  lastSeen: string | null;
};

export type DiscoveredDevice = {
  name: string;
  host: string;
  ip: string | null;
  port: number;
};

export type HostInfo = {
  hostname: string;
  port: number;
  instance: string;
};

export type Status = "connecting" | "connected" | "disconnected" | "error";

export type ConnectionStatus = {
  state: Status;
  host: string | null;
  message: string | null;
};

export type Screen = "create" | "dashboard" | "contacts" | "history" | "devices";
