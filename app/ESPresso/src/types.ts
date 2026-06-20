export type Profile = {
  deviceId: string;
  name: string;
  role: string;
  bio: string;
};

export type DBProfile = Profile & {
    id: number;
    created_at: string;
};

export type Contact = Profile & {
  id: number;
  saved_at: string;
};

export type WSMessage = {
  type: string;
  data: Profile[];
};

export type Status = "connecting" | "connected" | "disconnected" | "error";
export type Screen = "create" | "dashboard" | "contacts" | "history";
