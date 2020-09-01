import { titleCase } from "./titleCase";

describe('titleCase', () => {
  it('converts to Title case', () => {
    const converted = titleCase('a nice Sentence');
    expect(converted).toBe('A nice Sentence');
  })
});