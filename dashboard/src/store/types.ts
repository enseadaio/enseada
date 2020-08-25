import { Module as VuexModule } from "vuex";

export interface RootState {

}

export type Module<S> = VuexModule<S, RootState>;