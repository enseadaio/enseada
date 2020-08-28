import { Module as VuexModule } from "vuex";
import { Capabilities, User } from "../types";

export interface RootState {
  currentUser: User | null;
  capabilities: Capabilities | null;
}

export type Module<S> = VuexModule<S, RootState>;