export type Profile = {
  name: string;
  role: string;
  bio: string;
};

export type WSMessage = {
  type: string;
  data: Profile[];
};

export type Status = "connecting" | "connected" | "disconnected" | "error";
export type Screen = "create" | "dashboard";
