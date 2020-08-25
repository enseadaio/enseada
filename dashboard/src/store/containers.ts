import { Module } from "./types";
import { ContainerRepository, Page } from "../types";


interface ContainersState {
  repositories: Page<ContainerRepository>[]
}

const module: Module<ContainersState> = {
  state: () => ({ repositories: [] }),
  getters: {},
};

export default module;