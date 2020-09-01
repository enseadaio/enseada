import { pascalCase } from "./pascalCase";

describe('pascalCase', () => {
  it('converts to PascalCase', () => {
    const converted = pascalCase('thisIs some_example-text');
    expect(converted).toBe('ThisIs SomeExampleText')
  })
})