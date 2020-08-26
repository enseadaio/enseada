import { Module as VuexModule } from "vuex";
import { User } from "../types";

export interface RootState {
  currentUser: User | null;
}

export type Module<S> = VuexModule<S, RootState>;