import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Contact,
  ConnectionStatus,
  Device,
  DiscoveredDevice,
  HostInfo,
  Profile,
} from "../types";

export const EVENTS = {
  status: "connection://status",
  profiles: "profiles://updated",
  contacts: "contacts://updated",
  devices: "devices://updated",
  discovery: "discovery://done",
} as const;

/** Typed wrappers around the Rust backend commands. */
export const api = {
  getDeviceId: () => invoke<string>("get_device_id"),
  getStatus: () => invoke<ConnectionStatus>("get_status"),
  getHostInfo: () => invoke<HostInfo>("get_host_info"),
  getProfiles: () => invoke<Profile[]>("get_profiles"),
  getContacts: () => invoke<Contact[]>("get_contacts"),
  getDevices: () => invoke<Device[]>("get_devices"),

  addContact: (profile: Profile) => invoke<Contact[]>("add_contact", { profile }),
  addDevice: (host: string) => invoke<Device[]>("add_device", { host }),
  removeDevice: (id: number) => invoke<Device[]>("remove_device", { id }),

  connectTo: (host: string) => invoke<void>("connect_to", { host }),
  connectAuto: () => invoke<void>("connect_auto"),
  retry: () => invoke<void>("retry_connection"),
  sendProfile: (profile: Profile) => invoke<void>("send_profile", { profile }),
  discover: () => invoke<DiscoveredDevice[]>("discover_devices"),
};

/** Event subscription helpers — each returns an unlisten function. */
export function onStatus(cb: (s: ConnectionStatus) => void): Promise<UnlistenFn> {
  return listen<ConnectionStatus>(EVENTS.status, (e) => cb(e.payload));
}
export function onProfiles(cb: (p: Profile[]) => void): Promise<UnlistenFn> {
  return listen<Profile[]>(EVENTS.profiles, (e) => cb(e.payload));
}
export function onContacts(cb: (c: Contact[]) => void): Promise<UnlistenFn> {
  return listen<Contact[]>(EVENTS.contacts, (e) => cb(e.payload));
}
export function onDevices(cb: (d: Device[]) => void): Promise<UnlistenFn> {
  return listen<Device[]>(EVENTS.devices, (e) => cb(e.payload));
}
export function onDiscovery(cb: (d: DiscoveredDevice[]) => void): Promise<UnlistenFn> {
  return listen<DiscoveredDevice[]>(EVENTS.discovery, (e) => cb(e.payload));
}
