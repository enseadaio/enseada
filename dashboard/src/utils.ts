import { Service } from "./http/Service";
import { IdMapFn } from "./types";

export const svcGetter = <T>(svcName: string) => (self: any): Service<T> => self[svcName];
export const mapId: IdMapFn<{ id: string }> = ({ id }) => id;
export const identity: <T>(a: T) => T = (a) => a;

export interface FileTreeNode {
  name: string,
  children?: FileTreeNode[],
}

export function buildFileTree(files: string[]): FileTreeNode {
  const paths = files.map((file) => {
    const trimmed = file.startsWith('/') ? file.slice(1) : file;
    return trimmed.split('/')
  });

  const tree = paths.reduce(assembleFiles, {} as FileTree)

  return {
    name: '/',
    children: mapTree(tree),
  };
}

export interface FileTree {
  [name: string]: FileTree
}

function assembleFiles(tree: FileTree, path: string[]): FileTree {
  const segment = path.shift();
  if (!segment) return tree;

  return {
    ...tree,
    [segment]: assembleFiles(tree[segment] || {}, path)
  }
}

function mapTree(tree: FileTree): FileTreeNode[] {
  return Object.entries(tree).map(([name, subTree]) => {
    const children = mapTree(subTree);
    if (children.length > 0) {
      return { name, children };
    } else {
      return { name };
    }
  });
}