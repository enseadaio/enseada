import { truncate } from "./truncate";

describe('truncate', () => {
  it('truncates with dots', () => {
    const converted = truncate('a nice long sentence', 6);
    expect(converted).toBe('a nice...');
  })
});