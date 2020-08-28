export interface Role {
  role: string;
}

export interface Permission {
  subject?: string,
  object: string,
  action: string,
}