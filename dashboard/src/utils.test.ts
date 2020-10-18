import { buildFileTree } from "./utils";

describe('buildFileTree', () => {
  it('builds a file tree ', () => {
    const files = [
      '/test/scripts/world.txt',
      '/test/scripts/tree.txt',
      'another/test/file.js'
    ];
    const tree = buildFileTree(files);
    console.dir(JSON.stringify(tree, null, 2))
    expect(tree).toStrictEqual({
      name: '/',
      children: [
        {
          name: 'test',
          children: [
            {
              name: 'scripts',
              children: [
                {
                  name: 'world.txt',
                },
                {
                  name: 'tree.txt',
                },
              ],
            },
          ],
        },
        {
          name: 'another',
          children: [
            {
              name: 'test',
              children: [
                {
                  name: 'file.js',
                },
              ],
            },
          ],
        }
      ],
    });
  })
});
